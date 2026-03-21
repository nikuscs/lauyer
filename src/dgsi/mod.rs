#![allow(clippy::unused_async)]

use crate::error::Result;
use crate::http::HttpFetcher;

pub async fn search(
    _fetcher: &dyn HttpFetcher,
    _courts: &[String],
    _query: &str,
) -> Result<Vec<String>> {
    Ok(vec![])
}

pub async fn fetch_decision(_fetcher: &dyn HttpFetcher, _url: &str) -> Result<String> {
    Ok(String::new())
}
