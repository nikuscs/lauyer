use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::LawyerrError;
use crate::format::{OutputFormat, Renderable, SearchResponse};
use crate::http::HttpClient;
use crate::{dgsi, format};

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

pub struct AppState {
    pub config: Config,
    pub http_client: HttpClient,
}

// ---------------------------------------------------------------------------
// Error wrapper
// ---------------------------------------------------------------------------

pub struct AppError(LawyerrError);

impl From<LawyerrError> for AppError {
    fn from(err: LawyerrError) -> Self {
        Self(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            LawyerrError::Http { .. } => StatusCode::BAD_GATEWAY,
            LawyerrError::Session { .. } => StatusCode::SERVICE_UNAVAILABLE,
            LawyerrError::Parse { .. }
            | LawyerrError::Encoding { .. }
            | LawyerrError::Config { .. }
            | LawyerrError::Io { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({ "error": self.0.to_string() });
        (status, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Query param structs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DgsiSearchParams {
    pub q: String,
    pub court: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub limit: Option<u32>,
    pub sort: Option<String>,
    pub format: Option<String>,
    pub compact: Option<bool>,
    pub fetch_full: Option<bool>,
}

#[derive(Deserialize)]
pub struct DgsiFetchParams {
    pub url: String,
    pub format: Option<String>,
    pub compact: Option<bool>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct CourtInfo {
    alias: String,
    name: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: "0.1.0" })
}

async fn dgsi_courts() -> Json<Vec<CourtInfo>> {
    let courts = dgsi::list_courts();
    let infos: Vec<CourtInfo> =
        courts.into_iter().map(|(alias, name)| CourtInfo { alias, name }).collect();
    Json(infos)
}

async fn dgsi_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DgsiSearchParams>,
) -> Result<Response, AppError> {
    let court_aliases: Vec<String> = params
        .court
        .as_deref()
        .map(|c| c.split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    let courts = dgsi::resolve_courts(&court_aliases)?;

    let since = params
        .since
        .as_deref()
        .map(|s| {
            s.parse::<chrono::NaiveDate>()
                .map_err(|_| LawyerrError::Config { message: format!("Invalid since date: '{s}'") })
        })
        .transpose()?;

    let until = params
        .until
        .as_deref()
        .map(|s| {
            s.parse::<chrono::NaiveDate>()
                .map_err(|_| LawyerrError::Config { message: format!("Invalid until date: '{s}'") })
        })
        .transpose()?;

    let query = dgsi::build_query(&params.q, since, until, None);
    let limit = params.limit.unwrap_or(50);
    let sort_by_date = params.sort.as_deref() == Some("date");
    let fetch_full = params.fetch_full.unwrap_or(false);
    let compact = params.compact.unwrap_or(true);
    let fmt = parse_output_format(params.format.as_deref());

    let court_results =
        dgsi::search_all_courts(&state.http_client, &courts, &query, limit, sort_by_date, 3).await;

    let mut all_renderables: Vec<Box<dyn Renderable>> = Vec::new();
    let mut total: u64 = 0;
    let mut source_parts: Vec<String> = Vec::new();

    for (court, result) in court_results {
        match result {
            Err(e) => {
                tracing::warn!(court = court.alias(), error = %e, "Skipping court");
            }
            Ok((court_total, results)) => {
                total += court_total;
                source_parts.push(court.display_name().to_owned());

                if fetch_full && !results.is_empty() {
                    for r in &results {
                        match dgsi::fetch_full_decision(&state.http_client, &r.doc_url).await {
                            Ok(dec) => all_renderables.push(Box::new(dec)),
                            Err(e) => {
                                tracing::warn!(error = %e, "Failed to fetch decision");
                            }
                        }
                    }
                } else {
                    for r in results {
                        all_renderables.push(Box::new(r));
                    }
                }
            }
        }
    }

    let source = if source_parts.is_empty() { "DGSI".to_owned() } else { source_parts.join(", ") };

    let response = SearchResponse { source, query, total, results: all_renderables };
    let rendered = format::render(&response, &fmt, compact, false);

    Ok(format_response(&rendered, &fmt))
}

async fn dgsi_fetch(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DgsiFetchParams>,
) -> Result<Response, AppError> {
    let compact = params.compact.unwrap_or(true);
    let fmt = parse_output_format(params.format.as_deref());

    let decision = dgsi::fetch_full_decision(&state.http_client, &params.url).await?;
    let response = SearchResponse {
        source: "DGSI".to_owned(),
        query: params.url,
        total: 1,
        results: vec![Box::new(decision)],
    };
    let rendered = format::render(&response, &fmt, compact, false);

    Ok(format_response(&rendered, &fmt))
}

async fn dr_stub() -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorBody { error: "DR module not implemented yet".to_owned() }),
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_output_format(fmt: Option<&str>) -> OutputFormat {
    match fmt {
        Some("json") => OutputFormat::Json,
        Some("table") => OutputFormat::Table,
        _ => OutputFormat::Markdown,
    }
}

fn format_response(rendered: &str, fmt: &OutputFormat) -> Response {
    match fmt {
        OutputFormat::Json => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            rendered.to_owned(),
        )
            .into_response(),
        _ => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            rendered.to_owned(),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Router & server start
// ---------------------------------------------------------------------------

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/dgsi/search", get(dgsi_search))
        .route("/dgsi/fetch", get(dgsi_fetch))
        .route("/dgsi/courts", get(dgsi_courts))
        .route("/dr/search", get(dr_stub))
        .route("/dr/today", get(dr_stub))
        .route("/dr/types", get(dr_stub))
        .route("/dr/fetch", get(dr_stub))
        .with_state(state)
}

/// Build the router with the given state. Exposed for testing.
pub fn router(config: Config, http_client: HttpClient) -> Router {
    let state = Arc::new(AppState { config, http_client });
    build_router(state)
}

pub async fn start(
    host: &str,
    port: u16,
    config: Config,
    http_client: HttpClient,
) -> crate::error::Result<()> {
    let state = Arc::new(AppState { config, http_client });
    let app = build_router(state);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on http://{host}:{port}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| LawyerrError::Io { source: e })?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("Shutdown signal received, stopping server");
}
