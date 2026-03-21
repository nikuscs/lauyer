use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt as _;

use lawyerr::config::Config;
use lawyerr::http::HttpClient;
use lawyerr::server;

fn test_router() -> axum::Router {
    let config = Config::default();
    let http_client = HttpClient::new(None, 30, 3).expect("failed to build http client");
    server::router(config, http_client)
}

#[tokio::test]
async fn health_returns_ok() {
    let app = test_router();

    let response =
        app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], "0.1.0");
}

#[tokio::test]
async fn dgsi_courts_returns_json_array() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dgsi/courts").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array(), "Expected JSON array, got: {json}");

    let arr = json.as_array().unwrap();
    assert!(!arr.is_empty(), "Courts list should not be empty");

    // Each entry should have alias and name
    let first = &arr[0];
    assert!(first.get("alias").is_some(), "Missing alias field");
    assert!(first.get("name").is_some(), "Missing name field");
}

#[tokio::test]
async fn dr_search_returns_501() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/search").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "DR module not implemented yet");
}

#[tokio::test]
async fn dr_today_returns_501() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/today").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn dr_types_returns_501() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/types").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn dr_fetch_returns_501() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/fetch").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn dgsi_search_missing_q_returns_error() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dgsi/search").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Missing required `q` param should fail (400 from axum query extractor)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn dgsi_search_with_court_param() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dgsi/search?q=test&court=stj").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    // The upstream HTTP call to DGSI will fail (no real server), so we expect
    // a 200 with an empty result set — all courts that error are skipped via
    // the warn-and-continue path in dgsi_search.
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn dgsi_fetch_missing_url() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dgsi/fetch").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Missing required `url` param — axum returns 400.
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn dgsi_fetch_with_url() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dgsi/fetch?url=https%3A%2F%2Fwww.dgsi.pt%2Fjstj.nsf%2Ftest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Upstream is unreachable in tests → 502 BAD_GATEWAY.
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn health_response_structure() {
    let app = test_router();

    let response =
        app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("status").is_some(), "health JSON must have 'status' key");
    assert!(json.get("version").is_some(), "health JSON must have 'version' key");
}

#[tokio::test]
async fn dgsi_courts_response_structure() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dgsi/courts").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let arr = json.as_array().expect("response must be a JSON array");
    assert!(!arr.is_empty(), "courts array must not be empty");

    for entry in arr {
        let alias = entry.get("alias").and_then(|v| v.as_str());
        let name = entry.get("name").and_then(|v| v.as_str());
        assert!(alias.is_some(), "each court must have an 'alias' field");
        assert!(name.is_some(), "each court must have a 'name' field");
        assert!(!alias.unwrap().is_empty(), "alias must not be empty");
        assert!(!name.unwrap().is_empty(), "name must not be empty");
    }
}

// ---------------------------------------------------------------------------
// dgsi_search with date params (since/until) — exercises date parsing lines
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_search_with_dates() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dgsi/search?q=test&since=2024-01-01&until=2025-01-01")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Upstream DGSI unreachable → courts fail and are skipped → 200 with empty results
    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// dgsi_search with invalid date — exercises date parse error branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_search_with_invalid_date() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dgsi/search?q=test&since=not-a-date")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Config error → INTERNAL_SERVER_ERROR
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json["error"].as_str().is_some_and(|e| e.contains("Invalid since date")),
        "error should mention invalid since date, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// dgsi_search with markdown format — exercises parse_output_format and
// format_response for the Markdown branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_search_markdown_format() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dgsi/search?q=test&format=md").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("text/markdown"),
        "Expected text/markdown content-type, got: {content_type}"
    );
}

// ---------------------------------------------------------------------------
// dgsi_search with json format — explicit format=json, verifies JSON content-type
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_search_json_format() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dgsi/search?q=test&format=json").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("application/json"),
        "Expected application/json content-type, got: {content_type}"
    );
}

// ---------------------------------------------------------------------------
// dgsi_search with limit and sort params
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_search_with_limit_and_sort() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dgsi/search?q=test&limit=10&sort=date")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Upstream unreachable → 200 with empty results
    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// dgsi_fetch with markdown format — exercises parse_output_format path in
// dgsi_fetch handler; upstream unreachable returns 502
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_fetch_with_format_md() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dgsi/fetch?url=http%3A%2F%2Fexample.com%2Fdoc&format=md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Upstream unreachable → 502 BAD_GATEWAY
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

// ---------------------------------------------------------------------------
// dgsi_search with markdown alias "markdown"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_search_markdown_alias() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dgsi/search?q=test&format=markdown")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("text/markdown"),
        "Expected text/markdown for format=markdown, got: {content_type}"
    );
}
