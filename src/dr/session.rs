use serde_json::Value;
use tracing::info;

use crate::error::{LauyerError, Result};
use crate::http::{HttpClient, HttpFetcher};

/// Hardcoded anonymous CSRF token from `OutSystems.js`.
const CSRF_TOKEN: &str = "T6C+9iB49TLra4jEsMeSckDMNhQ=";

/// API version hash for `DataActionGetPesquisas`.
const API_VERSION: &str = "6Bnghy+TVcnOZSN2FpzXbQ";

const BASE_URL: &str = "https://diariodarepublica.pt";
const VERSION_INFO_URL: &str = "https://diariodarepublica.pt/dr/moduleservices/moduleversioninfo";
const ROLES_URL: &str = "https://diariodarepublica.pt/dr/moduleservices/roles";

/// Embedded body template (stripped of `_comment` keys at init time).
const TEMPLATE_JSON: &str = include_str!("request_template.json");

/// A DR session holds the HTTP client (with cookie jar), version tokens,
/// and a parsed body template that is cloned and modified per search.
pub struct DrSession {
    client: HttpClient,
    module_version: String,
    api_version: String,
    body_template: Value,
}

impl DrSession {
    /// Initialise a new DR session.
    ///
    /// 1. GET `moduleversioninfo` to obtain the current `versionToken`.
    /// 2. GET `roles` with `X-CSRFToken` header to set session cookies.
    /// 3. Parse and clean the embedded body template.
    pub async fn new(client: HttpClient) -> Result<Self> {
        Self::new_from_urls(client, VERSION_INFO_URL, ROLES_URL).await
    }

    /// Initialise a DR session using explicit endpoint URLs.
    ///
    /// Identical to [`new`] but lets callers override the version-info and
    /// roles URLs — primarily useful in tests where a mock server replaces
    /// the real DR endpoints.
    pub async fn new_from_urls(
        client: HttpClient,
        version_info_url: &str,
        roles_url: &str,
    ) -> Result<Self> {
        // Step 1: fetch module version
        let version_text = client.get_text(version_info_url).await.map_err(|e| {
            tracing::warn!(error = %e, "Failed to fetch DR module version info");
            e
        })?;

        let version_json: Value =
            serde_json::from_str(&version_text).map_err(|e| LauyerError::Parse {
                message: format!("Failed to parse moduleversioninfo JSON: {e}"),
                source_url: version_info_url.to_owned(),
            })?;

        let module_version = version_json
            .get("versionToken")
            .and_then(Value::as_str)
            .ok_or_else(|| LauyerError::Parse {
                message: "Missing versionToken in moduleversioninfo response".to_owned(),
                source_url: version_info_url.to_owned(),
            })?
            .to_owned();

        info!(module_version = %module_version, "DR module version obtained");

        // Step 2: call roles endpoint to set session cookies on the jar
        let _roles_response = client
            .inner()
            .get(roles_url)
            .header("X-CSRFToken", CSRF_TOKEN)
            .send()
            .await
            .map_err(|e| LauyerError::Http { source: e, url: roles_url.to_owned() })?;

        info!("DR session cookies set via roles endpoint");

        // Step 3: parse and clean the embedded template
        let mut template: Value =
            serde_json::from_str(TEMPLATE_JSON).map_err(|e| LauyerError::Parse {
                message: format!("Failed to parse embedded DR body template: {e}"),
                source_url: "embedded:dr_request_template.json".to_owned(),
            })?;
        strip_comment_keys(&mut template);

        Ok(Self {
            client,
            module_version,
            api_version: API_VERSION.to_owned(),
            body_template: template,
        })
    }

    pub const fn client(&self) -> &HttpClient {
        &self.client
    }

    pub fn module_version(&self) -> &str {
        &self.module_version
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    pub const fn body_template(&self) -> &Value {
        &self.body_template
    }

    /// The hardcoded CSRF token used for all anonymous requests.
    pub const fn csrf_token(&self) -> &str {
        CSRF_TOKEN
    }

    /// The base URL for cookie domain operations.
    pub const fn base_url() -> &'static str {
        BASE_URL
    }

    /// Re-fetch module version and re-call the roles endpoint to refresh the
    /// session. Uses the same logic as [`new_from_urls`] steps 1-2, without
    /// re-parsing the embedded body template.
    pub async fn refresh(&mut self) -> Result<()> {
        self.refresh_from_urls(VERSION_INFO_URL, ROLES_URL).await
    }

    /// Refresh the session using explicit endpoint URLs.
    ///
    /// Identical to [`refresh`] but lets callers override the version-info and
    /// roles URLs — primarily useful in tests where a mock server replaces
    /// the real DR endpoints.
    pub async fn refresh_from_urls(
        &mut self,
        version_info_url: &str,
        roles_url: &str,
    ) -> Result<()> {
        // Step 1: re-fetch module version
        let version_text = self.client.get_text(version_info_url).await.map_err(|e| {
            tracing::warn!(error = %e, "Failed to fetch DR module version info during refresh");
            e
        })?;

        let version_json: Value =
            serde_json::from_str(&version_text).map_err(|e| LauyerError::Parse {
                message: format!("Failed to parse moduleversioninfo JSON: {e}"),
                source_url: version_info_url.to_owned(),
            })?;

        let module_version = version_json
            .get("versionToken")
            .and_then(Value::as_str)
            .ok_or_else(|| LauyerError::Parse {
                message: "Missing versionToken in moduleversioninfo response".to_owned(),
                source_url: version_info_url.to_owned(),
            })?
            .to_owned();

        info!(module_version = %module_version, "DR module version refreshed");

        // Step 2: re-call roles endpoint to refresh session cookies
        let _roles_response = self
            .client
            .inner()
            .get(roles_url)
            .header("X-CSRFToken", CSRF_TOKEN)
            .send()
            .await
            .map_err(|e| LauyerError::Http { source: e, url: roles_url.to_owned() })?;

        info!("DR session cookies refreshed via roles endpoint");

        self.module_version = module_version;

        Ok(())
    }
}

/// Recursively remove all keys named `_comment` from a JSON value.
pub fn strip_comment_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("_comment");
            for v in map.values_mut() {
                strip_comment_keys(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                strip_comment_keys(v);
            }
        }
        _ => {}
    }
}
