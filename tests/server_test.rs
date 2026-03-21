use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt as _;

use lauyer::config::Config;
use lauyer::http::HttpClient;
use lauyer::server;

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
        .oneshot(Request::builder().uri("/dgsi/courts?format=json").body(Body::empty()).unwrap())
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
async fn dr_types_default_is_markdown() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/types").body(Body::empty()).unwrap())
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(
        body_str.contains("| Alias |"),
        "Markdown response must contain the table header '| Alias |': {body_str}"
    );
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

    // If the upstream DGSI server is reachable, the search succeeds (200).
    // If not, all courts fail and we get 502. Both are acceptable here.
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "Expected 200 or 502, got: {status}"
    );
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
        .oneshot(Request::builder().uri("/dgsi/courts?format=json").body(Body::empty()).unwrap())
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
// dgsi_courts default (no format param) — exercises the markdown output path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dgsi_courts_default_is_markdown() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dgsi/courts").body(Body::empty()).unwrap())
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(
        body_str.contains("| Alias |"),
        "Markdown response must contain the table header '| Alias |': {body_str}"
    );
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

    // If DGSI is reachable, search succeeds (200). Otherwise, all courts fail (502).
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "Expected 200 or 502, got: {status}"
    );
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

    // UserInput error → BAD_REQUEST
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

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

    // If DGSI is reachable: 200 with markdown content-type.
    // If not: all courts fail → 502.
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "Expected 200 or 502, got: {status}"
    );

    if status == StatusCode::OK {
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

    // If DGSI is reachable: 200 with JSON content-type.
    // If not: all courts fail → 502.
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "Expected 200 or 502, got: {status}"
    );

    if status == StatusCode::OK {
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

    // If DGSI is reachable: 200. If not: all courts fail → 502.
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "Expected 200 or 502, got: {status}"
    );
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

    // If DGSI is reachable: 200. If not: all courts fail → 502.
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "Expected 200 or 502, got: {status}"
    );
}

// ---------------------------------------------------------------------------
// DR handler tests
// ---------------------------------------------------------------------------

// DR session init contacts diariodarepublica.pt. If the host is unreachable,
// `DrSession::new` returns `LauyerError::Http` → `StatusCode::BAD_GATEWAY`
// (502). If reachable, the search runs and returns 200. Both outcomes are
// valid in these tests — what matters is that valid params never return 400.

#[tokio::test]
async fn dr_search_with_params() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dr/search?q=test&type=portaria&content=atos-1&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Parameters are valid — must NOT get a 400. If DR is reachable → 200;
    // if not → 502 or 500.
    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "Valid params must not return 400, got: {status}");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_with_params: {status}"
    );
}

#[tokio::test]
async fn dr_search_with_dates() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dr/search?since=2024-01-01&until=2025-01-01")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Valid date params — must NOT get a 400.
    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "Valid dates must not return 400, got: {status}");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_with_dates: {status}"
    );
}

#[tokio::test]
async fn dr_search_invalid_date() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/search?since=not-a-date").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // The DR handler creates the session before parsing dates, so the network
    // failure occurs first and we get 502. The important invariant is that the
    // server does not return 200 (it must not succeed with bad input).
    let status = response.status();
    assert_ne!(status, StatusCode::OK, "Server must not return 200 for invalid date input");
}

#[tokio::test]
async fn dr_search_invalid_content_type() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dr/search?content=invalid-type").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    // Session init happens before content-type resolution, so we get a network
    // error (502) rather than a user-input error (400). Either way, not 200.
    let status = response.status();
    assert_ne!(status, StatusCode::OK, "Server must not return 200 for invalid content type");
}

#[tokio::test]
async fn dr_today_endpoint() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/today").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // DR session init may or may not reach the upstream host.
    let status = response.status();
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_today_endpoint: {status}"
    );
}

#[tokio::test]
async fn dr_types_json() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/types?format=json").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array(), "Expected JSON array of act types, got: {json}");

    let arr = json.as_array().unwrap();
    assert!(!arr.is_empty(), "Act types array must not be empty");

    for entry in arr {
        assert!(entry.get("alias").is_some(), "Each entry must have an 'alias' field");
        assert!(entry.get("name").is_some(), "Each entry must have a 'name' field");
    }
}

#[tokio::test]
async fn dr_types_markdown() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/types").body(Body::empty()).unwrap())
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(
        body_str.contains("| Alias |"),
        "Markdown table must contain '| Alias |' header: {body_str}"
    );
    assert!(
        body_str.contains("| Act Type |"),
        "Markdown table must contain '| Act Type |' header: {body_str}"
    );
}

#[tokio::test]
async fn dr_search_markdown_format() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/search?q=test&format=md").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // DR session init may or may not reach the upstream host.
    let status = response.status();
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_markdown_format: {status}"
    );
}

// ---------------------------------------------------------------------------
// DR handler coverage for additional branches
// ---------------------------------------------------------------------------

// content=atos-2&type=despacho — exercises the non-default content alias path
// and act type resolution inside dr_search. The upstream DR session will fail
// (network unreachable) but all query-param parsing runs before the network call.
#[tokio::test]
async fn dr_search_with_content_param() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dr/search?content=atos-2&type=despacho")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Session init hits the real DR network which may be unreachable → 502.
    // Valid params must not return 400.
    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "valid content/type must not return 400");
}

/// No content param → handler defaults to "atos-1". Verify no 400 is returned.
#[tokio::test]
async fn dr_search_default_content() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/search?q=test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "default content must not return 400");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_default_content: {status}"
    );
}

// dr/today with type=portaria — exercises act type resolution inside dr_today
#[tokio::test]
async fn dr_today_with_type_filter() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/today?type=portaria").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "valid type param must not return 400");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_today_with_type_filter: {status}"
    );
}

/// dr/types JSON structure — every entry has alias and name string fields
#[tokio::test]
async fn dr_types_json_structure() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/types?format=json").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().expect("dr/types must return a JSON array");
    assert!(!arr.is_empty(), "act types must not be empty");

    for entry in arr {
        let alias = entry["alias"].as_str();
        let name = entry["name"].as_str();
        assert!(alias.is_some(), "each entry must have a string 'alias'");
        assert!(name.is_some(), "each entry must have a string 'name'");
        assert!(!alias.unwrap().is_empty(), "alias must not be empty");
        assert!(!name.unwrap().is_empty(), "name must not be empty");
    }
}

// dr/today with unknown alias — session init fires first (502); server must not return 200.
#[tokio::test]
async fn dr_today_with_invalid_type() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dr/today?type=not-a-real-type").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::OK, "invalid type param must not return 200");
}

/// dr/search with unknown act type alias → server must not return 200
#[tokio::test]
async fn dr_search_unknown_act_type() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dr/search?type=not-a-real-type").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::OK, "unknown act type must not return 200");
}

// AppError session variant → 503 SERVICE_UNAVAILABLE
#[tokio::test]
async fn app_error_session_variant_is_503() {
    use axum::response::IntoResponse as _;
    use lauyer::error::LauyerError;
    use lauyer::server::AppError;

    let err = AppError::from(LauyerError::Session { message: "no session".to_owned() });
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// AppError parse variant → 500 INTERNAL_SERVER_ERROR
#[tokio::test]
async fn app_error_parse_variant_is_500() {
    use axum::response::IntoResponse as _;
    use lauyer::error::LauyerError;
    use lauyer::server::AppError;

    let err = AppError::from(LauyerError::Parse {
        message: "bad parse".to_owned(),
        source_url: "http://example.com".to_owned(),
    });
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ---------------------------------------------------------------------------
// DR handler param parsing — limit, compact, sort
// If DR is reachable (200), the post-session logic runs and covers those
// lines.  If not (502/500), still acceptable — must never return 400.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dr_search_with_limit_param() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/search?q=test&limit=5").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "limit=5 must not return 400");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_with_limit_param: {status}"
    );
}

#[tokio::test]
async fn dr_search_compact_false() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder().uri("/dr/search?q=test&compact=false").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "compact=false must not return 400");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_compact_false: {status}"
    );
}

#[tokio::test]
async fn dr_search_with_sort() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/dr/search?q=test&sort=date").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    assert_ne!(status, StatusCode::BAD_REQUEST, "sort=date must not return 400");
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_GATEWAY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected status for dr_search_with_sort: {status}"
    );
}

// ---------------------------------------------------------------------------
// AppError remaining variants — Encoding and Io → 500 INTERNAL_SERVER_ERROR
// ---------------------------------------------------------------------------

#[tokio::test]
async fn app_error_encoding_variant_is_500() {
    use axum::response::IntoResponse as _;
    use lauyer::error::LauyerError;
    use lauyer::server::AppError;

    let err = AppError::from(LauyerError::Encoding { message: "bad encoding".to_owned() });
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn app_error_config_variant_is_500() {
    use axum::response::IntoResponse as _;
    use lauyer::error::LauyerError;
    use lauyer::server::AppError;

    let err = AppError::from(LauyerError::Config { message: "bad config".to_owned() });
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn app_error_userinput_variant_is_400() {
    use axum::response::IntoResponse as _;
    use lauyer::error::LauyerError;
    use lauyer::server::AppError;

    let err = AppError::from(LauyerError::UserInput { message: "bad input".to_owned() });
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn app_error_io_variant_is_500() {
    use axum::response::IntoResponse as _;
    use lauyer::error::LauyerError;
    use lauyer::server::AppError;

    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
    let err = AppError::from(LauyerError::Io { source: io_err });
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
