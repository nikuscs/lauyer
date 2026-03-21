use base64::Engine as _;
use chrono::NaiveDate;
use reqwest::Url;
use serde_json::{Map, Value, json};
use tracing::{info, warn};

use crate::error::{LauyerError, Result};

use super::content_types::DrContentType;
use super::session::DrSession;

// ---------------------------------------------------------------------------
// Search parameters
// ---------------------------------------------------------------------------

/// Parameters for a DR search request.
pub struct DrSearchParams {
    pub content_types: Vec<DrContentType>,
    pub query: String,
    pub act_types: Vec<String>,
    pub series: Vec<String>,
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub limit: u32,
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// A single search result from the DR `ElasticSearch` response.
#[derive(Debug, Clone)]
pub struct DrSearchResult {
    pub title: String,
    pub tipo: String,
    pub numero: String,
    pub data_publicacao: Option<NaiveDate>,
    pub emissor: String,
    pub sumario: String,
    pub serie: String,
    pub db_id: String,
    pub file_id: String,
    pub tipo_conteudo: String,
    pub ano: Option<u32>,
}

/// Aggregate search response.
#[derive(Debug)]
pub struct DrSearchResponse {
    pub total: u64,
    pub results: Vec<DrSearchResult>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Serialize a `serde_json::Value` to a JSON string.
/// `Value` keys are always strings so `to_string` is infallible.
fn serialize_value(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Cookie builder
// ---------------------------------------------------------------------------

/// Build the `PesquisaAvancada` cookie value (URL-encoded wrapper JSON).
pub fn build_pesquisa_cookie(params: &DrSearchParams) -> String {
    // 1. Build cookie filtros JSON (compact, plain arrays)
    let filtros = build_cookie_filtros(params);
    let filtros_json = serialize_value(&filtros);

    // 2. Base64-encode
    let filtros_b64 = base64::engine::general_purpose::STANDARD.encode(filtros_json.as_bytes());

    // 3. Build bools JSON
    let bools = build_bools(params);
    let bools_json = serialize_value(&bools);

    // 4. Build sort fields JSON
    let sort_fields = build_sort_fields();
    let sort_json = serialize_value(&sort_fields);

    // 5. Build wrapper with JSON-as-string values (double-encoded)
    let wrapper = json!({
        "PesquisaAvancadaFiltros": filtros_b64,
        "PesquisaAvancadaBools": bools_json,
        "SortFields": sort_json,
    });
    let wrapper_json = serialize_value(&wrapper);

    // 6. URL-encode
    percent_encoding::utf8_percent_encode(&wrapper_json, percent_encoding::NON_ALPHANUMERIC)
        .to_string()
}

/// Build the filtros object for cookie encoding (plain arrays, compact JSON).
pub fn build_cookie_filtros(params: &DrSearchParams) -> Value {
    let tipo_conteudo: Vec<&str> =
        params.content_types.iter().map(DrContentType::tipo_conteudo).collect();

    let mut filtros = json!({
        "tipoConteudo": tipo_conteudo,
        "serie": params.series,
        "tipo": params.act_types,
        "emissor": [],
        "entidadeProponente": [],
        "entidadePrincipal": [],
        "entidadeEmitente": [],
        "DescritorList": [],
    });

    if let Some(since) = params.since {
        filtros["dataPublicacaoDe"] = Value::String(since.format("%Y-%m-%d").to_string());
    }
    if let Some(until) = params.until {
        filtros["dataPublicacaoAte"] = Value::String(until.format("%Y-%m-%d").to_string());
    }

    filtros
}

/// Build the `PesquisaAvancadaBools` object. All known content type keys are
/// present; only the selected ones are `true`.
pub fn build_bools(params: &DrSearchParams) -> Value {
    let mut bools = Map::new();
    // All known boolean keys in the expected order
    let all_keys = [
        "DiarioRepublica",
        "Atos1",
        "Atos2",
        "AcordaosSTA",
        "AtosSocietarios",
        "Legacor",
        "DGODOUT",
        "DGAP",
        "REGTRAB",
        "Jurisprudencia",
    ];

    let selected: Vec<&str> = params.content_types.iter().map(DrContentType::bools_key).collect();

    for key in &all_keys {
        bools.insert((*key).to_owned(), Value::Bool(selected.contains(key)));
    }

    Value::Object(bools)
}

/// Build the sort fields array used in both cookie and body.
fn build_sort_fields() -> Value {
    json!([
        {"Field": "dataPublicacao", "Order": "desc"},
        {"Field": "numeroDR.keyword", "Order": "desc"},
        {"Field": "serieNR", "Order": "asc"},
        {"Field": "suplemento", "Order": "asc"},
        {"Field": "apendice.keyword", "Order": "asc"},
    ])
}

// ---------------------------------------------------------------------------
// Body builder
// ---------------------------------------------------------------------------

/// Build the full ~30KB POST body by cloning the session template and setting
/// dynamic fields.
pub fn build_search_body(session: &DrSession, params: &DrSearchParams) -> Value {
    let mut body = session.body_template().clone();

    // versionInfo
    body["versionInfo"]["moduleVersion"] = Value::String(session.module_version().to_owned());
    body["versionInfo"]["apiVersion"] = Value::String(session.api_version().to_owned());

    // FiltrosDePesquisa (OutSystems format)
    let filtros = build_body_filtros(params);
    body["screenData"]["variables"]["FiltrosDePesquisa"] = filtros;

    // PesquisaAvancadaFiltros (base64 string)
    let cookie_filtros = build_cookie_filtros(params);
    let filtros_json = serialize_value(&cookie_filtros);
    let filtros_b64 = base64::engine::general_purpose::STANDARD.encode(filtros_json.as_bytes());
    body["screenData"]["variables"]["PesquisaAvancadaFiltros"] = Value::String(filtros_b64);

    // PesquisaAvancadaBools (JSON string)
    let bools = build_bools(params);
    let bools_json = serialize_value(&bools);
    body["screenData"]["variables"]["PesquisaAvancadaBools"] = Value::String(bools_json);

    // Date fields
    let since_str = params.since.map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());
    let until_str = params.until.map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());

    body["screenData"]["variables"]["FiltrosDePesquisa"]["dataPublicacaoDe"] =
        Value::String(since_str.clone());
    body["screenData"]["variables"]["FiltrosDePesquisa"]["dataPublicacaoAte"] =
        Value::String(until_str.clone());
    body["screenData"]["variables"]["DataDe"] = Value::String(since_str);
    body["screenData"]["variables"]["DataAte"] = Value::String(until_str);

    // Cookie-related body fields
    let cookie_value = build_pesquisa_cookie(params);
    body["screenData"]["variables"]["GetCookiePesquisas"]["Pesquisas"]["Avancada"] =
        Value::String(cookie_value);

    // Decoded URL pesquisa avancada
    let sort_fields = build_sort_fields();
    let sort_json = serialize_value(&sort_fields);
    let cookie_filtros2 = build_cookie_filtros(params);
    let filtros_json2 = serialize_value(&cookie_filtros2);
    let filtros_b64_2 = base64::engine::general_purpose::STANDARD.encode(filtros_json2.as_bytes());
    let bools2 = build_bools(params);
    let bools_json2 = serialize_value(&bools2);
    let decoded_wrapper = json!({
        "PesquisaAvancadaFiltros": filtros_b64_2,
        "PesquisaAvancadaBools": bools_json2,
        "SortFields": sort_json,
    });
    let decoded_json = serialize_value(&decoded_wrapper);
    body["screenData"]["variables"]["GetDecodeURLPesquisaAvancada"]["PesquisaAvancada_URL_Decoded"] =
        Value::String(decoded_json);

    // Client variables
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    body["clientVariables"]["Data"] = Value::String(today);
    body["clientVariables"]["Session_GUID"] = Value::String(uuid::Uuid::new_v4().to_string());
    body["clientVariables"]["DateTime"] =
        Value::String(chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string());

    body
}

/// Build the `FiltrosDePesquisa` in `OutSystems` format (lists as
/// `{"List": [...], "EmptyListItem": ""}`).
pub fn build_body_filtros(params: &DrSearchParams) -> Value {
    let tipo_conteudo: Vec<Value> = params
        .content_types
        .iter()
        .map(|ct| Value::String(ct.tipo_conteudo().to_owned()))
        .collect();

    let act_types: Vec<Value> = params.act_types.iter().map(|t| Value::String(t.clone())).collect();
    let series: Vec<Value> = params.series.iter().map(|s| Value::String(s.clone())).collect();

    let since_str = params.since.map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());
    let until_str = params.until.map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());

    let empty_list = || json!({"List": [], "EmptyListItem": ""});
    let s = |v: &str| Value::String(v.to_owned());

    let mut m = Map::new();
    m.insert("tipoConteudo".into(), json!({"List": tipo_conteudo}));
    m.insert("serie".into(), json!({"List": series}));
    m.insert("numero".into(), s(""));
    m.insert("ano".into(), s("0"));
    m.insert("suplemento".into(), s("0"));
    m.insert("dataPublicacao".into(), s(""));
    m.insert("dataPublicacaoDe".into(), Value::String(since_str));
    m.insert("dataPublicacaoAte".into(), Value::String(until_str));
    m.insert("parte".into(), s(""));
    m.insert("apendice".into(), s(""));
    m.insert("fasciculo".into(), s(""));
    m.insert("tipo".into(), json!({"List": act_types, "EmptyListItem": ""}));
    m.insert("emissor".into(), empty_list());
    m.insert("texto".into(), Value::String(params.query.clone()));
    m.insert("sumario".into(), s(""));
    m.insert("entidadeProponente".into(), empty_list());
    m.insert("numeroDR".into(), s(""));
    m.insert("paginaInicial".into(), s("0"));
    m.insert("paginaFinal".into(), s("0"));
    m.insert("dataAssinatura".into(), s(""));
    m.insert("dataDistribuicao".into(), s(""));
    m.insert("entidadePrincipal".into(), empty_list());
    m.insert("entidadeEmitente".into(), empty_list());
    m.insert("docType".into(), s(""));
    m.insert("proferido".into(), s(""));
    m.insert("processo".into(), s(""));
    m.insert("assunto".into(), s(""));
    m.insert("recorrente".into(), s(""));
    m.insert("recorrido".into(), s(""));
    m.insert("relator".into(), s(""));
    m.insert("empresa".into(), s(""));
    m.insert("concelho".into(), s(""));
    m.insert("nif".into(), s(""));
    m.insert("anuncio".into(), s(""));
    m.insert("numeroDoc".into(), s(""));
    m.insert("DataAssinaturaDe".into(), s("1900-01-01"));
    m.insert("DataAssinaturaAte".into(), s("1900-01-01"));
    m.insert("DataDistribuicaoDe".into(), s("1900-01-01"));
    m.insert("DataDistribuicaoAte".into(), s("1900-01-01"));
    m.insert("semestre".into(), s(""));
    m.insert("IsLegConsolidadaSelected".into(), Value::Bool(false));
    m.insert("IsFromData".into(), Value::Bool(false));
    m.insert("DescritorList".into(), empty_list());

    Value::Object(m)
}

// ---------------------------------------------------------------------------
// Search execution
// ---------------------------------------------------------------------------

const SEARCH_URL: &str = "https://diariodarepublica.pt/dr/screenservices/dr/Pesquisas/PesquisaResultado/DataActionGetPesquisas";

/// Execute a DR search.
///
/// Sets required cookies on the session's jar, builds the POST body, sends
/// the request, and parses the double-encoded `ElasticSearch` response.
pub async fn search(session: &DrSession, params: &DrSearchParams) -> Result<DrSearchResponse> {
    let url: Url = DrSession::base_url()
        .parse()
        .map_err(|_| LauyerError::Session { message: "Invalid base URL".to_owned() })?;

    // Set cookies
    let cookie_value = build_pesquisa_cookie(params);
    session.client().cookie_jar().add_cookie_str(
        &format!("PesquisaAvancada={cookie_value}; Path=/; Domain=diariodarepublica.pt"),
        &url,
    );
    session
        .client()
        .cookie_jar()
        .add_cookie_str("sort=8; Path=/; Domain=diariodarepublica.pt", &url);
    session
        .client()
        .cookie_jar()
        .add_cookie_str("ComesFrom=PA; Path=/; Domain=diariodarepublica.pt", &url);

    // Build body
    let body = build_search_body(session, params);

    info!(url = SEARCH_URL, "Executing DR search");

    // POST with custom headers using the raw reqwest client
    let response = session
        .client()
        .inner()
        .post(SEARCH_URL)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-CSRFToken", session.csrf_token())
        .header("Accept", "application/json")
        .header("outsystems-locale", "pt-PT")
        .json(&body)
        .send()
        .await
        .map_err(|e| LauyerError::Http { source: e, url: SEARCH_URL.to_owned() })?;

    let status = response.status();
    if !status.is_success() {
        return Err(LauyerError::Session { message: format!("DR search returned HTTP {status}") });
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| LauyerError::Http { source: e, url: SEARCH_URL.to_owned() })?;

    parse_search_response(&response_text)
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

/// Parse the outer response, then double-parse `data.Resultado` which is a
/// JSON string containing `ElasticSearch` results.
pub fn parse_search_response(response_text: &str) -> Result<DrSearchResponse> {
    let outer: Value = serde_json::from_str(response_text).map_err(|e| LauyerError::Parse {
        message: format!("Failed to parse DR response JSON: {e}"),
        source_url: SEARCH_URL.to_owned(),
    })?;

    // Check for exception
    if let Some(exception) = outer.get("exception") {
        let msg = exception.get("message").and_then(Value::as_str).unwrap_or("Unknown exception");
        return Err(LauyerError::Session { message: format!("DR API exception: {msg}") });
    }

    // Check for API version change
    if let Some(version_info) = outer.get("versionInfo") {
        if version_info.get("hasApiVersionChanged").and_then(Value::as_bool).unwrap_or(false) {
            warn!(
                "DR API version has changed — results may be stale. Session refresh may be needed."
            );
        }
    }

    let data = outer.get("data").ok_or_else(|| LauyerError::Parse {
        message: "Missing 'data' field in DR response".to_owned(),
        source_url: SEARCH_URL.to_owned(),
    })?;

    // Total count
    let total = data
        .get("ResultsCount")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    // Double-parse Resultado (it's a JSON string!)
    let resultado_str = data.get("Resultado").and_then(Value::as_str).unwrap_or("{}");

    let es_results: Value =
        serde_json::from_str(resultado_str).map_err(|e| LauyerError::Parse {
            message: format!("Failed to double-parse data.Resultado: {e}"),
            source_url: SEARCH_URL.to_owned(),
        })?;

    // Extract hits from ES response
    let hits =
        es_results.pointer("/hits/hits").and_then(Value::as_array).cloned().unwrap_or_default();

    let results: Vec<DrSearchResult> = hits.iter().filter_map(parse_hit).collect();

    info!(total, result_count = results.len(), "DR search results parsed");

    Ok(DrSearchResponse { total, results })
}

/// Apply a limit to search results, truncating if needed.
pub fn apply_limit(mut response: DrSearchResponse, limit: u32) -> DrSearchResponse {
    let limit = limit as usize;
    if response.results.len() > limit {
        response.results.truncate(limit);
    }
    response
}

/// Parse a single ES hit `_source` into a `DrSearchResult`.
fn parse_hit(hit: &Value) -> Option<DrSearchResult> {
    let source = hit.get("_source")?;

    let get_str =
        |key: &str| -> String { source.get(key).and_then(Value::as_str).unwrap_or("").to_owned() };

    let data_publicacao = source
        .get("dataPublicacao")
        .and_then(Value::as_str)
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let ano = source.get("ano").and_then(Value::as_u64).map(|v| v as u32);

    Some(DrSearchResult {
        title: get_str("title"),
        tipo: get_str("tipo"),
        numero: get_str("numero"),
        data_publicacao,
        emissor: get_str("emissor"),
        sumario: get_str("sumario"),
        serie: get_str("serie"),
        db_id: get_str("dbId"),
        file_id: get_str("fileId"),
        tipo_conteudo: get_str("tipoConteudo"),
        ano,
    })
}
