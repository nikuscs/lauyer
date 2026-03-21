use std::sync::Mutex;

use lauyer::dgsi::courts::Court;
use lauyer::dgsi::{fetch_full_decision, resolve_courts, search_all_courts, search_court};
use lauyer::error::{LauyerError, Result};
use lauyer::http::HttpFetcher;

// ---------------------------------------------------------------------------
// MockHttpFetcher
// ---------------------------------------------------------------------------

// Responses are stored as `Ok(value)` or `Err(message_string)` so they can be
// cloned freely (LauyerError is not Clone).  Errors are materialised as
// `LauyerError::Session` at lookup time.
//
// Responses are matched by checking whether the requested URL *contains* one
// of the registered pattern keys (checked in insertion order).  The first
// matching entry wins, so more-specific patterns must be inserted before
// catch-all ones.
struct MockHttpFetcher {
    text_responses: Mutex<Vec<(String, std::result::Result<String, String>)>>,
    #[allow(clippy::type_complexity)]
    bytes_responses: Mutex<Vec<(String, std::result::Result<Vec<u8>, String>)>>,
}

impl MockHttpFetcher {
    #[allow(clippy::missing_const_for_fn)]
    fn new() -> Self {
        Self { text_responses: Mutex::new(Vec::new()), bytes_responses: Mutex::new(Vec::new()) }
    }

    fn add_text(&self, pattern: impl Into<String>, response: std::result::Result<String, String>) {
        self.text_responses.lock().unwrap().push((pattern.into(), response));
    }

    fn add_bytes(
        &self,
        pattern: impl Into<String>,
        response: std::result::Result<Vec<u8>, String>,
    ) {
        self.bytes_responses.lock().unwrap().push((pattern.into(), response));
    }
}

#[async_trait::async_trait]
impl HttpFetcher for MockHttpFetcher {
    async fn get(&self, url: &str) -> Result<Vec<u8>> {
        let responses = self.bytes_responses.lock().unwrap();
        for (pattern, resp) in responses.iter() {
            if url.contains(pattern.as_str()) {
                return resp.clone().map_err(|msg| LauyerError::Session { message: msg });
            }
        }
        drop(responses);
        Err(LauyerError::Session {
            message: format!("MockHttpFetcher: no bytes response registered for URL: {url}"),
        })
    }

    async fn get_text(&self, url: &str) -> Result<String> {
        let responses = self.text_responses.lock().unwrap();
        for (pattern, resp) in responses.iter() {
            if url.contains(pattern.as_str()) {
                return resp.clone().map_err(|msg| LauyerError::Session { message: msg });
            }
        }
        drop(responses);
        Err(LauyerError::Session {
            message: format!("MockHttpFetcher: no text response registered for URL: {url}"),
        })
    }

    async fn post_json(
        &self,
        url: &str,
        _body: &serde_json::Value,
        _headers: &[(String, String)],
    ) -> Result<String> {
        Err(LauyerError::Session {
            message: format!("MockHttpFetcher: post_json not configured for URL: {url}"),
        })
    }
}

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn search_results_html() -> String {
    std::fs::read_to_string("tests/fixtures/dgsi_search_results.html")
        .expect("dgsi_search_results.html fixture missing")
}

fn decision_html_bytes() -> Vec<u8> {
    // Latin-1 (Windows-1252) encoded version of the decision fixture.
    // fetch_full_decision feeds raw bytes through the Windows-1252 decoder,
    // so the fixture must already be encoded in that charset.
    std::fs::read("tests/fixtures/dgsi_decision_latin1.html")
        .expect("dgsi_decision_latin1.html fixture missing")
}

// Minimal two-result page used as the "last page" in pagination tests.
fn search_results_html_2() -> String {
    r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN">
<html><head><title>Search Results</title></head><body>
<h4>2 documents returned; 1000 found</h4><hr>
<table border="0" cellpadding="2" cellspacing="0">
<tr><th></th><th nowrap align="center"><b><font size="2">SESSÃO</font></b></th><th nowrap align="center"><b><font size="2">PROCESSO</font></b></th><th nowrap align="center"><b><font size="2">RELATOR</font></b></th><th nowrap align="left"><b><font size="2">DESCRITOR</font></b></th></tr>
<tr valign="top"><td align="center"><img src="/icons/vwicnsr2.gif" border="0" height="12" width="12" alt="90%"></td><td nowrap><font size="2">01/10/2010</font></td><td nowrap><font size="2"><a href="/jstj.nsf/abc123?OpenDocument">PAGE2-001</a></font></td><td nowrap><font size="2">RELATOR PAGE2</font></td><td nowrap><font size="2">DESCRITOR A</font></td></tr>
<tr valign="top"><td align="center"><img src="/icons/vwicnsr2.gif" border="0" height="12" width="12" alt="88%"></td><td nowrap><font size="2">02/15/2011</font></td><td nowrap><font size="2"><a href="/jstj.nsf/def456?OpenDocument">PAGE2-002</a></font></td><td nowrap><font size="2">RELATOR PAGE2 B</font></td><td nowrap><font size="2">DESCRITOR B</font></td></tr>
</table>
</body></html>"#.to_owned()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// 1. Single-page search returns all 5 results from the fixture.
#[tokio::test]
async fn search_court_single_page() {
    let mock = MockHttpFetcher::new();
    mock.add_text("jstj.nsf", Ok(search_results_html()));

    let (total, results) = search_court(&mock, Court::Stj, "usucapiao", 50, false, None)
        .await
        .expect("search_court should succeed");

    assert_eq!(total, 1000, "total found should be 1000");
    assert_eq!(results.len(), 5, "fixture contains 5 results");
    assert_eq!(results[0].processo, "084380");
    assert_eq!(results[0].relator, "MARIO CANCELA");
    assert_eq!(results[0].relevance, 94);
}

// 2. Limit is respected — even though the fixture has 5 results only 2 come back.
#[tokio::test]
async fn search_court_respects_limit() {
    let mock = MockHttpFetcher::new();
    mock.add_text("jstj.nsf", Ok(search_results_html()));

    let (total, results) = search_court(&mock, Court::Stj, "usucapiao", 2, false, None)
        .await
        .expect("search_court should succeed");

    assert_eq!(total, 1000);
    assert_eq!(results.len(), 2, "limit=2 should truncate to 2 results");
}

// 3. `search_all_courts` over two courts — both return results.
#[tokio::test]
async fn search_all_courts_multiple() {
    let mock = MockHttpFetcher::new();
    mock.add_text("jstj.nsf", Ok(search_results_html()));
    mock.add_text("jsta.nsf", Ok(search_results_html()));

    let courts = vec![Court::Stj, Court::Sta];
    let outcomes = search_all_courts(&mock, &courts, "usucapiao", 50, false, 2, None).await;

    assert_eq!(outcomes.len(), 2);
    for (_court, result) in &outcomes {
        let (_total, results) = result.as_ref().expect("each court should succeed");
        assert_eq!(results.len(), 5);
    }
}

// 4. `search_all_courts`: one court errors, the other succeeds — partial results returned.
#[tokio::test]
#[allow(clippy::similar_names)]
async fn search_all_courts_one_fails() {
    let mock = MockHttpFetcher::new();
    // STJ succeeds; STA has no registered response → mock returns an error.
    mock.add_text("jstj.nsf", Ok(search_results_html()));

    let courts = vec![Court::Stj, Court::Sta];
    let outcomes = search_all_courts(&mock, &courts, "usucapiao", 50, false, 2, None).await;

    assert_eq!(outcomes.len(), 2);

    let stj_outcome = outcomes.iter().find(|(c, _)| *c == Court::Stj).unwrap();
    let sta_outcome = outcomes.iter().find(|(c, _)| *c == Court::Sta).unwrap();

    assert!(stj_outcome.1.is_ok(), "STJ should succeed");
    assert!(sta_outcome.1.is_err(), "STA should fail (no mock registered)");
}

// 5. `fetch_full_decision` decodes bytes and returns a parsed decision.
#[tokio::test]
async fn fetch_full_decision_latin1() {
    let mock = MockHttpFetcher::new();
    let url = "https://www.dgsi.pt/jstj.nsf/test-decision";
    mock.add_bytes(url, Ok(decision_html_bytes()));

    let decision =
        fetch_full_decision(&mock, url).await.expect("fetch_full_decision should succeed");

    assert_eq!(decision.processo, "084380");
    assert_eq!(decision.relator, "MARIO CANCELA");
    assert_eq!(decision.votacao, "UNANIMIDADE");
    assert_eq!(decision.decisao, "NEGADA A REVISTA.");
    assert!(
        decision.sumario.to_lowercase().contains("usucapi"),
        "sumario should mention usucapião: {}",
        decision.sumario
    );
}

// 6. `resolve_courts` resolves known aliases correctly.
#[tokio::test]
async fn resolve_courts_valid_aliases() {
    let aliases = vec!["stj".to_owned(), "rel-porto".to_owned()];
    let courts = resolve_courts(&aliases).expect("valid aliases should resolve");

    assert_eq!(courts.len(), 2);
    assert!(courts.contains(&Court::Stj));
    assert!(courts.contains(&Court::RelPorto));
}

// 7. `search_court` paginates: first page full (5 results, Count=5), second
//     page short (2 results) → stops.  Results are then truncated to limit=5.
#[tokio::test]
async fn search_court_pagination() {
    let mock = MockHttpFetcher::new();

    // Register the page-2 pattern FIRST so its more-specific "Start=6" wins
    // over the general "jstj.nsf" fallback for the second request.
    mock.add_text("Start=6", Ok(search_results_html_2()));
    mock.add_text("jstj.nsf", Ok(search_results_html()));

    // With limit=5 the first request asks Count=5.  The fixture returns exactly
    // 5 results, so page_len(5) == page_size(5) → the loop continues with
    // Start=6.  The second response has only 2 entries, so page_len(2) <
    // page_size(5) → pagination stops.  Results are then truncated to limit=5.
    let (total, results) = search_court(&mock, Court::Stj, "usucapiao", 5, false, None)
        .await
        .expect("paginated search should succeed");

    assert_eq!(total, 1000, "total should be 1000 from the first page");
    // 5 from page 1 + 2 from page 2 = 7, but limit=5 → truncated to 5.
    assert_eq!(results.len(), 5, "limit=5 means exactly 5 results after truncation");
    // All 5 come from page 1 (fixture).
    assert_eq!(results[0].processo, "084380", "first result should be from page 1");
}
