#![allow(dead_code)]

use std::fmt;
use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::str::FromStr;

use crate::compact::{compact_text, strip_stopwords};

// ---------------------------------------------------------------------------
// OutputFormat
// ---------------------------------------------------------------------------

#[derive(
    Debug, Clone, Default, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
    Table,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Markdown => f.write_str("markdown"),
            Self::Json => f.write_str("json"),
            Self::Table => f.write_str("table"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "table" => Ok(Self::Table),
            other => Err(format!("Unknown output format: '{other}'")),
        }
    }
}

// ---------------------------------------------------------------------------
// Renderable trait
// ---------------------------------------------------------------------------

pub trait Renderable {
    fn to_markdown(&self) -> String;
    fn to_json(&self) -> serde_json::Value;
}

// ---------------------------------------------------------------------------
// SearchResponse
// ---------------------------------------------------------------------------

pub struct SearchResponse {
    pub source: String,
    pub query: String,
    pub total: u64,
    pub results: Vec<Box<dyn Renderable>>,
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Render a `SearchResponse` to a string according to the requested format.
///
/// Post-processing pipeline (applied to each result's text):
/// 1. `compact_text` — if `compact` is `true`
/// 2. `strip_stopwords` — if `strip_sw` is `true`
pub fn render(
    response: &SearchResponse,
    format: &OutputFormat,
    compact: bool,
    strip_sw: bool,
) -> String {
    match format {
        OutputFormat::Markdown => render_markdown(response, compact, strip_sw),
        OutputFormat::Json => render_json(response, compact, strip_sw),
        OutputFormat::Table => render_table(response, compact, strip_sw),
    }
}

fn post_process(text: String, compact: bool, strip_sw: bool) -> String {
    let text = if compact { compact_text(&text) } else { text };
    if strip_sw { strip_stopwords(&text) } else { text }
}

fn render_markdown(response: &SearchResponse, compact: bool, strip_sw: bool) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "# {} — {}\n\n_{} results_\n\n",
        response.source, response.query, response.total
    );
    for result in &response.results {
        let text = post_process(result.to_markdown(), compact, strip_sw);
        out.push_str(&text);
        out.push_str("\n\n---\n\n");
    }
    out
}

fn render_json(response: &SearchResponse, compact: bool, strip_sw: bool) -> String {
    let results: Vec<serde_json::Value> = response
        .results
        .iter()
        .map(|r| {
            let mut v = r.to_json();
            if let Some(obj) = v.as_object_mut() {
                for val in obj.values_mut() {
                    if let Some(s) = val.as_str() {
                        *val = serde_json::Value::String(post_process(
                            s.to_owned(),
                            compact,
                            strip_sw,
                        ));
                    }
                }
            }
            v
        })
        .collect();

    let wrapper = serde_json::json!({
        "source": response.source,
        "query": response.query,
        "total": response.total,
        "results": results,
    });

    serde_json::to_string_pretty(&wrapper).unwrap_or_else(|_| "{}".to_owned())
}

fn render_table(response: &SearchResponse, compact: bool, strip_sw: bool) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();

    for result in &response.results {
        let v = result.to_json();
        if let Some(obj) = v.as_object() {
            let row: Vec<String> = obj
                .values()
                .map(|val| {
                    let s = match val {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    post_process(s, compact, strip_sw)
                })
                .collect();
            rows.push(row);
        }
    }

    if rows.is_empty() {
        return format!(
            "Source: {}\nQuery:  {}\nTotal:  {}\n\n(no results)\n",
            response.source, response.query, response.total
        );
    }

    let ncols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0usize; ncols];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }

    let mut out = format!(
        "Source: {}\nQuery:  {}\nTotal:  {}\n\n",
        response.source, response.query, response.total
    );

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or(cell.len());
            let _ = write!(out, "{cell:<w$}  ");
        }
        out.push('\n');
    }

    out
}

// ---------------------------------------------------------------------------
// write_output
// ---------------------------------------------------------------------------

/// Write `content` to stdout or to a file if `output_path` is provided.
pub fn write_output(content: &str, output_path: Option<&std::path::Path>) -> std::io::Result<()> {
    match output_path {
        Some(path) => {
            let mut f = std::fs::File::create(path)?;
            f.write_all(content.as_bytes())
        }
        None => std::io::stdout().write_all(content.as_bytes()),
    }
}

// ---------------------------------------------------------------------------
// DateRange
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct DateRange {
    pub since: Option<chrono::NaiveDate>,
    pub until: Option<chrono::NaiveDate>,
}

// ---------------------------------------------------------------------------
// parse_recent
// ---------------------------------------------------------------------------

/// Parse a "recent" shorthand string into an absolute `NaiveDate`.
///
/// Supported values: `1w`, `2w`, `1m`, `3m`, `6m`, `1y`.
/// The returned date is today minus the indicated duration.
pub fn parse_recent(s: &str) -> Result<chrono::NaiveDate, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(format!("Empty time specification: '{s}'"));
    }

    let unit = &trimmed[trimmed.len() - 1..];
    let num_str = &trimmed[..trimmed.len() - 1];

    let n: i64 = num_str.parse().map_err(|_| format!("Cannot parse number from '{s}'"))?;

    let today = chrono::Local::now().date_naive();

    match unit {
        "w" => today
            .checked_sub_signed(chrono::Duration::weeks(n))
            .ok_or_else(|| format!("Date out of range for '{s}'")),
        "m" => {
            use chrono::Datelike as _;
            let total_months = i64::from(today.year()) * 12 + i64::from(today.month0()) - n;
            let year = i32::try_from(total_months.div_euclid(12))
                .map_err(|_| "Year out of i32 range".to_owned())?;
            let month = u32::try_from(total_months.rem_euclid(12))
                .map_err(|_| "Month out of range".to_owned())?
                + 1;
            // Try the exact day first; clamp to end-of-month on failure.
            chrono::NaiveDate::from_ymd_opt(year, month, today.day())
                .or_else(|| {
                    chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).and_then(|d| d.pred_opt())
                })
                .ok_or_else(|| format!("Invalid resulting date for '{s}'"))
        }
        "y" => {
            use chrono::Datelike as _;
            let n32 = i32::try_from(n).map_err(|_| "Year offset out of i32 range".to_owned())?;
            let year = today.year() - n32;
            chrono::NaiveDate::from_ymd_opt(year, today.month(), today.day())
                .or_else(|| {
                    // Feb 29 on a non-leap year → Feb 28
                    chrono::NaiveDate::from_ymd_opt(year, today.month(), today.day() - 1)
                })
                .ok_or_else(|| format!("Invalid resulting date for '{s}'"))
        }
        other => Err(format!("Unknown time unit '{other}' in '{s}'")),
    }
}
