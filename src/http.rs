use std::sync::Arc;
use std::time::Duration;

use encoding_rs::WINDOWS_1252;
use tracing::{info, warn};

use crate::error::{LauyerError, Result};

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait HttpFetcher: Send + Sync {
    async fn get(&self, url: &str) -> Result<Vec<u8>>;
    async fn get_text(&self, url: &str) -> Result<String>;
    async fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
        headers: &[(String, String)],
    ) -> Result<String>;
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct HttpClient {
    client: reqwest::Client,
    cookie_jar: Arc<reqwest::cookie::Jar>,
    retries: u32,
}

impl HttpClient {
    pub fn new(proxy: Option<&str>, timeout_secs: u64, retries: u32) -> Result<Self> {
        let cookie_jar = Arc::new(reqwest::cookie::Jar::default());

        let mut builder = reqwest::Client::builder()
            .cookie_provider(Arc::clone(&cookie_jar))
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Mozilla/5.0 (compatible; lauyer/0.1; +https://github.com/nikuscs/lauyer)")
            .gzip(true)
            .brotli(true);

        if let Some(proxy_url) = proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| LauyerError::Http { source: e, url: proxy_url.to_owned() })?;
            builder = builder.proxy(proxy);
        }

        let client = builder
            .build()
            .map_err(|e| LauyerError::Http { source: e, url: "<client build>".to_owned() })?;

        Ok(Self { client, cookie_jar, retries })
    }

    pub const fn cookie_jar(&self) -> &Arc<reqwest::cookie::Jar> {
        &self.cookie_jar
    }

    pub const fn inner(&self) -> &reqwest::Client {
        &self.client
    }

    pub async fn get_latin1(&self, url: &str) -> Result<String> {
        let bytes = self.get_bytes(url).await?;
        let (cow, _encoding, had_errors) = WINDOWS_1252.decode(&bytes);
        if had_errors {
            warn!(url, "Latin-1 decoding encountered unmappable bytes");
        }
        Ok(cow.into_owned())
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        self.get(url).await
    }

    async fn execute_with_retry(
        &self,
        url: &str,
        build: impl Fn() -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let mut last_err: Option<LauyerError> = None;

        for attempt in 0..=self.retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(500 * (1u64 << (attempt - 1)));
                info!(attempt, ?backoff, "Retrying request after backoff");
                tokio::time::sleep(backoff).await;
            }

            let response = build().send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        return Ok(resp);
                    }
                    if is_retryable_status(status) && attempt < self.retries {
                        warn!(attempt, %status, "Retryable HTTP status");
                        last_err = Some(LauyerError::Http {
                            source: resp.error_for_status().unwrap_err(),
                            url: url.to_owned(),
                        });
                        continue;
                    }
                    return resp.error_for_status().map_err(|e| LauyerError::Http {
                        url: e.url().map_or_else(|| "<unknown>".to_owned(), ToString::to_string),
                        source: e,
                    });
                }
                Err(e) => {
                    if is_retryable_error(&e) && attempt < self.retries {
                        warn!(attempt, error = %e, "Retryable request error");
                        last_err = Some(LauyerError::Http {
                            url: e
                                .url()
                                .map_or_else(|| "<unknown>".to_owned(), ToString::to_string),
                            source: e,
                        });
                        continue;
                    }
                    return Err(LauyerError::Http {
                        url: e.url().map_or_else(|| "<unknown>".to_owned(), ToString::to_string),
                        source: e,
                    });
                }
            }
        }

        Err(last_err.unwrap_or_else(|| LauyerError::Session {
            message: "Retry loop exhausted with no error recorded".to_owned(),
        }))
    }
}

// ---------------------------------------------------------------------------
// Trait implementation
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl HttpFetcher for HttpClient {
    async fn get(&self, url: &str) -> Result<Vec<u8>> {
        let owned_url = url.to_owned();
        let resp = self.execute_with_retry(url, || self.client.get(&owned_url)).await?;
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| LauyerError::Http { url: url.to_owned(), source: e })
    }

    async fn get_text(&self, url: &str) -> Result<String> {
        let owned_url = url.to_owned();
        let resp = self.execute_with_retry(url, || self.client.get(&owned_url)).await?;
        resp.text().await.map_err(|e| LauyerError::Http { url: url.to_owned(), source: e })
    }

    async fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
        headers: &[(String, String)],
    ) -> Result<String> {
        let owned_url = url.to_owned();
        let owned_body = body.clone();
        let owned_headers: Vec<(String, String)> = headers.to_vec();

        let resp = self
            .execute_with_retry(url, || {
                let mut req = self.client.post(&owned_url).json(&owned_body);
                for (k, v) in &owned_headers {
                    req = req.header(k.as_str(), v.as_str());
                }
                req
            })
            .await?;

        resp.text().await.map_err(|e| LauyerError::Http { url: url.to_owned(), source: e })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::SERVICE_UNAVAILABLE
            | reqwest::StatusCode::TOO_MANY_REQUESTS
            | reqwest::StatusCode::GATEWAY_TIMEOUT
            | reqwest::StatusCode::BAD_GATEWAY
    )
}

fn is_retryable_error(e: &reqwest::Error) -> bool {
    e.is_timeout() || e.is_connect()
}
