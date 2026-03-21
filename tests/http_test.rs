use lauyer::http::{HttpClient, HttpFetcher as _};
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// HttpClient::new
// ---------------------------------------------------------------------------

#[tokio::test]
async fn new_with_no_proxy_succeeds() {
    let client = HttpClient::new(None, 30, 3);
    assert!(client.is_ok(), "HttpClient::new with no proxy should succeed");
}

#[tokio::test]
async fn new_with_short_timeout_succeeds() {
    let client = HttpClient::new(None, 5, 0);
    assert!(client.is_ok(), "HttpClient::new with 5s timeout should succeed");
}

#[tokio::test]
async fn new_with_invalid_proxy_returns_error() {
    let result = HttpClient::new(Some("not a valid proxy url !!"), 30, 0);
    assert!(result.is_err(), "Invalid proxy URL should return an error");
}

// ---------------------------------------------------------------------------
// HttpFetcher::get — returns bytes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_returns_bytes() {
    let server = MockServer::start().await;
    let body = b"hello bytes";

    Mock::given(method("GET"))
        .and(path("/data"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.as_slice()))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/data", server.uri());
    let bytes = client.get(&url).await.unwrap();

    assert_eq!(bytes, body);
}

// ---------------------------------------------------------------------------
// HttpFetcher::get_text — returns decoded text
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_text_returns_string() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/text"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hello text"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/text", server.uri());
    let text = client.get_text(&url).await.unwrap();

    assert_eq!(text, "hello text");
}

// ---------------------------------------------------------------------------
// HttpFetcher::post_json — sends JSON body, returns response text
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_json_sends_body_and_returns_response() {
    let server = MockServer::start().await;
    let request_body = json!({"key": "value"});

    Mock::given(method("POST"))
        .and(path("/submit"))
        .and(body_json(&request_body))
        .respond_with(ResponseTemplate::new(200).set_body_string("accepted"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/submit", server.uri());
    let response = client.post_json(&url, &request_body, &[]).await.unwrap();

    assert_eq!(response, "accepted");
}

// ---------------------------------------------------------------------------
// HttpClient::get_latin1 — serves Latin-1 bytes, verifies UTF-8 output
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_latin1_decodes_windows1252_bytes() {
    let server = MockServer::start().await;

    // Windows-1252 / Latin-1: 0xe9 = é, 0xe0 = à, 0xf3 = ó
    let latin1_bytes: Vec<u8> = vec![b'c', b'a', b'f', 0xe9]; // "café" in Windows-1252

    Mock::given(method("GET"))
        .and(path("/latin1"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(latin1_bytes))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/latin1", server.uri());
    let text = client.get_latin1(&url).await.unwrap();

    assert_eq!(text, "café", "Windows-1252 0xe9 should decode to é");
}

// ---------------------------------------------------------------------------
// HttpClient::get_bytes — returns raw bytes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_bytes_returns_raw_bytes() {
    let server = MockServer::start().await;
    let raw: Vec<u8> = vec![0x00, 0xff, 0x80, 0x42];

    Mock::given(method("GET"))
        .and(path("/raw"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(raw.clone()))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/raw", server.uri());
    let bytes = client.get_bytes(&url).await.unwrap();

    assert_eq!(bytes, raw);
}

// ---------------------------------------------------------------------------
// Retry on 503 — first request returns 503, second returns 200
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_on_503_succeeds_on_second_attempt() {
    let server = MockServer::start().await;

    // First call: 503
    Mock::given(method("GET"))
        .and(path("/flaky"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Subsequent calls: 200
    Mock::given(method("GET"))
        .and(path("/flaky"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 3).unwrap();
    let url = format!("{}/flaky", server.uri());
    let result = client.get_text(&url).await;

    assert!(result.is_ok(), "Should succeed after retry: {result:?}");
    assert_eq!(result.unwrap(), "ok");
}

// ---------------------------------------------------------------------------
// No retry on 404 — returns error immediately
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_retry_on_404() {
    let server = MockServer::start().await;

    // Mount 404 with expectation of exactly 1 call — verifies no retry happened
    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 3).unwrap();
    let url = format!("{}/missing", server.uri());
    let result = client.get(&url).await;

    assert!(result.is_err(), "404 should return an error");
    // MockServer drop will panic if the mock was called != 1 times, verifying no retry
}

// ---------------------------------------------------------------------------
// Retry on 429 — retries and succeeds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_on_429_succeeds_after_retry() {
    let server = MockServer::start().await;

    // First call: 429 Too Many Requests
    Mock::given(method("GET"))
        .and(path("/limited"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Subsequent calls: 200
    Mock::given(method("GET"))
        .and(path("/limited"))
        .respond_with(ResponseTemplate::new(200).set_body_string("now ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 3).unwrap();
    let url = format!("{}/limited", server.uri());
    let result = client.get_text(&url).await;

    assert!(result.is_ok(), "Should succeed after 429 retry: {result:?}");
    assert_eq!(result.unwrap(), "now ok");
}

// ---------------------------------------------------------------------------
// Custom headers in post_json — verify headers arrive at mock server
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_json_custom_headers_are_sent() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api"))
        .and(header("x-api-key", "secret-token"))
        .and(header("x-request-id", "abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_string("authorised"))
        .expect(1)
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/api", server.uri());
    let headers = vec![
        ("x-api-key".to_owned(), "secret-token".to_owned()),
        ("x-request-id".to_owned(), "abc-123".to_owned()),
    ];
    let result = client.post_json(&url, &json!({}), &headers).await.unwrap();

    assert_eq!(result, "authorised");
}

// ---------------------------------------------------------------------------
// Cookie jar is shared — set a cookie, verify it's sent on the next request
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cookie_jar_is_shared_across_requests() {
    let server = MockServer::start().await;

    // First request sets a cookie via Set-Cookie header
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200).insert_header("set-cookie", "session=abc123; Path=/"),
        )
        .mount(&server)
        .await;

    // Second request expects the cookie to be present
    Mock::given(method("GET"))
        .and(path("/protected"))
        .and(header_exists("cookie"))
        .respond_with(ResponseTemplate::new(200).set_body_string("authenticated"))
        .expect(1)
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let base = server.uri();

    // Trigger the login to store the cookie
    client.get(&format!("{base}/login")).await.unwrap();

    // Now access the protected route — cookie jar should send the cookie automatically
    let result = client.get_text(&format!("{base}/protected")).await;
    assert!(result.is_ok(), "Protected route should succeed with cookie: {result:?}");
    assert_eq!(result.unwrap(), "authenticated");
}

// ---------------------------------------------------------------------------
// cookie_jar() accessor — returns the shared Arc<Jar>
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cookie_jar_accessor_returns_jar() {
    let client = HttpClient::new(None, 30, 0).unwrap();
    // Just verify we can call cookie_jar() without issues and that it returns an Arc
    let jar = client.cookie_jar();
    // The Arc should have a reference count of at least 2 (client + this clone)
    let _clone = std::sync::Arc::clone(jar);
}

// ---------------------------------------------------------------------------
// inner() accessor — exposes the underlying reqwest::Client
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inner_accessor_returns_client() {
    let client = HttpClient::new(None, 30, 0).unwrap();
    let _inner: &reqwest::Client = client.inner();
}

// ---------------------------------------------------------------------------
// Fatal error on 400 — not retried
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_retry_on_400_bad_request() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/validate"))
        .respond_with(ResponseTemplate::new(400))
        .expect(1)
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 3).unwrap();
    let url = format!("{}/validate", server.uri());
    let result = client.post_json(&url, &json!({"bad": "input"}), &[]).await;

    assert!(result.is_err(), "400 should return an error without retrying");
}

// ---------------------------------------------------------------------------
// Retry exhausted — all retries return 503, final result is an error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_exhausted_returns_error() {
    let server = MockServer::start().await;

    // Always return 503 — retries will all fail
    Mock::given(method("GET"))
        .and(path("/always-down"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    // retries=1: attempts 0 and 1 (2 total), both fail
    let client = HttpClient::new(None, 30, 1).unwrap();
    let url = format!("{}/always-down", server.uri());
    let result = client.get(&url).await;

    assert!(result.is_err(), "Should fail when all retries return 503");
}

// ---------------------------------------------------------------------------
// proxy_is_set_successfully — valid proxy URL succeeds at construction time
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proxy_is_set_successfully() {
    // A syntactically valid proxy URL — no connection is made at build time,
    // so construction should succeed even though nothing listens on this port.
    let result = HttpClient::new(Some("socks5://127.0.0.1:1080"), 30, 0);
    assert!(result.is_ok(), "Valid proxy URL should build successfully");
}

// ---------------------------------------------------------------------------
// retry_on_connection_error — dropped server → connection refused → Err branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retry_on_connection_error() {
    // Start a mock server so we get a real port, then drop it immediately.
    // The OS will refuse connections to that port, which triggers the Err(e)
    // branch in execute_with_retry with is_retryable_error returning false
    // (connection-refused is is_connect() == true) AND attempt >= self.retries
    // (retries=0), so we fall through to `return Err(...)` at lines 129-131.
    let port = {
        let server = MockServer::start().await;
        // server is dropped here, freeing the port
        server.uri()
    };

    let client = HttpClient::new(None, 5, 0).unwrap();
    let url = format!("{port}/any");
    let result = client.get(&url).await;

    assert!(result.is_err(), "Connection refused should return an error");
}

// ---------------------------------------------------------------------------
// get_text_on_502 — non-retryable error status (retries=0) returns error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_text_on_502_gateway() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/bad-gateway"))
        .respond_with(ResponseTemplate::new(502))
        .mount(&server)
        .await;

    // retries=0: 502 is retryable but attempt (0) is not < retries (0),
    // so execute_with_retry returns the error_for_status immediately.
    let client = HttpClient::new(None, 30, 0).unwrap();
    let url = format!("{}/bad-gateway", server.uri());
    let result = client.get_text(&url).await;

    assert!(result.is_err(), "502 with no retries should return an error");
}

// ---------------------------------------------------------------------------
// retry_connection_refused — server dropped → connection error Err branch
// with retries=1 so is_retryable_error fires for attempt 0, then falls through
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_retry_connection_refused() {
    // Acquire a free port by binding a mock server then immediately dropping it.
    let dead_uri = {
        let server = MockServer::start().await;
        server.uri()
    };
    // retries=1: attempt 0 fails with connection error (is_connect → retryable),
    // stores last_err, attempt 1 also fails (attempt==retries → not retryable),
    // returns Err immediately from the Err(e) branch at lines 128-131.
    let client = HttpClient::new(None, 5, 1).unwrap();
    let url = format!("{dead_uri}/any");
    let result = client.get(&url).await;
    assert!(result.is_err(), "connection refused with retry should ultimately return error");
}

// ---------------------------------------------------------------------------
// retry_exhausted_via_connection_error — all retries consume last_err path
// (lines 136-137: last_err.unwrap_or_else fallback never triggered here but
// the last_err Some branch IS reached when retries>0 and connection fails)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_retry_exhausted_via_503() {
    // 503 is retryable. With retries=2, three total attempts all return 503.
    // After the loop exhausts, last_err is Some → unwrap_or_else branch at
    // line 136 returns the stored error (not the fallback).
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/always-503"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 2).unwrap();
    let url = format!("{}/always-503", server.uri());
    let result = client.get_text(&url).await;
    assert!(result.is_err(), "exhausted retries on 503 must return error");
}
