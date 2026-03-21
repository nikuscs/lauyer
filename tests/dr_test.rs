use lawyerr::dr::{
    DrContentType, DrSearchResult, list_act_types, resolve_act_type, resolve_content_types,
};
use lawyerr::format::Renderable;

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

    assert_eq!(headers, vec!["Date", "Tipo", "Número", "Emissor"]);
    assert_eq!(values[0], "2026-03-20");
    assert_eq!(values[1], "Portaria");
    assert_eq!(values[2], "122/2026/1");
    assert_eq!(values[3], "Economia e Coesão Territorial");
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
// Integration tests (require network, marked #[ignore])
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access to diariodarepublica.pt"]
async fn live_dr_session_init() {
    let client = lawyerr::http::HttpClient::new(None, 30, 3).unwrap();
    let session = lawyerr::dr::DrSession::new(client).await.unwrap();
    assert!(!session.module_version().is_empty());
}

#[tokio::test]
#[ignore = "requires network access to diariodarepublica.pt"]
async fn live_dr_search_portarias() {
    let client = lawyerr::http::HttpClient::new(None, 30, 3).unwrap();
    let session = lawyerr::dr::DrSession::new(client).await.unwrap();
    let params = lawyerr::dr::DrSearchParams {
        content_types: vec![lawyerr::dr::DrContentType::AtosSerie1],
        query: String::new(),
        act_types: vec!["Portaria".to_owned()],
        since: Some(chrono::Local::now().date_naive() - chrono::Duration::weeks(1)),
        until: Some(chrono::Local::now().date_naive()),
        limit: 5,
    };
    let response = lawyerr::dr::search(&session, &params).await.unwrap();
    assert!(response.total > 0);
    assert!(!response.results.is_empty());
}
