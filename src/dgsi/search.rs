use std::fmt::Write as _;

use chrono::NaiveDate;
use scraper::{Html, Selector};
use tracing::warn;

use crate::error::{LauyerError, Result};

// ---------------------------------------------------------------------------
// Query builder
// ---------------------------------------------------------------------------

pub fn build_query(
    text: &str,
    since: Option<NaiveDate>,
    until: Option<NaiveDate>,
    field: Option<(&str, &str)>,
) -> String {
    let mut query = text.to_owned();

    if let Some(date) = since {
        let _ = write!(query, " AND [DATAAC] > {}", date.format("%m/%d/%Y"));
    }
    if let Some(date) = until {
        let _ = write!(query, " AND [DATAAC] < {}", date.format("%m/%d/%Y"));
    }
    if let Some((name, value)) = field {
        let _ = write!(query, " FIELD {name} contains {value}");
    }

    query
}

// ---------------------------------------------------------------------------
// Search result type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DgsiSearchResult {
    pub relevance: u8,
    pub date: NaiveDate,
    pub processo: String,
    pub doc_url: String,
    pub doc_unid: String,
    pub relator: String,
    pub descriptors: Vec<String>,
}

// ---------------------------------------------------------------------------
// HTML parser
// ---------------------------------------------------------------------------

pub fn parse_search_results(html: &str, base_db: &str) -> Result<(u64, Vec<DgsiSearchResult>)> {
    let document = Html::parse_document(html);

    let total = parse_total(&document, base_db)?;

    let row_sel = Selector::parse("tr[valign=\"top\"]").map_err(|e| LauyerError::Parse {
        message: e.to_string(),
        source_url: base_db.to_owned(),
    })?;
    let td_sel = Selector::parse("td").map_err(|e| LauyerError::Parse {
        message: e.to_string(),
        source_url: base_db.to_owned(),
    })?;
    let img_sel = Selector::parse("img").map_err(|e| LauyerError::Parse {
        message: e.to_string(),
        source_url: base_db.to_owned(),
    })?;
    let font_sel = Selector::parse("font").map_err(|e| LauyerError::Parse {
        message: e.to_string(),
        source_url: base_db.to_owned(),
    })?;
    let a_sel = Selector::parse("a").map_err(|e| LauyerError::Parse {
        message: e.to_string(),
        source_url: base_db.to_owned(),
    })?;

    let mut results = Vec::new();

    for row in document.select(&row_sel) {
        let tds: Vec<_> = row.select(&td_sel).collect();
        if tds.len() < 5 {
            warn!(expected = 5, got = tds.len(), "Skipping row with unexpected column count");
            continue;
        }

        // td[0]: relevance from <img alt="NN%">
        let relevance = tds[0]
            .select(&img_sel)
            .next()
            .and_then(|img| img.attr("alt"))
            .and_then(|alt| alt.strip_suffix('%'))
            .and_then(|n| n.parse::<u8>().ok());
        let Some(relevance) = relevance else {
            warn!("Skipping row: could not parse relevance");
            continue;
        };

        // td[1]: date from <font> text, format MM/DD/YYYY
        let date_str = tds[1].select(&font_sel).next().map(|f| f.text().collect::<String>());
        let date_str = date_str.as_deref().map_or("", str::trim).to_owned();
        let date = match NaiveDate::parse_from_str(&date_str, "%m/%d/%Y") {
            Ok(d) => d,
            Err(e) => {
                warn!(date = %date_str, error = %e, "Skipping row: could not parse date");
                continue;
            }
        };

        // td[2]: processo + doc_url + doc_unid
        let anchor = tds[2].select(&a_sel).next();
        let Some(anchor) = anchor else {
            warn!("Skipping row: no anchor in processo cell");
            continue;
        };
        let processo = anchor.text().collect::<String>().trim().to_owned();
        let href = anchor.attr("href").unwrap_or("").to_owned();
        let doc_url =
            if href.starts_with('/') { format!("https://www.dgsi.pt{href}") } else { href.clone() };
        // Extract UNID: last path segment before the '?'
        let path_part = href.split('?').next().unwrap_or(&href);
        let doc_unid = path_part.rsplit('/').next().unwrap_or("").to_owned();

        // td[3]: relator
        let relator = tds[3]
            .select(&font_sel)
            .next()
            .map(|f| f.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        // td[4]: descriptors — split inner HTML of <font> by <br> variants, strip tags
        let descriptors = tds[4]
            .select(&font_sel)
            .next()
            .map(|f| split_by_br(&f.inner_html()))
            .unwrap_or_default();

        results.push(DgsiSearchResult {
            relevance,
            date,
            processo,
            doc_url,
            doc_unid,
            relator,
            descriptors,
        });
    }

    Ok((total, results))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_total(document: &Html, source_url: &str) -> Result<u64> {
    let h4_sel = Selector::parse("h4").map_err(|e| LauyerError::Parse {
        message: e.to_string(),
        source_url: source_url.to_owned(),
    })?;

    let h4_text =
        document.select(&h4_sel).next().map(|el| el.text().collect::<String>()).unwrap_or_default();
    let h4_text = h4_text.trim();

    // "N documents returned; M found" → total is M
    // "N documents found"             → total is N
    h4_text.split(';').nth(1).map_or_else(
        || {
            let num_str = h4_text.split_whitespace().next().unwrap_or("0");
            num_str.parse::<u64>().map_err(|e| LauyerError::Parse {
                message: format!("could not parse total from h4 '{h4_text}': {e}"),
                source_url: source_url.to_owned(),
            })
        },
        |found_str| {
            let num_str = found_str.split_whitespace().next().unwrap_or("0");
            num_str.parse::<u64>().map_err(|e| LauyerError::Parse {
                message: format!("could not parse total from h4 '{h4_text}': {e}"),
                source_url: source_url.to_owned(),
            })
        },
    )
}

/// Split HTML inner content on `<br>` tag variants, strip remaining tags, and trim.
fn split_by_br(inner_html: &str) -> Vec<String> {
    let normalized =
        inner_html.replace("<br/>", "\n").replace("<br />", "\n").replace("<br>", "\n");

    normalized
        .split('\n')
        .map(|s| crate::compact::strip_html_tags(s).trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}
