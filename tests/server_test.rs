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
