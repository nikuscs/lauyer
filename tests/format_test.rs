use std::path::Path;

use lauyer::format::{
    OutputFormat, Renderable, SearchResponse, format_from_extension, parse_recent, render,
    write_output,
};

// -----------------------------------------------------------------------
// OutputFormat
// -----------------------------------------------------------------------

#[test]
fn output_format_from_str() {
    assert_eq!("markdown".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
    assert_eq!("md".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
    assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    assert_eq!("table".parse::<OutputFormat>().unwrap(), OutputFormat::Table);
    assert!("xml".parse::<OutputFormat>().is_err());
}

#[test]
fn output_format_display() {
    assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
    assert_eq!(OutputFormat::Json.to_string(), "json");
    assert_eq!(OutputFormat::Table.to_string(), "table");
}

#[test]
fn output_format_default_is_markdown() {
    assert_eq!(OutputFormat::default(), OutputFormat::Markdown);
}

// -----------------------------------------------------------------------
// parse_recent
// -----------------------------------------------------------------------

#[test]
fn parse_recent_weeks() {
    let today = chrono::Local::now().date_naive();
    let result = parse_recent("1w").unwrap();
    assert_eq!(result, today - chrono::Duration::weeks(1));

    let result2 = parse_recent("2w").unwrap();
    assert_eq!(result2, today - chrono::Duration::weeks(2));
}

#[test]
fn parse_recent_months() {
    let result = parse_recent("1m");
    assert!(result.is_ok(), "1m failed: {result:?}");

    let result3 = parse_recent("3m");
    assert!(result3.is_ok(), "3m failed: {result3:?}");

    let result6 = parse_recent("6m");
    assert!(result6.is_ok(), "6m failed: {result6:?}");

    let today = chrono::Local::now().date_naive();
    assert!(result.unwrap() < today);
}

#[test]
fn parse_recent_year() {
    use chrono::Datelike as _;
    let today = chrono::Local::now().date_naive();
    let result = parse_recent("1y").unwrap();
    assert_eq!(result.year(), today.year() - 1);
}

#[test]
fn parse_recent_invalid_unit() {
    assert!(parse_recent("2d").is_err());
    assert!(parse_recent("abc").is_err());
    assert!(parse_recent("").is_err());
}

// -----------------------------------------------------------------------
// Rendering helpers
// -----------------------------------------------------------------------

struct DummyResult {
    title: String,
    body: String,
}

impl Renderable for DummyResult {
    fn to_markdown(&self) -> String {
        format!("## {}\n\n{}", self.title, self.body)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "title": self.title,
            "body": self.body,
        })
    }
}

fn make_response() -> SearchResponse {
    SearchResponse {
        source: "DGSI".to_owned(),
        query: "contrato".to_owned(),
        total: 1,
        results: vec![Box::new(DummyResult {
            title: "Acórdão 123".to_owned(),
            body: "Texto do acórdão".to_owned(),
        })],
    }
}

#[test]
fn render_markdown_contains_header() {
    let response = make_response();
    let out = render(&response, &OutputFormat::Markdown, false, false);
    assert!(out.contains("DGSI"));
    assert!(out.contains("contrato"));
    assert!(out.contains("Acórdão 123"));
}

#[test]
fn render_json_is_valid() {
    let response = make_response();
    let out = render(&response, &OutputFormat::Json, false, false);
    let v: serde_json::Value = serde_json::from_str(&out).expect("should be valid JSON");
    assert_eq!(v["source"], "DGSI");
    assert_eq!(v["total"], 1u64);
}

#[test]
fn render_table_smoke() {
    let response = make_response();
    let out = render(&response, &OutputFormat::Table, false, false);
    assert!(out.contains("DGSI"));
}

#[test]
fn write_output_to_file() {
    let f = tempfile::NamedTempFile::new().unwrap();
    write_output("hello", Some(f.path())).unwrap();
    let content = std::fs::read_to_string(f.path()).unwrap();
    assert_eq!(content, "hello");
}

// -----------------------------------------------------------------------
// format_from_extension
// -----------------------------------------------------------------------

#[test]
fn format_from_extension_json() {
    assert_eq!(format_from_extension(Path::new("out.json")), Some(OutputFormat::Json));
}

#[test]
fn format_from_extension_md() {
    assert_eq!(format_from_extension(Path::new("results.md")), Some(OutputFormat::Markdown));
}

#[test]
fn format_from_extension_unknown() {
    assert_eq!(format_from_extension(Path::new("results.txt")), None);
}

#[test]
fn format_from_extension_no_ext() {
    assert_eq!(format_from_extension(Path::new("results")), None);
}

// -----------------------------------------------------------------------
// Table rendering with table_row()
// -----------------------------------------------------------------------

struct StructuredResult {
    date: String,
    processo: String,
    relator: String,
    descritores: String,
}

impl Renderable for StructuredResult {
    fn to_markdown(&self) -> String {
        format!("## {} - {}", self.processo, self.date)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "date": self.date,
            "processo": self.processo,
        })
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        Some((
            vec!["Date", "Processo", "Relator", "Descritores"],
            vec![
                self.date.clone(),
                self.processo.clone(),
                self.relator.clone(),
                self.descritores.clone(),
            ],
        ))
    }
}

#[test]
fn render_table_with_structured_headers() {
    let response = SearchResponse {
        source: "STJ".to_owned(),
        query: "contrato".to_owned(),
        total: 1,
        results: vec![Box::new(StructuredResult {
            date: "2024-01-15".to_owned(),
            processo: "123/20".to_owned(),
            relator: "Silva".to_owned(),
            descritores: "Contrato, Nulidade".to_owned(),
        })],
    };
    let out = render(&response, &OutputFormat::Table, false, false);
    assert!(out.contains("Date"), "should have Date header");
    assert!(out.contains("Processo"), "should have Processo header");
    assert!(out.contains("Relator"), "should have Relator header");
    assert!(out.contains("Descritores"), "should have Descritores header");
    assert!(out.contains("123/20"), "should have processo value");
    assert!(out.contains("---"), "should have separator row");
}

#[test]
fn render_table_empty_results() {
    let response = SearchResponse {
        source: "STJ".to_owned(),
        query: "nada".to_owned(),
        total: 0,
        results: vec![],
    };
    let out = render(&response, &OutputFormat::Table, false, false);
    assert!(out.contains("(no results)"));
}

// -----------------------------------------------------------------------
// truncate — multi-byte character boundary adjustment (format.rs 154-161)
// -----------------------------------------------------------------------

// DummyResult without table_row() so the JSON-key fallback path is exercised.
struct JsonOnlyResult {
    label: String,
    value: String,
}

impl lauyer::format::Renderable for JsonOnlyResult {
    fn to_markdown(&self) -> String {
        format!("{}: {}", self.label, self.value)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "label": self.label,
            "value": self.value,
        })
    }
    // table_row() returns None (default), so build_table_data falls back to JSON keys
}

#[test]
fn render_table_json_key_fallback() {
    // Exercises build_table_data fallback branch (format.rs line 260)
    let response = SearchResponse {
        source: "TEST".to_owned(),
        query: "foo".to_owned(),
        total: 1,
        results: vec![Box::new(JsonOnlyResult {
            label: "hello".to_owned(),
            value: "world".to_owned(),
        })],
    };
    let out = render(&response, &OutputFormat::Table, false, false);
    assert!(out.contains("label") || out.contains("value"), "JSON keys should appear as headers");
    assert!(out.contains("hello") || out.contains("world"), "JSON values should appear in table");
}

#[test]
fn truncate_multibyte_char_boundary() {
    // "Acórdão" — 'ó' is 2 bytes, 'ã' is 2 bytes.
    // Build a string longer than MAX_COL_WIDTH (50) so truncate is triggered,
    // ending near a multi-byte char boundary.
    // format.rs lines 154-161: truncate adjusts end to avoid splitting a char.
    let s = "Acórdão do Supremo Tribunal de Justiça — Processo número 12345/20";
    // Call render_table with a result that has this long value, which goes through truncate().
    let response = SearchResponse {
        source: "TEST".to_owned(),
        query: "foo".to_owned(),
        total: 1,
        results: vec![Box::new(JsonOnlyResult { label: "desc".to_owned(), value: s.to_owned() })],
    };
    let out = render(&response, &OutputFormat::Table, false, false);
    // The value must be truncated with "..." and must be valid UTF-8 (no panic = success).
    assert!(out.contains("..."), "long value should be truncated with '...'");
    // Verify the output is valid UTF-8 (it's a String, so this always holds, but the
    // real check is that we didn't panic due to char boundary slicing).
    assert!(std::str::from_utf8(out.as_bytes()).is_ok());
}

// -----------------------------------------------------------------------
// format_from_extension — unknown / no-extension cases (format.rs line 283)
// -----------------------------------------------------------------------

#[test]
fn format_from_extension_table_not_inferred() {
    // "table" is not an extension — returns None (exercises the `_ => None` arm)
    assert_eq!(format_from_extension(Path::new("out.csv")), None);
    assert_eq!(format_from_extension(Path::new("out.xml")), None);
    assert_eq!(format_from_extension(Path::new("out.txt")), None);
}

// -----------------------------------------------------------------------
// parse_recent — additional edge cases (format.rs lines 346, 357)
// -----------------------------------------------------------------------

#[test]
fn parse_recent_empty_string() {
    // Empty string → error on is_empty check (line 320-322)
    assert!(parse_recent("").is_err());
}

#[test]
fn parse_recent_non_numeric() {
    // Non-numeric prefix → parse::<i64> fails (line 327)
    assert!(parse_recent("xw").is_err());
    assert!(parse_recent("abcm").is_err());
}

#[test]
fn parse_recent_year_leap_day() {
    // 1y from a non-Feb-29 date should always succeed.
    // This exercises the year path (lines 351-359).
    let result = parse_recent("1y");
    assert!(result.is_ok(), "1y should always succeed: {result:?}");
}
