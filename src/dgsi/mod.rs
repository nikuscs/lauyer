pub mod courts;
pub mod decision;
pub mod markdown;
pub mod search;

use std::sync::Arc;

use courts::Court;
use decision::{DgsiDecision, parse_decision};
use search::{DgsiSearchResult, parse_search_results};

use crate::error::{LauyerError, Result};
use crate::http::HttpFetcher;

// Re-export for use from main.rs / tests
pub use search::build_query;

// ---------------------------------------------------------------------------
// Encoding helpers
// ---------------------------------------------------------------------------

/// Decode raw bytes as Latin-1 / Windows-1252 → UTF-8.
fn decode_latin1(bytes: &[u8]) -> String {
    let (cow, _encoding, _had_errors) = encoding_rs::WINDOWS_1252.decode(bytes);
    cow.into_owned()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Search a single court, auto-paginating until `limit` results are collected
/// or the server returns fewer results than requested.
///
/// If `delay_ms` is `Some(ms)`, sleep for `ms` milliseconds between page fetches.
pub async fn search_court(
    fetcher: &dyn HttpFetcher,
    court: Court,
    query: &str,
    limit: u32,
    sort_by_date: bool,
    delay_ms: Option<u64>,
) -> Result<(u64, Vec<DgsiSearchResult>)> {
    const PAGE_SIZE: u32 = 50;

    let mut results: Vec<DgsiSearchResult> = Vec::new();
    let mut total_found: u64 = 0;
    let mut start: u32 = 1;
    let mut is_first_page = true;

    loop {
        let remaining = limit.saturating_sub(results.len() as u32);
        if remaining == 0 {
            break;
        }
        let page_size = remaining.min(PAGE_SIZE);

        // Apply delay between page fetches (not before the first one).
        if !is_first_page {
            if let Some(ms) = delay_ms {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            }
        }
        is_first_page = false;

        let url = court.search_url(query, page_size, start, sort_by_date);
        // Search results pages are served as UTF-8 (charset=UTF-8 header).
        // Use get_text() which lets reqwest handle charset decoding.
        let html = fetcher.get_text(&url).await?;

        let (page_total, page_results) = parse_search_results(&html, court.db())
            .map_err(|e| LauyerError::Parse { message: e.to_string(), source_url: url.clone() })?;

        total_found = page_total;
        let page_len = page_results.len() as u32;
        results.extend(page_results);

        // Stop if the page was shorter than requested (last page) or we have enough
        if page_len < page_size || results.len() as u32 >= limit {
            break;
        }

        start += page_len;
    }

    results.truncate(limit as usize);
    Ok((total_found, results))
}

/// Search multiple courts concurrently, bounded by `max_concurrent`.
/// Results are returned in the same order as the input `courts` slice.
///
/// If `delay_ms` is `Some(ms)`, sleep for `ms` milliseconds between page
/// fetches within each court search.
pub async fn search_all_courts(
    fetcher: &dyn HttpFetcher,
    courts: &[Court],
    query: &str,
    limit: u32,
    sort_by_date: bool,
    max_concurrent: usize,
    delay_ms: Option<u64>,
) -> Vec<(Court, Result<(u64, Vec<DgsiSearchResult>)>)> {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));

    let futures: Vec<_> = courts
        .iter()
        .map(|&court| {
            let sem = Arc::clone(&semaphore);
            async move {
                let Ok(_permit) = sem.acquire().await else {
                    return (
                        court,
                        Err(LauyerError::Session {
                            message: "concurrency semaphore closed".to_owned(),
                        }),
                    );
                };
                let outcome =
                    search_court(fetcher, court, query, limit, sort_by_date, delay_ms).await;
                if let Err(ref e) = outcome {
                    tracing::warn!(court = court.alias(), error = %e, "Court search failed");
                }
                (court, outcome)
            }
        })
        .collect();

    futures::future::join_all(futures).await
}

/// Fetch and parse a single court decision from `url`.
pub async fn fetch_full_decision(fetcher: &dyn HttpFetcher, url: &str) -> Result<DgsiDecision> {
    // Decision pages are served as ISO-8859-1 (Latin-1). Fetch raw bytes
    // and decode manually.
    let bytes = fetcher.get(url).await?;
    let html = decode_latin1(&bytes);
    parse_decision(&html, url)
        .map_err(|e| LauyerError::Parse { message: e.to_string(), source_url: url.to_owned() })
}

/// Resolve a list of court aliases to `Court` values.
/// If `aliases` is empty, returns all known courts.
pub fn resolve_courts(aliases: &[String]) -> Result<Vec<Court>> {
    if aliases.is_empty() {
        return Ok(Court::all().to_vec());
    }
    aliases
        .iter()
        .map(|alias| {
            Court::from_alias(alias).ok_or_else(|| LauyerError::Config {
                message: format!("Unknown court alias: '{alias}'"),
            })
        })
        .collect()
}

/// Returns `(alias, display_name)` pairs for every known court.
pub fn list_courts() -> Vec<(String, String)> {
    Court::all().iter().map(|c| (c.alias().to_owned(), c.display_name().to_owned())).collect()
}
