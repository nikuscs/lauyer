use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::get;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::LauyerError;
use crate::format::{OutputFormat, Renderable, SearchResponse};
use crate::http::HttpClient;
use crate::{dgsi, dr, format};

const DEFAULT_SEARCH_LIMIT: u32 = 50;

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

pub struct AppError(LauyerError);

impl From<LauyerError> for AppError {
    fn from(err: LauyerError) -> Self {
        Self(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            LauyerError::Http { .. } => StatusCode::BAD_GATEWAY,
            LauyerError::Session { .. } => StatusCode::SERVICE_UNAVAILABLE,
            LauyerError::UserInput { .. } => StatusCode::BAD_REQUEST,
            LauyerError::Parse { .. }
            | LauyerError::Encoding { .. }
            | LauyerError::Config { .. }
            | LauyerError::Io { .. } => StatusCode::INTERNAL_SERVER_ERROR,
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

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: "0.1.0" })
}

#[derive(Deserialize)]
pub struct CourtsParams {
    pub format: Option<String>,
}

async fn dgsi_courts(Query(params): Query<CourtsParams>) -> Response {
    let courts = dgsi::list_courts();
    let fmt = parse_output_format(params.format.as_deref());

    if fmt == OutputFormat::Json {
        let infos: Vec<CourtInfo> =
            courts.into_iter().map(|(alias, name)| CourtInfo { alias, name }).collect();
        Json(infos).into_response()
    } else {
        let mut out = String::from("| Alias | Court |\n|---|---|\n");
        for (alias, name) in &courts {
            let _ = std::fmt::Write::write_fmt(&mut out, format_args!("| `{alias}` | {name} |\n"));
        }
        (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")], out)
            .into_response()
    }
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
            s.parse::<chrono::NaiveDate>().map_err(|_| LauyerError::UserInput {
                message: format!("Invalid since date: '{s}'"),
            })
        })
        .transpose()?;

    let until = params
        .until
        .as_deref()
        .map(|s| {
            s.parse::<chrono::NaiveDate>().map_err(|_| LauyerError::UserInput {
                message: format!("Invalid until date: '{s}'"),
            })
        })
        .transpose()?;

    let query = dgsi::build_query(&params.q, since, until, None);
    let limit = params.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
    let sort_by_date = params.sort.as_deref() == Some("date");
    let fetch_full = params.fetch_full.unwrap_or(false);
    let compact = params.compact.unwrap_or(state.config.output.compact);
    let fmt = parse_output_format(params.format.as_deref());
    let max_concurrent = state.config.http.max_concurrent.max(1);

    let court_results = dgsi::search_all_courts(
        &state.http_client,
        &courts,
        &query,
        limit,
        sort_by_date,
        max_concurrent,
        None,
    )
    .await;

    let mut all_renderables: Vec<Box<dyn Renderable>> = Vec::new();
    let mut total: u64 = 0;
    let mut source_parts: Vec<String> = Vec::new();
    let mut error_count: usize = 0;
    let court_count = court_results.len();

    for (court, result) in court_results {
        match result {
            Err(e) => {
                error_count += 1;
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

    // If all courts failed, return a 502 error instead of empty 200
    if all_renderables.is_empty() && error_count > 0 && error_count == court_count {
        let body = serde_json::json!({
            "error": format!("All {court_count} court(s) failed to respond")
        });
        return Ok((StatusCode::BAD_GATEWAY, Json(body)).into_response());
    }

    let source = if source_parts.is_empty() { "DGSI".to_owned() } else { source_parts.join(", ") };

    let strip_sw = state.config.output.strip_stopwords;
    let response = SearchResponse { source, query, total, results: all_renderables };
    let rendered = format::render(&response, &fmt, compact, strip_sw);

    Ok(format_response(&rendered, &fmt))
}

async fn dgsi_fetch(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DgsiFetchParams>,
) -> Result<Response, AppError> {
    let compact = params.compact.unwrap_or(state.config.output.compact);
    let fmt = parse_output_format(params.format.as_deref());

    let strip_sw = state.config.output.strip_stopwords;
    let decision = dgsi::fetch_full_decision(&state.http_client, &params.url).await?;
    let response = SearchResponse {
        source: "DGSI".to_owned(),
        query: params.url,
        total: 1,
        results: vec![Box::new(decision)],
    };
    let rendered = format::render(&response, &fmt, compact, strip_sw);

    Ok(format_response(&rendered, &fmt))
}

// ---------------------------------------------------------------------------
// DR query params
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct DrSearchQueryParams {
    pub q: Option<String>,
    #[serde(rename = "type")]
    pub act_type: Option<String>,
    pub content: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub limit: Option<u32>,
    pub format: Option<String>,
    pub compact: Option<bool>,
}

#[derive(Deserialize)]
pub struct DrTodayQueryParams {
    #[serde(rename = "type")]
    pub act_type: Option<String>,
    pub format: Option<String>,
    pub compact: Option<bool>,
}

#[derive(Deserialize)]
pub struct DrTypesQueryParams {
    pub format: Option<String>,
}

// ---------------------------------------------------------------------------
// DR handlers
// ---------------------------------------------------------------------------

async fn dr_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DrSearchQueryParams>,
) -> Result<Response, AppError> {
    // Fresh client per request: DR sessions set CSRF + search cookies on the jar,
    // so sharing AppState.http_client would cause cross-request cookie contamination.
    let client = HttpClient::new(
        state.config.http.proxy.as_deref(),
        state.config.http.timeout_secs,
        state.config.http.retries,
    )
    .map_err(AppError)?;
    let session = dr::DrSession::new(client).await.map_err(AppError)?;

    // Resolve content types
    let content_aliases: Vec<String> = params.content.as_deref().map_or_else(
        || vec!["atos-1".to_owned()],
        |c| c.split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect(),
    );

    let content_types = dr::resolve_content_types(&content_aliases).map_err(AppError)?;

    // Resolve act types
    let mut act_types = Vec::new();
    if let Some(ref type_str) = params.act_type {
        for alias in type_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let resolved = dr::resolve_act_type(alias).ok_or_else(|| {
                AppError(LauyerError::UserInput {
                    message: format!("Unknown act type alias: '{alias}'"),
                })
            })?;
            act_types.push(resolved);
        }
    }

    // Parse dates
    let since = params
        .since
        .as_deref()
        .map(|s| {
            s.parse::<chrono::NaiveDate>().map_err(|_| LauyerError::UserInput {
                message: format!("Invalid since date: '{s}'"),
            })
        })
        .transpose()?;

    let until = params
        .until
        .as_deref()
        .map(|s| {
            s.parse::<chrono::NaiveDate>().map_err(|_| LauyerError::UserInput {
                message: format!("Invalid until date: '{s}'"),
            })
        })
        .transpose()?;

    let search_params = dr::DrSearchParams {
        content_types,
        query: params.q.unwrap_or_default(),
        act_types,
        series: vec![],
        since,
        until,
        limit: params.limit.unwrap_or(DEFAULT_SEARCH_LIMIT),
    };

    let response = dr::search(&session, &search_params).await.map_err(AppError)?;
    let limit = search_params.limit;
    let response = dr::apply_limit(response, limit);

    let compact = params.compact.unwrap_or(state.config.output.compact);
    let fmt = parse_output_format(params.format.as_deref());

    let renderables: Vec<Box<dyn Renderable>> =
        response.results.into_iter().map(|r| Box::new(r) as Box<dyn Renderable>).collect();

    let strip_sw = state.config.output.strip_stopwords;
    let search_response = SearchResponse {
        source: "Diário da República".to_owned(),
        query: search_params.query,
        total: response.total,
        results: renderables,
    };
    let rendered = format::render(&search_response, &fmt, compact, strip_sw);

    Ok(format_response(&rendered, &fmt))
}

async fn dr_today(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DrTodayQueryParams>,
) -> Result<Response, AppError> {
    let client = HttpClient::new(
        state.config.http.proxy.as_deref(),
        state.config.http.timeout_secs,
        state.config.http.retries,
    )
    .map_err(AppError)?;
    let session = dr::DrSession::new(client).await.map_err(AppError)?;

    let content_types = dr::resolve_content_types(&[String::from("atos-1")]).map_err(AppError)?;

    let mut act_types = Vec::new();
    if let Some(ref type_str) = params.act_type {
        for alias in type_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let resolved = dr::resolve_act_type(alias).ok_or_else(|| {
                AppError(LauyerError::UserInput {
                    message: format!("Unknown act type alias: '{alias}'"),
                })
            })?;
            act_types.push(resolved);
        }
    }

    let today = chrono::Local::now().date_naive();
    let search_params = dr::DrSearchParams {
        content_types,
        query: String::new(),
        act_types,
        series: vec![],
        since: Some(today),
        until: Some(today),
        limit: DEFAULT_SEARCH_LIMIT,
    };

    let response = dr::search(&session, &search_params).await.map_err(AppError)?;

    let compact = params.compact.unwrap_or(state.config.output.compact);
    let fmt = parse_output_format(params.format.as_deref());

    let renderables: Vec<Box<dyn Renderable>> =
        response.results.into_iter().map(|r| Box::new(r) as Box<dyn Renderable>).collect();

    let strip_sw = state.config.output.strip_stopwords;
    let search_response = SearchResponse {
        source: "Diário da República — Today".to_owned(),
        query: String::new(),
        total: response.total,
        results: renderables,
    };
    let rendered = format::render(&search_response, &fmt, compact, strip_sw);

    Ok(format_response(&rendered, &fmt))
}

async fn dr_types(Query(params): Query<DrTypesQueryParams>) -> Response {
    let types = dr::list_act_types();
    let fmt = parse_output_format(params.format.as_deref());

    if fmt == OutputFormat::Json {
        let items: Vec<serde_json::Value> = types
            .into_iter()
            .map(|(alias, name)| serde_json::json!({"alias": alias, "name": name}))
            .collect();
        Json(items).into_response()
    } else {
        let mut out = String::from("| Alias | Act Type |\n|---|---|\n");
        for (alias, name) in &types {
            let _ = std::fmt::Write::write_fmt(&mut out, format_args!("| `{alias}` | {name} |\n"));
        }
        (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "text/markdown; charset=utf-8")], out)
            .into_response()
    }
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
        .route("/dr/search", get(dr_search))
        .route("/dr/today", get(dr_today))
        .route("/dr/types", get(dr_types))
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
        .map_err(|e| LauyerError::Io { source: e })?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {},
            _ = sigterm.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
    }
    tracing::info!("Shutdown signal received, stopping server");
}
