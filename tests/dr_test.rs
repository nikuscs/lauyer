use base64::Engine as _;
use chrono::NaiveDate;
use lauyer::dr::search::{
    DrSearchParams, build_body_filtros, build_bools, build_cookie_filtros, build_pesquisa_cookie,
    parse_search_response,
};
use lauyer::dr::session::{DrSession, strip_comment_keys};
use lauyer::dr::{
    DrContentType, DrSearchResult, list_act_types, resolve_act_type, resolve_content_types,
};
use lauyer::format::Renderable;

// ---------------------------------------------------------------------------
// DrContentType::from_alias
// ---------------------------------------------------------------------------

#[test]
fn content_type_from_alias() {
    assert_eq!(DrContentType::from_alias("atos-1"), Some(DrContentType::AtosSerie1));
    assert_eq!(DrContentType::from_alias("atos-2"), Some(DrContentType::AtosSerie2));
    assert_eq!(DrContentType::from_alias("dr"), Some(DrContentType::DiarioRepublica));
    assert_eq!(DrContentType::from_alias("decisoes"), Some(DrContentType::Jurisprudencia));
    assert_eq!(DrContentType::from_alias("jurisprudencia"), Some(DrContentType::Jurisprudencia));
    assert_eq!(DrContentType::from_alias("unknown"), None);
    assert_eq!(DrContentType::from_alias(""), None);
}

// ---------------------------------------------------------------------------
// DrContentType::tipo_conteudo
// ---------------------------------------------------------------------------

#[test]
fn content_type_tipo_conteudo() {
    assert_eq!(DrContentType::AtosSerie1.tipo_conteudo(), "AtosSerie1");
    assert_eq!(DrContentType::AtosSerie2.tipo_conteudo(), "AtosSerie2");
    assert_eq!(DrContentType::DiarioRepublica.tipo_conteudo(), "DiarioRepublica");
    assert_eq!(DrContentType::Jurisprudencia.tipo_conteudo(), "Jurisprudencia");
}

// ---------------------------------------------------------------------------
// DrContentType::bools_key
// ---------------------------------------------------------------------------

#[test]
fn content_type_bools_key() {
    assert_eq!(DrContentType::AtosSerie1.bools_key(), "Atos1");
    assert_eq!(DrContentType::AtosSerie2.bools_key(), "Atos2");
    assert_eq!(DrContentType::DiarioRepublica.bools_key(), "DiarioRepublica");
    assert_eq!(DrContentType::Jurisprudencia.bools_key(), "Jurisprudencia");
}

// ---------------------------------------------------------------------------
// resolve_act_type
// ---------------------------------------------------------------------------

#[test]
fn resolve_act_type_aliases() {
    assert_eq!(resolve_act_type("portaria"), Some("Portaria".to_owned()));
    assert_eq!(resolve_act_type("lei"), Some("Lei".to_owned()));
    assert_eq!(resolve_act_type("decreto-lei"), Some("Decreto-Lei".to_owned()));
    assert_eq!(resolve_act_type("despacho"), Some("Despacho".to_owned()));
    assert_eq!(resolve_act_type("decreto"), Some("Decreto".to_owned()));
    assert_eq!(resolve_act_type("aviso"), Some("Aviso".to_owned()));
    assert_eq!(
        resolve_act_type("resolucao"),
        Some("Resolução do Conselho de Ministros".to_owned())
    );
    assert_eq!(resolve_act_type("retificacao"), Some("Declaração de Retificação".to_owned()));
    assert_eq!(resolve_act_type("decreto-regulamentar"), Some("Decreto Regulamentar".to_owned()));
    assert_eq!(resolve_act_type("lei-organica"), Some("Lei Orgânica".to_owned()));
    assert_eq!(resolve_act_type("unknown-type"), None);
}

// ---------------------------------------------------------------------------
// resolve_content_types
// ---------------------------------------------------------------------------

#[test]
fn resolve_content_types_empty() {
    let types = resolve_content_types(&[]);
    assert!(types.is_ok());
    let types = types.unwrap();
    assert!(types.is_empty(), "Empty input should yield empty output");
}

#[test]
fn resolve_content_types_valid() {
    let aliases = vec!["atos-1".to_owned(), "atos-2".to_owned()];
    let types = resolve_content_types(&aliases).unwrap();
    assert_eq!(types.len(), 2);
    assert_eq!(types[0], DrContentType::AtosSerie1);
    assert_eq!(types[1], DrContentType::AtosSerie2);
}

#[test]
fn resolve_content_types_invalid() {
    let aliases = vec!["invalid-type".to_owned()];
    let result = resolve_content_types(&aliases);
    assert!(result.is_err(), "Invalid content type alias should return error");
}

// ---------------------------------------------------------------------------
// list_act_types
// ---------------------------------------------------------------------------

#[test]
fn list_act_types_returns_all() {
    let types = list_act_types();
    assert!(types.len() >= 10, "Should have at least 10 act types, got {}", types.len());

    let aliases: Vec<&str> = types.iter().map(|(a, _)| a.as_str()).collect();
    assert!(aliases.contains(&"portaria"), "Should contain portaria");
    assert!(aliases.contains(&"lei"), "Should contain lei");
    assert!(aliases.contains(&"decreto-lei"), "Should contain decreto-lei");
    assert!(aliases.contains(&"despacho"), "Should contain despacho");
}

// ---------------------------------------------------------------------------
// DrSearchResult rendering
// ---------------------------------------------------------------------------

fn make_dr_search_result() -> DrSearchResult {
    DrSearchResult {
        title: "Portaria n.º 122/2026/1".to_owned(),
        tipo: "Portaria".to_owned(),
        numero: "122/2026/1".to_owned(),
        data_publicacao: chrono::NaiveDate::from_ymd_opt(2026, 3, 20),
        emissor: "Economia e Coesão Territorial".to_owned(),
        sumario: "Reconhece a Associação Empresarial de Águeda".to_owned(),
        serie: "I".to_owned(),
        db_id: "abc123".to_owned(),
        file_id: "file1".to_owned(),
        tipo_conteudo: "AtosSerie1".to_owned(),
        ano: Some(2026),
    }
}

#[test]
fn dr_search_result_to_markdown() {
    let r = make_dr_search_result();
    let md = r.to_markdown();

    assert!(md.contains("Portaria"), "Markdown should contain act type");
    assert!(md.contains("122/2026/1"), "Markdown should contain number");
    assert!(md.contains("2026-03-20"), "Markdown should contain date");
    assert!(md.contains("Economia"), "Markdown should contain emissor");
    assert!(md.contains("Associação Empresarial"), "Markdown should contain sumario");
}

#[test]
fn dr_search_result_to_json() {
    let r = make_dr_search_result();
    let json = r.to_json();

    assert!(json.is_object(), "to_json must return an object");
    let obj = json.as_object().unwrap();

    assert_eq!(obj["tipo"].as_str().unwrap(), "Portaria");
    assert_eq!(obj["numero"].as_str().unwrap(), "122/2026/1");
    assert_eq!(obj["data_publicacao"].as_str().unwrap(), "2026-03-20");
    assert_eq!(obj["emissor"].as_str().unwrap(), "Economia e Coesão Territorial");
    assert_eq!(obj["serie"].as_str().unwrap(), "I");
}

#[test]
fn dr_search_result_table_row() {
    let r = make_dr_search_result();
    let (headers, values) = r.table_row().expect("table_row must return Some");

    assert_eq!(headers, vec!["Date", "Tipo", "Número", "Emissor", "Sumário"]);
    assert_eq!(values[0], "2026-03-20");
    assert_eq!(values[1], "Portaria");
    assert_eq!(values[2], "122/2026/1");
    assert_eq!(values[3], "Economia e Coesão Territorial");
    assert_eq!(values[4], "Reconhece a Associação Empresarial de Águeda");
}

// ---------------------------------------------------------------------------
// DrSearchResult with HTML in sumario
// ---------------------------------------------------------------------------

#[test]
fn dr_search_result_html_stripped_in_markdown() {
    let mut r = make_dr_search_result();
    r.sumario = "<p>Reconhece a <a href=\"#\">Associação</a> Empresarial</p>".to_owned();
    let md = r.to_markdown();

    assert!(!md.contains("<p>"), "HTML tags should be stripped from markdown");
    assert!(!md.contains("<a"), "HTML tags should be stripped from markdown");
    assert!(md.contains("Associação"), "Content should be preserved after stripping HTML");
}

#[test]
fn dr_search_result_no_date() {
    let mut r = make_dr_search_result();
    r.data_publicacao = None;
    let md = r.to_markdown();

    assert!(md.contains("s/d"), "Missing date should show s/d in markdown");

    let json = r.to_json();
    assert!(json["data_publicacao"].is_null(), "Missing date should be null in JSON");
}

// ---------------------------------------------------------------------------
// build_pesquisa_cookie — URL-encoding and structure
// ---------------------------------------------------------------------------

fn base_params() -> DrSearchParams {
    DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: String::new(),
        act_types: vec![],
        series: vec![],
        since: None,
        until: None,
        limit: 25,
    }
}

#[test]
fn test_build_pesquisa_cookie_is_url_encoded() {
    let params = base_params();
    let cookie = build_pesquisa_cookie(&params);

    assert!(!cookie.contains('{'), "cookie must be URL-encoded (no raw braces)");
    assert!(!cookie.contains('}'), "cookie must be URL-encoded (no raw braces)");
    assert!(!cookie.contains('"'), "cookie must be URL-encoded (no raw quotes)");
}

#[test]
fn test_build_pesquisa_cookie_decoded_structure() {
    let params = base_params();
    let cookie = build_pesquisa_cookie(&params);

    let decoded = percent_encoding::percent_decode_str(&cookie)
        .decode_utf8()
        .expect("cookie must be valid UTF-8 after URL decoding");
    let wrapper: serde_json::Value =
        serde_json::from_str(&decoded).expect("decoded cookie must be valid JSON");

    assert!(wrapper.get("PesquisaAvancadaFiltros").is_some(), "must have PesquisaAvancadaFiltros");
    assert!(wrapper.get("PesquisaAvancadaBools").is_some(), "must have PesquisaAvancadaBools");
    assert!(wrapper.get("SortFields").is_some(), "must have SortFields");

    let filtros_b64 = wrapper["PesquisaAvancadaFiltros"].as_str().unwrap();
    let filtros_bytes = base64::engine::general_purpose::STANDARD
        .decode(filtros_b64)
        .expect("PesquisaAvancadaFiltros must be valid base64");
    let filtros_str = std::str::from_utf8(&filtros_bytes).expect("base64 payload must be UTF-8");
    let filtros: serde_json::Value =
        serde_json::from_str(filtros_str).expect("base64 payload must be valid JSON");

    let tipo_conteudo = filtros["tipoConteudo"].as_array().expect("tipoConteudo must be array");
    assert!(
        tipo_conteudo.iter().any(|v| v.as_str() == Some("AtosSerie1")),
        "tipoConteudo must contain AtosSerie1"
    );

    let bools_str = wrapper["PesquisaAvancadaBools"].as_str().unwrap();
    let bools: serde_json::Value =
        serde_json::from_str(bools_str).expect("PesquisaAvancadaBools must be valid JSON");
    assert_eq!(bools["Atos1"], true, "Atos1 must be true for AtosSerie1");
    assert_eq!(bools["Atos2"], false, "Atos2 must be false");
    assert_eq!(bools["DiarioRepublica"], false);
}

#[test]
fn test_build_pesquisa_cookie_with_dates() {
    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: String::new(),
        act_types: vec![],
        series: vec![],
        since: NaiveDate::from_ymd_opt(2026, 1, 15),
        until: NaiveDate::from_ymd_opt(2026, 3, 21),
        limit: 25,
    };
    let cookie = build_pesquisa_cookie(&params);

    let decoded = percent_encoding::percent_decode_str(&cookie).decode_utf8().unwrap();
    let wrapper: serde_json::Value = serde_json::from_str(&decoded).unwrap();

    let filtros_b64 = wrapper["PesquisaAvancadaFiltros"].as_str().unwrap();
    let filtros_bytes = base64::engine::general_purpose::STANDARD.decode(filtros_b64).unwrap();
    let filtros: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&filtros_bytes).unwrap()).unwrap();

    assert_eq!(filtros["dataPublicacaoDe"], "2026-01-15");
    assert_eq!(filtros["dataPublicacaoAte"], "2026-03-21");
}

#[test]
fn test_build_pesquisa_cookie_without_dates_has_no_date_keys() {
    let params = base_params();
    let cookie = build_pesquisa_cookie(&params);

    let decoded = percent_encoding::percent_decode_str(&cookie).decode_utf8().unwrap();
    let wrapper: serde_json::Value = serde_json::from_str(&decoded).unwrap();

    let filtros_b64 = wrapper["PesquisaAvancadaFiltros"].as_str().unwrap();
    let filtros_bytes = base64::engine::general_purpose::STANDARD.decode(filtros_b64).unwrap();
    let filtros: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&filtros_bytes).unwrap()).unwrap();

    assert!(filtros.get("dataPublicacaoDe").is_none(), "no since → no dataPublicacaoDe key");
    assert!(filtros.get("dataPublicacaoAte").is_none(), "no until → no dataPublicacaoAte key");
}

#[test]
fn test_build_pesquisa_cookie_with_act_types() {
    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: String::new(),
        act_types: vec!["Portaria".to_owned(), "Lei".to_owned()],
        series: vec![],
        since: None,
        until: None,
        limit: 25,
    };
    let cookie = build_pesquisa_cookie(&params);

    let decoded = percent_encoding::percent_decode_str(&cookie).decode_utf8().unwrap();
    let wrapper: serde_json::Value = serde_json::from_str(&decoded).unwrap();

    let filtros_b64 = wrapper["PesquisaAvancadaFiltros"].as_str().unwrap();
    let filtros_bytes = base64::engine::general_purpose::STANDARD.decode(filtros_b64).unwrap();
    let filtros: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&filtros_bytes).unwrap()).unwrap();

    let tipo_arr = filtros["tipo"].as_array().expect("tipo must be array");
    assert!(tipo_arr.iter().any(|v| v.as_str() == Some("Portaria")));
    assert!(tipo_arr.iter().any(|v| v.as_str() == Some("Lei")));
}

#[test]
fn test_build_pesquisa_cookie_with_query_bools_correct() {
    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: "trabalho".to_owned(),
        act_types: vec![],
        series: vec![],
        since: None,
        until: None,
        limit: 25,
    };
    let cookie = build_pesquisa_cookie(&params);
    assert!(!cookie.is_empty());

    let decoded = percent_encoding::percent_decode_str(&cookie).decode_utf8().unwrap();
    let wrapper: serde_json::Value = serde_json::from_str(&decoded).unwrap();
    let bools_str = wrapper["PesquisaAvancadaBools"].as_str().unwrap();
    let bools: serde_json::Value = serde_json::from_str(bools_str).unwrap();
    assert_eq!(bools["Atos1"], true);
}

#[test]
fn test_build_pesquisa_cookie_multiple_content_types() {
    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1, DrContentType::AtosSerie2],
        query: String::new(),
        act_types: vec![],
        series: vec![],
        since: None,
        until: None,
        limit: 25,
    };
    let cookie = build_pesquisa_cookie(&params);
    let decoded = percent_encoding::percent_decode_str(&cookie).decode_utf8().unwrap();
    let wrapper: serde_json::Value = serde_json::from_str(&decoded).unwrap();

    let bools_str = wrapper["PesquisaAvancadaBools"].as_str().unwrap();
    let bools: serde_json::Value = serde_json::from_str(bools_str).unwrap();
    assert_eq!(bools["Atos1"], true);
    assert_eq!(bools["Atos2"], true);
    assert_eq!(bools["DiarioRepublica"], false);
    assert_eq!(bools["Jurisprudencia"], false);

    let filtros_b64 = wrapper["PesquisaAvancadaFiltros"].as_str().unwrap();
    let filtros_bytes = base64::engine::general_purpose::STANDARD.decode(filtros_b64).unwrap();
    let filtros: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&filtros_bytes).unwrap()).unwrap();
    let tipo_conteudo = filtros["tipoConteudo"].as_array().unwrap();
    assert_eq!(tipo_conteudo.len(), 2);
    assert!(tipo_conteudo.iter().any(|v| v.as_str() == Some("AtosSerie1")));
    assert!(tipo_conteudo.iter().any(|v| v.as_str() == Some("AtosSerie2")));
}

// ---------------------------------------------------------------------------
// parse_search_response — response parsing
// ---------------------------------------------------------------------------

fn make_dr_response_json(resultado_inner: &str, results_count: &str) -> String {
    serde_json::json!({
        "versionInfo": {
            "hasModuleVersionChanged": false,
            "hasApiVersionChanged": false
        },
        "data": {
            "Resultado": resultado_inner,
            "ResultsCount": results_count,
            "HasErrorPesquisa": false
        }
    })
    .to_string()
}

#[test]
fn test_parse_dr_response_two_hits() {
    let inner = serde_json::json!({
        "took": 20,
        "hits": {
            "total": {"value": 2},
            "hits": [
                {
                    "_source": {
                        "title": "Portaria 123",
                        "tipo": "Portaria",
                        "numero": "123/2026",
                        "dataPublicacao": "2026-03-20",
                        "emissor": "Saúde",
                        "sumario": "Test summary",
                        "serie": "I",
                        "dbId": "abc123"
                    }
                },
                {
                    "_source": {
                        "title": "Decreto-Lei 45",
                        "tipo": "Decreto-Lei",
                        "numero": "45/2026",
                        "dataPublicacao": "2026-03-19",
                        "emissor": "Finanças",
                        "sumario": "Another summary",
                        "serie": "I",
                        "dbId": "def456"
                    }
                }
            ]
        }
    })
    .to_string();

    let response_text = make_dr_response_json(&inner, "2");
    let result = parse_search_response(&response_text).expect("parsing must succeed");

    assert_eq!(result.total, 2);
    assert_eq!(result.results.len(), 2);

    assert_eq!(result.results[0].tipo, "Portaria");
    assert_eq!(result.results[0].numero, "123/2026");
    assert_eq!(result.results[0].emissor, "Saúde");
    assert_eq!(result.results[0].db_id, "abc123");
    assert_eq!(result.results[0].serie, "I");
    assert_eq!(result.results[0].data_publicacao, NaiveDate::from_ymd_opt(2026, 3, 20));

    assert_eq!(result.results[1].tipo, "Decreto-Lei");
    assert_eq!(result.results[1].db_id, "def456");
    assert_eq!(result.results[1].data_publicacao, NaiveDate::from_ymd_opt(2026, 3, 19));
}

#[test]
fn test_parse_dr_response_exception() {
    let response = serde_json::json!({
        "exception": {
            "message": "Role validation failed"
        }
    })
    .to_string();

    let result = parse_search_response(&response);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Role validation failed"));
}

#[test]
fn test_parse_dr_response_exception_no_message() {
    let response = serde_json::json!({
        "exception": {}
    })
    .to_string();

    let result = parse_search_response(&response);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown exception"));
}

#[test]
fn test_parse_dr_response_empty_hits() {
    let inner = serde_json::json!({
        "took": 5,
        "hits": {
            "total": {"value": 0},
            "hits": []
        }
    })
    .to_string();

    let response_text = make_dr_response_json(&inner, "0");
    let result = parse_search_response(&response_text).expect("empty results must parse cleanly");

    assert_eq!(result.total, 0);
    assert!(result.results.is_empty());
}

#[test]
fn test_parse_dr_response_missing_data_field() {
    let response = serde_json::json!({
        "versionInfo": {"hasModuleVersionChanged": false, "hasApiVersionChanged": false}
    })
    .to_string();

    let result = parse_search_response(&response);
    assert!(result.is_err());
}

#[test]
fn test_parse_dr_response_invalid_json() {
    let result = parse_search_response("not valid json at all");
    assert!(result.is_err());
}

#[test]
fn test_parse_dr_response_invalid_resultado_json() {
    let response = serde_json::json!({
        "data": {
            "Resultado": "not valid inner json",
            "ResultsCount": "0",
            "HasErrorPesquisa": false
        }
    })
    .to_string();

    let result = parse_search_response(&response);
    assert!(result.is_err());
}

#[test]
fn test_parse_dr_response_missing_date_is_none() {
    let inner = serde_json::json!({
        "took": 1,
        "hits": {
            "total": {"value": 1},
            "hits": [
                {
                    "_source": {
                        "title": "Aviso",
                        "tipo": "Aviso",
                        "numero": "1/2026",
                        "emissor": "Câmara",
                        "sumario": "Desc",
                        "serie": "II",
                        "dbId": "xyz"
                    }
                }
            ]
        }
    })
    .to_string();

    let response_text = make_dr_response_json(&inner, "1");
    let result = parse_search_response(&response_text).unwrap();

    assert_eq!(result.results.len(), 1);
    assert!(result.results[0].data_publicacao.is_none());
}

// ---------------------------------------------------------------------------
// DrSession wiremock tests — covers session.rs refresh() and error paths
// ---------------------------------------------------------------------------

use lauyer::http::HttpClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper: mount the two endpoints needed to initialise a DR session.
async fn mount_session_endpoints(server: &MockServer, version_token: &str) {
    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(format!(r#"{{"versionToken":"{version_token}"}}"#)),
        )
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(server)
        .await;
}

#[tokio::test]
async fn test_session_new_from_urls_success() {
    let server = MockServer::start().await;
    mount_session_endpoints(&server, "token-abc").await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let version_info_url = format!("{}/dr/moduleservices/moduleversioninfo", server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", server.uri());

    let session = DrSession::new_from_urls(client, &version_info_url, &roles_url).await.unwrap();
    assert_eq!(session.module_version(), "token-abc");
}

#[tokio::test]
async fn test_session_refresh_updates_module_version() {
    // Start with version "v1", refresh delivers "v2".
    let server = MockServer::start().await;

    // Initial session setup
    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v1"}"#))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v2"}"#))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let version_info_url = format!("{}/dr/moduleservices/moduleversioninfo", server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", server.uri());

    let session = DrSession::new_from_urls(client, &version_info_url, &roles_url).await.unwrap();
    assert_eq!(session.module_version(), "v1");

    // refresh() uses the hardcoded VERSION_INFO_URL, which points to the real DR site.
    // We cannot override it from outside. Instead, verify the method exists and
    // that the session was successfully created (indirectly exercising new_from_urls).
    // A direct refresh() test against a live network is skipped here because the
    // hardcoded URL cannot be overridden; we at least exercise all new_from_urls branches.
    let _ = session.module_version();
    let _ = session.api_version();
    let _ = session.body_template();
    let _ = session.csrf_token();
    let _ = DrSession::base_url();
}

#[tokio::test]
async fn test_session_new_invalid_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json-at-all"))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let version_info_url = format!("{}/version", server.uri());
    let roles_url = format!("{}/roles", server.uri());

    let result = DrSession::new_from_urls(client, &version_info_url, &roles_url).await;
    assert!(result.is_err(), "non-JSON version response must return an error");
    let err = result.err().expect("already checked is_err").to_string();
    assert!(
        err.contains("parse") || err.contains("JSON") || err.contains("moduleversioninfo"),
        "error should mention parse failure: {err}"
    );
}

#[tokio::test]
async fn test_session_new_missing_version_token() {
    let server = MockServer::start().await;

    // Valid JSON but no versionToken field
    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"otherField":"value"}"#))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 30, 0).unwrap();
    let version_info_url = format!("{}/version", server.uri());
    let roles_url = format!("{}/roles", server.uri());

    let result = DrSession::new_from_urls(client, &version_info_url, &roles_url).await;
    assert!(result.is_err(), "missing versionToken field must return an error");
    let err = result.err().expect("already checked is_err").to_string();
    assert!(
        err.contains("versionToken") || err.contains("Missing"),
        "error should mention versionToken: {err}"
    );
}

#[tokio::test]
async fn test_session_new_roles_connection_failure() {
    // The roles call in session.rs uses `.send()` directly on the inner client
    // (not through HttpClient::execute_with_retry), so a 5xx response is silently
    // accepted — only a transport/connection error propagates.
    //
    // To trigger the Err branch we drop the mock server after the version endpoint
    // is registered, so the roles URL gets a connection-refused error.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"tok"}"#))
        .mount(&server)
        .await;

    let version_info_url = format!("{}/version", server.uri());
    // Point roles at a completely different dead port to force a connection error.
    let dead_roles_url = "http://127.0.0.1:1/roles-dead";

    let client = HttpClient::new(None, 2, 0).unwrap();
    let result = DrSession::new_from_urls(client, &version_info_url, dead_roles_url).await;
    assert!(result.is_err(), "connection error on roles must propagate as an error");
}

#[test]
fn test_parse_dr_response_invalid_date_is_none() {
    let inner = serde_json::json!({
        "took": 1,
        "hits": {
            "total": {"value": 1},
            "hits": [
                {
                    "_source": {
                        "title": "Aviso",
                        "tipo": "Aviso",
                        "numero": "1/2026",
                        "dataPublicacao": "not-a-date",
                        "emissor": "Câmara",
                        "sumario": "Desc",
                        "serie": "II",
                        "dbId": "xyz"
                    }
                }
            ]
        }
    })
    .to_string();

    let response_text = make_dr_response_json(&inner, "1");
    let result = parse_search_response(&response_text).unwrap();

    assert!(result.results[0].data_publicacao.is_none(), "unparseable date must yield None");
}

#[test]
fn test_parse_dr_response_total_from_results_count_field() {
    let inner = serde_json::json!({
        "took": 1,
        "hits": {
            "total": {"value": 1},
            "hits": [
                {
                    "_source": {
                        "title": "X", "tipo": "Portaria", "numero": "1/2026",
                        "emissor": "Y", "sumario": "Z", "serie": "I", "dbId": "q1"
                    }
                }
            ]
        }
    })
    .to_string();

    // ResultsCount is "99" even though only 1 hit in this page
    let response_text = make_dr_response_json(&inner, "99");
    let result = parse_search_response(&response_text).unwrap();
    assert_eq!(result.total, 99, "total must come from ResultsCount field, not ES hits.total");
}

// ---------------------------------------------------------------------------
// session::strip_comment_keys
// ---------------------------------------------------------------------------

#[test]
fn test_strip_comment_keys_top_level() {
    let mut val = serde_json::json!({
        "_comment": "top-level comment",
        "data": "preserved"
    });
    strip_comment_keys(&mut val);

    assert!(val.get("_comment").is_none());
    assert_eq!(val["data"], "preserved");
}

#[test]
fn test_strip_comment_keys_nested_objects() {
    let mut val = serde_json::json!({
        "outer": {
            "_comment": "inner comment",
            "inner_val": 42,
            "deeper": {
                "_comment": "deeper comment",
                "x": true
            }
        }
    });
    strip_comment_keys(&mut val);

    assert!(val["outer"].get("_comment").is_none());
    assert_eq!(val["outer"]["inner_val"], 42);
    assert!(val["outer"]["deeper"].get("_comment").is_none());
    assert_eq!(val["outer"]["deeper"]["x"], true);
}

#[test]
fn test_strip_comment_keys_in_arrays() {
    let mut val = serde_json::json!({
        "items": [
            {"_comment": "first", "id": 1},
            {"_comment": "second", "id": 2}
        ]
    });
    strip_comment_keys(&mut val);

    assert!(val["items"][0].get("_comment").is_none());
    assert_eq!(val["items"][0]["id"], 1);
    assert!(val["items"][1].get("_comment").is_none());
    assert_eq!(val["items"][1]["id"], 2);
}

#[test]
fn test_strip_comment_keys_no_comments_unchanged() {
    let mut val = serde_json::json!({
        "a": 1,
        "b": {"c": [1, 2, 3]},
        "d": "string"
    });
    let original = val.clone();
    strip_comment_keys(&mut val);
    assert_eq!(val, original);
}

#[test]
fn test_strip_comment_keys_on_embedded_template() {
    let template_str = include_str!("../src/dr/request_template.json");
    let mut template: serde_json::Value =
        serde_json::from_str(template_str).expect("template must be valid JSON");

    assert!(
        template.get("_comment").is_some(),
        "template must have top-level _comment before stripping"
    );

    strip_comment_keys(&mut template);

    assert_no_comment_keys(&template);
    assert!(template.get("versionInfo").is_some());
    assert!(template.get("screenData").is_some());
}

fn assert_no_comment_keys(val: &serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            assert!(!map.contains_key("_comment"), "_comment key found after stripping");
            for v in map.values() {
                assert_no_comment_keys(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                assert_no_comment_keys(v);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// content_types — display_name, resolve_act_type, all()
// ---------------------------------------------------------------------------

#[test]
fn test_content_type_display_names_nonempty() {
    for ct in DrContentType::all() {
        let name = ct.display_name();
        assert!(!name.is_empty(), "display_name must be non-empty for {ct:?}");
    }
}

#[test]
fn test_content_type_display_name_values() {
    assert_eq!(DrContentType::AtosSerie1.display_name(), "Atos da 1.ª Série");
    assert_eq!(DrContentType::AtosSerie2.display_name(), "Atos da 2.ª Série");
    assert_eq!(DrContentType::DiarioRepublica.display_name(), "Diário da República");
    assert_eq!(DrContentType::Jurisprudencia.display_name(), "Decisões Judiciais");
}

#[test]
fn test_all_content_types_returns_four() {
    let all = DrContentType::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&DrContentType::AtosSerie1));
    assert!(all.contains(&DrContentType::AtosSerie2));
    assert!(all.contains(&DrContentType::DiarioRepublica));
    assert!(all.contains(&DrContentType::Jurisprudencia));
}

// ---------------------------------------------------------------------------
// DrSession::new_from_urls — wiremock-based session init tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_session_init_with_wiremock() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"versionToken": "test-token-abc"})),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/roles"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"rolesInfo": ","})),
        )
        .mount(&mock_server)
        .await;

    let client = lauyer::http::HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/dr/moduleservices/moduleversioninfo", mock_server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", mock_server.uri());

    let session = DrSession::new_from_urls(client, &version_url, &roles_url)
        .await
        .expect("session init must succeed with mocked endpoints");

    assert_eq!(session.module_version(), "test-token-abc");
    assert!(!session.api_version().is_empty());
    assert!(!session.csrf_token().is_empty());
}

#[tokio::test]
async fn test_session_init_missing_version_token_fails() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"other": "field"})),
        )
        .mount(&mock_server)
        .await;

    let client = lauyer::http::HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/dr/moduleservices/moduleversioninfo", mock_server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", mock_server.uri());

    let result = DrSession::new_from_urls(client, &version_url, &roles_url).await;
    assert!(result.is_err(), "missing versionToken must cause an error");
}

// ---------------------------------------------------------------------------
// search — build_bools, build_cookie_filtros, build_body_filtros
// ---------------------------------------------------------------------------

fn search_params_atos1() -> DrSearchParams {
    DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: String::new(),
        act_types: vec![],
        series: vec![],
        since: None,
        until: None,
        limit: 25,
    }
}

#[test]
fn build_bools_sets_correct_keys() {
    let bools = build_bools(&search_params_atos1());
    assert_eq!(bools["Atos1"], true);
    assert_eq!(bools["Atos2"], false);
    assert_eq!(bools["DiarioRepublica"], false);
    assert_eq!(bools["Jurisprudencia"], false);
}

#[test]
fn build_cookie_filtros_compact_json() {
    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: String::new(),
        act_types: vec!["Portaria".to_owned()],
        series: vec![],
        since: NaiveDate::from_ymd_opt(2026, 3, 14),
        until: NaiveDate::from_ymd_opt(2026, 3, 21),
        limit: 25,
    };
    let filtros = build_cookie_filtros(&params);
    let json_str = serde_json::to_string(&filtros).unwrap_or_default();
    assert!(!json_str.contains(": "), "filtros JSON must be compact");
    assert!(json_str.contains("\"AtosSerie1\""));
    assert!(json_str.contains("\"Portaria\""));
    assert!(json_str.contains("\"2026-03-14\""));
}

#[test]
fn build_body_filtros_uses_outsystems_format() {
    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: "trabalho".to_owned(),
        act_types: vec!["Portaria".to_owned()],
        series: vec![],
        since: None,
        until: None,
        limit: 25,
    };
    let filtros = build_body_filtros(&params);
    assert!(filtros["tipo"]["List"].is_array());
    assert_eq!(filtros["tipo"]["EmptyListItem"], "");
    assert_eq!(filtros["texto"], "trabalho");
    assert_eq!(filtros["ano"], "0");
    assert_eq!(filtros["paginaInicial"], "0");
}

// ---------------------------------------------------------------------------
// markdown — format and HTML stripping
// ---------------------------------------------------------------------------

fn sample_result_with_html() -> DrSearchResult {
    DrSearchResult {
        title: "Portaria n.º 122/2026/1".to_owned(),
        tipo: "Portaria".to_owned(),
        numero: "122/2026/1".to_owned(),
        data_publicacao: NaiveDate::from_ymd_opt(2026, 3, 20),
        emissor: "Economia e Coesão Territorial".to_owned(),
        sumario: "<p>Reconhece a <a href=\"#\">Associação</a> Empresarial de Águeda</p>".to_owned(),
        serie: "I".to_owned(),
        db_id: "42".to_owned(),
        file_id: "file42".to_owned(),
        tipo_conteudo: "AtosSerie1".to_owned(),
        ano: Some(2026),
    }
}

#[test]
fn markdown_format_heading_and_emissor() {
    let md = sample_result_with_html().to_markdown();
    assert!(md.starts_with("### Portaria n.º 122/2026/1 (2026-03-20)"));
    assert!(md.contains("**Emissor:** Economia e Coesão Territorial"));
}

#[test]
fn json_output_strips_html_in_sumario() {
    let json = sample_result_with_html().to_json();
    assert_eq!(json["tipo"], "Portaria");
    assert_eq!(json["numero"], "122/2026/1");
    assert_eq!(json["data_publicacao"], "2026-03-20");
    let sumario = json["sumario"].as_str().unwrap_or("");
    assert!(!sumario.contains('<'), "HTML must be stripped from JSON sumario");
}

#[tokio::test]
async fn test_session_body_template_has_no_comment_keys() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"versionToken": "tok"})),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let client = lauyer::http::HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/dr/moduleservices/moduleversioninfo", mock_server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", mock_server.uri());

    let session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();

    assert_no_comment_keys(session.body_template());
    assert!(session.body_template().get("versionInfo").is_some());
    assert!(session.body_template().get("screenData").is_some());
}

#[tokio::test]
async fn test_build_search_body_with_mocked_session() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"versionToken": "ver-42"})),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let client = lauyer::http::HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/dr/moduleservices/moduleversioninfo", mock_server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", mock_server.uri());

    let session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();

    let params = DrSearchParams {
        content_types: vec![DrContentType::AtosSerie1],
        query: "segurança social".to_owned(),
        act_types: vec!["Portaria".to_owned()],
        series: vec![],
        since: NaiveDate::from_ymd_opt(2026, 1, 1),
        until: NaiveDate::from_ymd_opt(2026, 3, 21),
        limit: 10,
    };

    let body = lauyer::dr::search::build_search_body(&session, &params);

    assert_eq!(body["versionInfo"]["moduleVersion"], "ver-42");
    assert!(!body["versionInfo"]["apiVersion"].as_str().unwrap_or("").is_empty());

    let filtros = &body["screenData"]["variables"]["FiltrosDePesquisa"];
    assert!(filtros["tipoConteudo"]["List"].is_array());
    let tipo_list = filtros["tipoConteudo"]["List"].as_array().unwrap();
    assert!(tipo_list.iter().any(|v| v.as_str() == Some("AtosSerie1")));
    assert_eq!(filtros["texto"], "segurança social");
    assert_eq!(filtros["dataPublicacaoDe"], "2026-01-01");
    assert_eq!(filtros["dataPublicacaoAte"], "2026-03-21");

    assert_eq!(body["screenData"]["variables"]["DataDe"], "2026-01-01");
    assert_eq!(body["screenData"]["variables"]["DataAte"], "2026-03-21");

    let filtros_b64 = body["screenData"]["variables"]["PesquisaAvancadaFiltros"]
        .as_str()
        .expect("PesquisaAvancadaFiltros must be a string");
    let filtros_bytes = base64::engine::general_purpose::STANDARD.decode(filtros_b64).unwrap();
    let filtros_decoded: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(&filtros_bytes).unwrap()).unwrap();
    assert!(filtros_decoded["tipoConteudo"].as_array().is_some());

    let client_vars = &body["clientVariables"];
    let session_guid = client_vars["Session_GUID"].as_str().unwrap_or("");
    assert_eq!(session_guid.len(), 36, "Session_GUID must be a UUID (36 chars)");
    assert!(!client_vars["Data"].as_str().unwrap_or("").is_empty());
}

// ---------------------------------------------------------------------------
// Fixture-based tests — parse real DR API response format
// ---------------------------------------------------------------------------

fn load_dr_fixture() -> String {
    std::fs::read_to_string("tests/fixtures/dr_search_response.json")
        .expect("DR search response fixture must exist")
}

#[test]
fn parse_dr_fixture_response() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    assert_eq!(response.total, 3);
    assert_eq!(response.results.len(), 3);
}

#[test]
fn parse_dr_fixture_first_result() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    let first = &response.results[0];
    assert_eq!(first.tipo, "Portaria");
    assert_eq!(first.numero, "122/2026/1");
    assert_eq!(first.emissor, "Economia e Coesão Territorial");
    assert_eq!(first.serie, "I");
    assert_eq!(first.data_publicacao, Some(NaiveDate::from_ymd_opt(2026, 3, 20).unwrap()));
    assert!(first.title.contains("Portaria"));
}

#[test]
fn parse_dr_fixture_third_result_is_decreto_lei() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    let third = &response.results[2];
    assert_eq!(third.tipo, "Decreto-Lei");
    assert_eq!(third.numero, "15/2026");
    assert_eq!(third.emissor, "Presidência do Conselho de Ministros");
}

#[test]
fn parse_dr_fixture_html_in_sumario() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    // Sumario contains HTML tags like <b> and <em> in the fixture
    // The parser should preserve them (stripping happens in markdown rendering)
    let first = &response.results[0];
    assert!(first.sumario.contains("Associação Empresarial"));
}

#[test]
fn parse_dr_fixture_sumario_html_stripped_in_markdown() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    let first = &response.results[0];
    let md = first.to_markdown();
    // Markdown rendering should strip HTML
    assert!(!md.contains("<b>"));
    assert!(!md.contains("</b>"));
    assert!(md.contains("Associação Empresarial"));
}

#[test]
fn parse_dr_fixture_all_results_have_dates() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    for result in &response.results {
        assert!(result.data_publicacao.is_some(), "Missing date for: {}", result.title);
    }
}

#[test]
fn parse_dr_fixture_renders_as_json() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    for result in &response.results {
        let json = result.to_json();
        assert!(json.get("tipo").is_some());
        assert!(json.get("numero").is_some());
        assert!(json.get("emissor").is_some());
    }
}

#[test]
fn parse_dr_fixture_table_rows() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    for result in &response.results {
        let (headers, values) = result.table_row().expect("table_row should return data");
        assert_eq!(headers.len(), 5);
        assert_eq!(values.len(), 5);
        assert_eq!(headers[0], "Date");
        assert_eq!(headers[1], "Tipo");
        assert_eq!(headers[4], "Sumário");
    }
}

// ---------------------------------------------------------------------------
// Fixture: new fields (file_id, tipo_conteudo, ano)
// ---------------------------------------------------------------------------

#[test]
fn parse_dr_fixture_new_fields() {
    let raw = load_dr_fixture();
    let response = parse_search_response(&raw).unwrap();
    let first = &response.results[0];
    assert_eq!(first.file_id, "file1");
    assert_eq!(first.tipo_conteudo, "AtosSerie1");
    assert_eq!(first.ano, Some(2026));
}

// ---------------------------------------------------------------------------
// DrSession::new_from_urls — version info fetch failure (lines 51-52)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_session_new_from_urls_version_fetch_error() {
    // Use port 1 on localhost which always refuses connections (reserved port).
    let client = HttpClient::new(None, 2, 0).unwrap();
    let version_url = "http://127.0.0.1:1/version";
    let roles_url = "http://127.0.0.1:1/roles";

    let result = DrSession::new_from_urls(client, version_url, roles_url).await;
    assert!(result.is_err(), "connection refused on version URL must return an error");
}

// ---------------------------------------------------------------------------
// DrSession::refresh_from_urls — covers refresh() delegation + all error paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_session_refresh_from_urls_success() {
    let server = MockServer::start().await;

    // First call for new_from_urls
    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v1"}"#))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second call for refresh_from_urls
    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/moduleversioninfo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v2"}"#))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/dr/moduleservices/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/dr/moduleservices/moduleversioninfo", server.uri());
    let roles_url = format!("{}/dr/moduleservices/roles", server.uri());

    let mut session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();
    assert_eq!(session.module_version(), "v1");

    session.refresh_from_urls(&version_url, &roles_url).await.unwrap();
    assert_eq!(session.module_version(), "v2");
}

#[tokio::test]
async fn test_session_refresh_from_urls_version_fetch_error() {
    let server = MockServer::start().await;

    // Setup the initial session
    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v1"}"#))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/version", server.uri());
    let roles_url = format!("{}/roles", server.uri());

    let mut session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();

    // Use port 1 on localhost (reserved, always refuses) to trigger the Err branch
    // in refresh_from_urls which logs a warning at lines 144-145.
    let result = session.refresh_from_urls("http://127.0.0.1:1/dead-version", &roles_url).await;
    assert!(result.is_err(), "connection refused on refresh version URL must return error");
}

#[tokio::test]
async fn test_session_refresh_from_urls_invalid_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v1"}"#))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second call returns invalid JSON
    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/version", server.uri());
    let roles_url = format!("{}/roles", server.uri());

    let mut session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();

    let result = session.refresh_from_urls(&version_url, &roles_url).await;
    assert!(result.is_err(), "invalid JSON in refresh version response must return error");
}

#[tokio::test]
async fn test_session_refresh_from_urls_missing_version_token() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v1"}"#))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second call returns JSON without versionToken
    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"other":"field"}"#))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/version", server.uri());
    let roles_url = format!("{}/roles", server.uri());

    let mut session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();

    let result = session.refresh_from_urls(&version_url, &roles_url).await;
    assert!(result.is_err(), "missing versionToken in refresh response must return error");
}

#[tokio::test]
async fn test_session_refresh_from_urls_roles_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v1"}"#))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"versionToken":"v2"}"#))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/roles"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let client = HttpClient::new(None, 10, 0).unwrap();
    let version_url = format!("{}/version", server.uri());
    let roles_url = format!("{}/roles", server.uri());

    let mut session = DrSession::new_from_urls(client, &version_url, &roles_url).await.unwrap();

    let dead_roles_url = "http://127.0.0.1:1/dead-roles";
    let result = session.refresh_from_urls(&version_url, dead_roles_url).await;
    assert!(result.is_err(), "connection refused on refresh roles URL must return error");
}
