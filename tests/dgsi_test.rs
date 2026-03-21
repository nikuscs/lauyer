use chrono::NaiveDate;
use lawyerr::dgsi::courts::Court;
use lawyerr::dgsi::decision::parse_decision;
use lawyerr::dgsi::search::{build_query, parse_search_results};
use lawyerr::format::Renderable;

// ---------------------------------------------------------------------------
// Court helpers
// ---------------------------------------------------------------------------

#[test]
fn court_from_alias() {
    assert_eq!(Court::from_alias("stj"), Some(Court::Stj));
    assert_eq!(Court::from_alias("STJ"), Some(Court::Stj));
    assert_eq!(Court::from_alias("sta"), Some(Court::Sta));
    assert_eq!(Court::from_alias("conflitos"), Some(Court::Conflitos));
    assert_eq!(Court::from_alias("rel-porto"), Some(Court::RelPorto));
    assert_eq!(Court::from_alias("rel-lisboa"), Some(Court::RelLisboa));
    assert_eq!(Court::from_alias("rel-coimbra"), Some(Court::RelCoimbra));
    assert_eq!(Court::from_alias("rel-guimaraes"), Some(Court::RelGuimaraes));
    assert_eq!(Court::from_alias("rel-evora"), Some(Court::RelEvora));
    assert_eq!(Court::from_alias("tca-sul"), Some(Court::TcaSul));
    assert_eq!(Court::from_alias("tca-norte"), Some(Court::TcaNorte));
    assert_eq!(Court::from_alias("unknown-court"), None);
    assert_eq!(Court::from_alias(""), None);
}

#[test]
fn court_search_url() {
    let url = Court::Stj.search_url("usucapiao", 50, 1, false);
    assert!(url.contains("dgsi.pt"), "URL should contain dgsi.pt: {url}");
    assert!(url.contains("jstj.nsf"), "URL should reference STJ db: {url}");
    assert!(url.contains("usucapiao"), "URL should contain encoded query: {url}");
    assert!(url.contains("Count=50"), "URL should include Count: {url}");
    assert!(url.contains("Start=1"), "URL should include Start: {url}");
    assert!(!url.contains("SearchOrder=1"), "Non-date sort should omit SearchOrder: {url}");

    let url_date = Court::Stj.search_url("usucapiao", 50, 1, true);
    assert!(
        url_date.contains("SearchOrder=1"),
        "Date sort should include SearchOrder=1: {url_date}"
    );
}

// ---------------------------------------------------------------------------
// build_query
// ---------------------------------------------------------------------------

#[test]
fn build_query_basic() {
    let q = build_query("usucapiao", None, None, None);
    assert_eq!(q, "usucapiao");
}

#[test]
fn build_query_with_dates() {
    let since = NaiveDate::from_ymd_opt(2020, 1, 15).unwrap();
    let until = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    let q = build_query("contrato", Some(since), Some(until), None);
    // Dates should appear in MM/DD/YYYY format per DGSI requirements
    assert!(q.contains("01/15/2020"), "since date missing or wrong format: {q}");
    assert!(q.contains("12/31/2023"), "until date missing or wrong format: {q}");
    assert!(q.contains("contrato"), "base query missing: {q}");
}

#[test]
fn build_query_with_field() {
    // field is passed as Option<(&str, &str)> — (field_name, value)
    let q = build_query("usucapiao", None, None, Some(("RELATOR", "MARIO CANCELA")));
    assert!(q.contains("RELATOR"), "Field name should appear in query: {q}");
    assert!(q.contains("usucapiao"), "Query text should appear: {q}");
}

// ---------------------------------------------------------------------------
// parse_search_results fixture
// ---------------------------------------------------------------------------

#[test]
fn parse_search_results_fixture() {
    let html = std::fs::read_to_string("tests/fixtures/dgsi_search_results.html").unwrap();
    let (total, results) = parse_search_results(&html, "jstj.nsf").unwrap();

    assert_eq!(total, 1000, "Expected 1000 total found");
    assert_eq!(results.len(), 5, "Expected 5 results in fixture");

    let first = &results[0];
    assert_eq!(first.relevance, 94, "First result relevance should be 94");
    assert_eq!(
        first.date,
        NaiveDate::from_ymd_opt(1994, 4, 21).unwrap(),
        "First result date should be 1994-04-21"
    );
    assert_eq!(first.processo, "084380", "First result processo should be 084380");
    assert_eq!(first.relator, "MARIO CANCELA", "First result relator mismatch");
    assert!(
        first.descriptors.iter().any(|d| d.contains("USUCAPI")),
        "Descriptors should include USUCAPIÃO: {:?}",
        first.descriptors
    );
}

// ---------------------------------------------------------------------------
// parse_decision fixture
// ---------------------------------------------------------------------------

#[test]
fn parse_decision_fixture() {
    let html = std::fs::read_to_string("tests/fixtures/dgsi_decision.html").unwrap();
    let url = "https://www.dgsi.pt/jstj.nsf/test";
    let decision = parse_decision(&html, url).unwrap();

    assert_eq!(decision.processo, "084380", "processo mismatch");
    assert_eq!(decision.relator, "MARIO CANCELA", "relator mismatch");
    assert!(
        decision.descritores.iter().any(|d| d.contains("USUCAPI")),
        "descritores should include USUCAPIÃO: {:?}",
        decision.descritores
    );
    assert_eq!(
        decision.data_acordao,
        Some(NaiveDate::from_ymd_opt(1994, 4, 21).unwrap()),
        "data_acordao should be 1994-04-21"
    );
    assert_eq!(decision.votacao, "UNANIMIDADE", "votacao mismatch");
    assert_eq!(decision.decisao, "NEGADA A REVISTA.", "decisao mismatch");
    assert!(
        decision.sumario.to_lowercase().contains("usucapi"),
        "sumario should mention usucapião: {}",
        decision.sumario
    );
}

// ---------------------------------------------------------------------------
// Renderable implementations
// ---------------------------------------------------------------------------

#[test]
fn search_result_to_markdown() {
    let html = std::fs::read_to_string("tests/fixtures/dgsi_search_results.html").unwrap();
    let (_total, results) = parse_search_results(&html, "jstj.nsf").unwrap();
    let first = &results[0];
    let md = first.to_markdown();

    assert!(md.contains("084380"), "Markdown should contain processo");
    assert!(md.contains("MARIO CANCELA"), "Markdown should contain relator");
    assert!(!md.is_empty(), "Markdown should not be empty");
}

#[test]
fn decision_to_markdown() {
    let html = std::fs::read_to_string("tests/fixtures/dgsi_decision.html").unwrap();
    let url = "https://www.dgsi.pt/jstj.nsf/test";
    let decision = parse_decision(&html, url).unwrap();
    let md = decision.to_markdown();

    assert!(md.contains("084380"), "Markdown should contain processo");
    assert!(md.contains("MARIO CANCELA"), "Markdown should contain relator");
    assert!(!md.is_empty(), "Markdown should not be empty");
}

#[test]
fn decision_to_json() {
    let html = std::fs::read_to_string("tests/fixtures/dgsi_decision.html").unwrap();
    let url = "https://www.dgsi.pt/jstj.nsf/test";
    let decision = parse_decision(&html, url).unwrap();
    let json = decision.to_json();

    assert!(json.is_object(), "JSON output should be an object");
    let obj = json.as_object().unwrap();

    // Check key fields are present
    let processo = obj.get("processo").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(processo, "084380", "JSON processo mismatch");

    let relator = obj.get("relator").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(relator, "MARIO CANCELA", "JSON relator mismatch");

    let decisao = obj.get("decisao").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(decisao, "NEGADA A REVISTA.", "JSON decisao mismatch");
}

// ---------------------------------------------------------------------------
// Court listing
// ---------------------------------------------------------------------------

#[test]
fn court_list_all() {
    assert_eq!(Court::all().len(), 10, "Should have exactly 10 courts");
}

// ---------------------------------------------------------------------------
// resolve_courts
// ---------------------------------------------------------------------------

#[test]
fn resolve_courts_empty() {
    let courts = lawyerr::dgsi::resolve_courts(&[]).unwrap();
    assert_eq!(courts.len(), 10, "Empty aliases should resolve to all 10 courts");
}

#[test]
fn resolve_courts_unknown() {
    let result = lawyerr::dgsi::resolve_courts(&["not-a-real-court".to_owned()]);
    assert!(result.is_err(), "Unknown alias should return an error");
}
