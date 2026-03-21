use std::collections::HashMap;

use chrono::NaiveDate;
use lauyer::dgsi::courts::Court;
use lauyer::dgsi::decision::{DgsiDecision, parse_decision};
use lauyer::dgsi::search::{DgsiSearchResult, build_query, parse_search_results};
use lauyer::format::Renderable;

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
    let courts = lauyer::dgsi::resolve_courts(&[]).unwrap();
    assert_eq!(courts.len(), 10, "Empty aliases should resolve to all 10 courts");
}

#[test]
fn resolve_courts_unknown() {
    let result = lauyer::dgsi::resolve_courts(&["not-a-real-court".to_owned()]);
    assert!(result.is_err(), "Unknown alias should return an error");
}

// ---------------------------------------------------------------------------
// DgsiSearchResult helpers
// ---------------------------------------------------------------------------

fn make_search_result() -> DgsiSearchResult {
    DgsiSearchResult {
        relevance: 87,
        date: NaiveDate::from_ymd_opt(2021, 6, 15).unwrap(),
        processo: "1234/21.0T8LSB".to_owned(),
        doc_url: "https://www.dgsi.pt/jstj.nsf/abc".to_owned(),
        doc_unid: "abc".to_owned(),
        relator: "JOÃO SILVA".to_owned(),
        descriptors: vec!["USUCAPIÃO".to_owned(), "POSSE".to_owned()],
    }
}

#[test]
fn search_result_to_json() {
    let r = make_search_result();
    let json = r.to_json();

    assert!(json.is_object(), "to_json must return an object");
    let obj = json.as_object().unwrap();

    assert_eq!(obj["processo"].as_str().unwrap(), "1234/21.0T8LSB");
    assert_eq!(obj["relator"].as_str().unwrap(), "JOÃO SILVA");
    assert_eq!(obj["relevance"].as_u64().unwrap(), 87);
    assert_eq!(obj["date"].as_str().unwrap(), "2021-06-15");
    assert_eq!(obj["url"].as_str().unwrap(), "https://www.dgsi.pt/jstj.nsf/abc");

    let descriptors = obj["descriptors"].as_array().unwrap();
    assert_eq!(descriptors.len(), 2);
    assert_eq!(descriptors[0].as_str().unwrap(), "USUCAPIÃO");
}

#[test]
fn search_result_table_row() {
    let r = make_search_result();
    let (headers, values) = r.table_row().expect("table_row must return Some");

    assert_eq!(headers, vec!["Date", "Processo", "Relator", "Descritores"]);
    assert_eq!(values[0], "2021-06-15");
    assert_eq!(values[1], "1234/21.0T8LSB");
    assert_eq!(values[2], "JOÃO SILVA");
    assert!(values[3].contains("USUCAPIÃO"), "Descritores value should contain USUCAPIÃO");
    assert!(values[3].contains("POSSE"), "Descritores value should contain POSSE");
}

// ---------------------------------------------------------------------------
// DgsiDecision helpers
// ---------------------------------------------------------------------------

fn make_decision() -> DgsiDecision {
    DgsiDecision {
        processo: "084380".to_owned(),
        relator: "MARIO CANCELA".to_owned(),
        descritores: vec!["USUCAPIÃO".to_owned(), "PRESCRIÇÃO AQUISITIVA".to_owned()],
        data_acordao: NaiveDate::from_ymd_opt(1994, 4, 21),
        votacao: "UNANIMIDADE".to_owned(),
        meio_processual: "REVISTA".to_owned(),
        decisao: "NEGADA A REVISTA.".to_owned(),
        sumario: "O prazo de usucapião é de 20 anos.".to_owned(),
        texto_integral: "Texto completo da decisão.".to_owned(),
        legislacao_nacional: String::new(),
        jurisprudencia_nacional: String::new(),
        doutrina: String::new(),
        url: "https://www.dgsi.pt/jstj.nsf/test".to_owned(),
        extra_fields: HashMap::new(),
    }
}

#[test]
fn decision_table_row() {
    let d = make_decision();
    let (headers, values) = d.table_row().expect("table_row must return Some");

    assert_eq!(headers, vec!["Date", "Processo", "Relator", "Descritores"]);
    assert_eq!(values[0], "1994-04-21");
    assert_eq!(values[1], "084380");
    assert_eq!(values[2], "MARIO CANCELA");
    assert!(values[3].contains("USUCAPIÃO"), "Descritores should contain USUCAPIÃO");
}

#[test]
fn decision_to_markdown_empty_fields() {
    let mut d = make_decision();
    d.sumario = String::new();
    d.texto_integral = String::new();

    let md = d.to_markdown();

    assert!(md.contains("084380"), "markdown must contain processo");
    assert!(!md.contains("## Sumário"), "empty sumario section must be omitted");
    assert!(!md.contains("## Texto Integral"), "empty texto_integral section must be omitted");
}

#[test]
fn decision_to_markdown_texto_integral_n_omitted() {
    let mut d = make_decision();
    d.texto_integral = "N".to_owned();

    let md = d.to_markdown();

    assert!(!md.contains("## Texto Integral"), "texto_integral='N' must be omitted");
}

#[test]
fn decision_to_json_with_extra_fields() {
    let mut d = make_decision();
    d.extra_fields.insert("Tribunal".to_owned(), "STJ".to_owned());
    d.extra_fields.insert("EmptyField".to_owned(), String::new());

    let json = d.to_json();
    let obj = json.as_object().unwrap();

    assert_eq!(obj["Tribunal"].as_str().unwrap(), "STJ", "extra_fields must appear in JSON");
    assert!(obj.get("EmptyField").is_none(), "empty extra_fields must be omitted");
}

// ---------------------------------------------------------------------------
// Court property invariants
// ---------------------------------------------------------------------------

#[test]
fn court_display_names() {
    for court in Court::all() {
        let name = court.display_name();
        assert!(!name.is_empty(), "display_name() must not be empty for {court:?}");
    }
}

#[test]
fn court_db_names() {
    for court in Court::all() {
        let db = court.db();
        // DGSI databases always use lowercase .nsf (Domino format); the
        // comparison is intentionally case-sensitive.
        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        let ends_nsf = db.ends_with(".nsf");
        assert!(ends_nsf, "db() must end with '.nsf' for {court:?}, got '{db}'");
    }
}

#[test]
fn court_view_unids() {
    for court in Court::all() {
        let unid = court.view_unid();
        let len = unid.len();
        assert_eq!(len, 32, "view_unid() must be 32 hex chars for {court:?}, got len {len}");
        assert!(
            unid.chars().all(|c| c.is_ascii_hexdigit()),
            "view_unid() must be all hex for {court:?}: '{unid}'"
        );
    }
}

#[test]
fn court_display_trait() {
    for court in Court::all() {
        let display = court.to_string();
        assert_eq!(
            display,
            court.display_name(),
            "Display impl must match display_name() for {court:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// parse_search_results — edge cases (search.rs uncovered branches)
// ---------------------------------------------------------------------------

#[test]
fn parse_search_results_zero_results() {
    let html = r"<html><body><h4>0 documents found</h4><table></table></body></html>";
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 0);
    assert!(results.is_empty());
}

#[test]
fn parse_search_results_malformed_rows() {
    // Row with only 3 <td> columns — should be skipped (lines 83-85)
    let html = r#"<html><body>
<h4>10 documents found</h4>
<table>
  <tr valign="top"><td>col1</td><td>col2</td><td>col3</td></tr>
</table>
</body></html>"#;
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 10);
    assert!(results.is_empty(), "malformed rows must be skipped");
}

#[test]
fn parse_search_results_bad_relevance() {
    // Row with 5 tds but img alt without '%' — should be skipped (lines 95-97)
    let html = r#"<html><body>
<h4>10 documents found</h4>
<table>
  <tr valign="top">
    <td><img alt="no-percent-here"/></td>
    <td><font>04/21/1994</font></td>
    <td><a href="/jstj.nsf/ABC">12345</a></td>
    <td><font>RELATOR</font></td>
    <td><font>DESC</font></td>
  </tr>
</table>
</body></html>"#;
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 10);
    assert!(results.is_empty(), "row with un-parseable relevance must be skipped");
}

#[test]
fn parse_search_results_bad_date() {
    // Row with valid relevance but unparseable date — should be skipped (lines 105-107)
    let html = r#"<html><body>
<h4>10 documents found</h4>
<table>
  <tr valign="top">
    <td><img alt="94%"/></td>
    <td><font>not-a-date</font></td>
    <td><a href="/jstj.nsf/ABC">12345</a></td>
    <td><font>RELATOR</font></td>
    <td><font>DESC</font></td>
  </tr>
</table>
</body></html>"#;
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 10);
    assert!(results.is_empty(), "row with bad date must be skipped");
}

#[test]
fn parse_search_results_no_anchor() {
    // Row with valid relevance and date but no <a> in processo cell — should be skipped (lines 113-115)
    let html = r#"<html><body>
<h4>10 documents found</h4>
<table>
  <tr valign="top">
    <td><img alt="94%"/></td>
    <td><font>04/21/1994</font></td>
    <td><font>no-link-here</font></td>
    <td><font>RELATOR</font></td>
    <td><font>DESC</font></td>
  </tr>
</table>
</body></html>"#;
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 10);
    assert!(results.is_empty(), "row without anchor must be skipped");
}

#[test]
fn parse_search_results_only_found_format() {
    // h4 with "N documents found" (no semicolon) — exercises the fallback branch (lines 170-175)
    let html = r"<html><body>
<h4>42 documents found</h4>
<table></table>
</body></html>";
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 42);
    assert!(results.is_empty());
}

#[test]
fn parse_search_results_returned_found_format() {
    // h4 with "N documents returned; M found" — exercises the semicolon branch (lines 177-183)
    let html = r"<html><body>
<h4>10 documents returned; 1000 found</h4>
<table></table>
</body></html>";
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 1000, "total should be the 'found' count after the semicolon");
    assert!(results.is_empty());
}

#[test]
fn parse_search_results_descriptors_with_br_and_tags() {
    // Descriptors cell with <br> variants and inner tags — exercises split_by_br + strip_html_tags
    let html = r#"<html><body>
<h4>1 documents found</h4>
<table>
  <tr valign="top">
    <td><img alt="94%"/></td>
    <td><font>04/21/1994</font></td>
    <td><a href="/jstj.nsf/ABC123">12345</a></td>
    <td><font>RELATOR NAME</font></td>
    <td><font><b>USUCAPIAO</b><br/>POSSE<br>AQUISICAO</font></td>
  </tr>
</table>
</body></html>"#;
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 1);
    assert_eq!(results.len(), 1);
    let r = &results[0];
    assert!(
        r.descriptors.iter().any(|d| d.contains("USUCAPIAO")),
        "USUCAPIAO should be a descriptor: {:?}",
        r.descriptors
    );
    assert!(
        r.descriptors.iter().any(|d| d.contains("POSSE")),
        "POSSE should be a descriptor: {:?}",
        r.descriptors
    );
    assert!(
        r.descriptors.iter().any(|d| d.contains("AQUISICAO")),
        "AQUISICAO should be a descriptor: {:?}",
        r.descriptors
    );
}

// ---------------------------------------------------------------------------
// parse_decision — edge cases (decision.rs uncovered branches)
// ---------------------------------------------------------------------------

#[test]
fn parse_decision_no_table() {
    // HTML without the expected decision table → LauyerError::Parse (lines 77-80)
    let html = r"<html><body><p>No table here</p></body></html>";
    let result = parse_decision(html, "https://example.com/test");
    assert!(result.is_err(), "missing decision table must return an error");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("decision table not found"),
        "error message should mention 'decision table not found': {err}"
    );
}

#[test]
fn parse_decision_empty_date() {
    // Decision where "Data do Acordão" value is empty → data_acordao is None (lines 212-213)
    let html = r##"<html><body>
<table width="100%" border="0">
  <tr valign="top">
    <td bgcolor="#71B2CF"><font color="#000080">Data do Acord&#227;o:</font></td>
    <td bgcolor="#E0F1FF"><font color="#000080"></font></td>
  </tr>
  <tr valign="top">
    <td bgcolor="#71B2CF"><font color="#000080">Processo:</font></td>
    <td bgcolor="#E0F1FF"><font color="#000080">99999</font></td>
  </tr>
</table>
</body></html>"##;
    let decision = parse_decision(html, "https://example.com/decision").unwrap();
    assert!(decision.data_acordao.is_none(), "empty date value should produce data_acordao = None");
    assert_eq!(decision.processo, "99999");
}

#[test]
fn decision_format_date_none_via_markdown() {
    // format_date(None) is pub(super); exercise it indirectly via to_markdown() (line 272)
    use lauyer::format::Renderable as _;
    let mut d = make_decision();
    d.data_acordao = None;
    let md = d.to_markdown();
    assert!(
        md.contains("(unknown date)"),
        "markdown of decision with no date should contain '(unknown date)': {md}"
    );
}

// ---------------------------------------------------------------------------
// LauyerError From<reqwest::Error> (error.rs lines 35-40)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn lauyer_error_from_reqwest_error() {
    use lauyer::error::LauyerError;

    // Sending to an invalid scheme triggers a reqwest error without real network I/O.
    let client = reqwest::Client::new();
    let err = client.get("not://invalid-scheme/path").send().await.unwrap_err();
    let lauyer_err = LauyerError::from(err);
    let msg = lauyer_err.to_string();
    assert!(
        msg.contains("HTTP error"),
        "LauyerError::from(reqwest::Error) should produce an Http variant: {msg}"
    );
}

// ---------------------------------------------------------------------------
// LauyerError::UserInput display (error.rs)
// ---------------------------------------------------------------------------

#[test]
fn user_input_error_display() {
    let err = lauyer::error::LauyerError::UserInput { message: "bad date".to_owned() };
    assert!(
        err.to_string().contains("bad date"),
        "UserInput display should contain the message, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// parse_search_results — additional branches for parse_total (search.rs)
// ---------------------------------------------------------------------------

#[test]
fn parse_search_results_no_h4() {
    let html = r"<html><body><table></table></body></html>";
    let (total, results) = parse_search_results(html, "test.nsf").unwrap();
    assert_eq!(total, 0, "no h4 element should yield total=0");
    assert!(results.is_empty());
}

/// h4 text is completely non-numeric (no leading digits) → parse error returned
#[test]
fn parse_search_results_unparseable_h4() {
    let html = r"<html><body><h4>gibberish no numbers</h4><table></table></body></html>";
    let result = parse_search_results(html, "test.nsf");
    assert!(result.is_err(), "h4 with no leading number should return a parse error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("parse") || err.contains("h4") || err.contains("total"),
        "error should mention the parse failure: {err}"
    );
}

/// h4 with semicolon but non-numeric "found" part → parse error from the semicolon branch
#[test]
fn parse_search_results_unparseable_found_part() {
    let html = r"<html><body><h4>10 documents returned; unparseable found</h4></body></html>";
    let result = parse_search_results(html, "test.nsf");
    assert!(result.is_err(), "non-numeric found-count after semicolon should return error");
}

// ---------------------------------------------------------------------------
// parse_decision — edge cases (decision.rs uncovered branches)
// ---------------------------------------------------------------------------

/// A <tr valign="top"> with only 1 <td> — skipped by the `cells.len() < 2` guard (line 84).
#[test]
fn parse_decision_single_cell_row() {
    let html = r##"<html><body>
<table width="100%" border="0">
  <tr valign="top">
    <td bgcolor="#71B2CF"><font color="#000080">OnlyCell:</font></td>
  </tr>
  <tr valign="top">
    <td bgcolor="#71B2CF"><font color="#000080">Processo:</font></td>
    <td bgcolor="#E0F1FF"><font color="#000080">12345</font></td>
  </tr>
</table>
</body></html>"##;
    let decision = parse_decision(html, "https://example.com/d").unwrap();
    // Single-cell row is silently skipped; the two-cell Processo row is parsed.
    assert_eq!(decision.processo, "12345");
}

/// A row where the label cell text is all whitespace → `label.is_empty()` guard (line 105-107).
#[test]
fn parse_decision_empty_label() {
    let html = r##"<html><body>
<table width="100%" border="0">
  <tr valign="top">
    <td bgcolor="#71B2CF"><font color="#000080">   </font></td>
    <td bgcolor="#E0F1FF"><font color="#000080">SomeValue</font></td>
  </tr>
  <tr valign="top">
    <td bgcolor="#71B2CF"><font color="#000080">Processo:</font></td>
    <td bgcolor="#E0F1FF"><font color="#000080">67890</font></td>
  </tr>
</table>
</body></html>"##;
    let decision = parse_decision(html, "https://example.com/d").unwrap();
    // The whitespace-label row is skipped; Processo is picked up correctly.
    assert_eq!(decision.processo, "67890");
    // "SomeValue" must NOT appear in any field (the empty-label row was dropped).
    let json = decision.to_json();
    let text = json.to_string();
    assert!(!text.contains("SomeValue"), "empty-label row must be skipped: {text}");
}
