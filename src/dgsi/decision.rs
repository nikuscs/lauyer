use std::collections::HashMap;
use std::fmt::Write as _;

use scraper::{Html, Selector};

use crate::error::{LauyerError, Result};

// ---------------------------------------------------------------------------
// DgsiDecision
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DgsiDecision {
    pub processo: String,
    pub relator: String,
    pub descritores: Vec<String>,
    pub data_acordao: Option<chrono::NaiveDate>,
    pub votacao: String,
    pub meio_processual: String,
    pub decisao: String,
    pub sumario: String,
    pub texto_integral: String,
    pub legislacao_nacional: String,
    pub jurisprudencia_nacional: String,
    pub doutrina: String,
    pub url: String,
    /// Any extra fields not explicitly modeled above.
    pub extra_fields: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// parse_decision
// ---------------------------------------------------------------------------

/// Parse a DGSI decision page into a [`DgsiDecision`].
///
/// The page contains a two-column `<table>` with label cells styled
/// `bgcolor="#71B2CF"` and value cells styled `bgcolor="#E0F1FF"`.
/// Separator rows have `bgcolor="#FFFFFF"` on both cells and are skipped.
pub fn parse_decision(html: &str, url: &str) -> Result<DgsiDecision> {
    let document = Html::parse_document(html);

    // Selectors — all CSS strings are known-valid at compile time.
    let sel_table =
        Selector::parse(r#"table[width="100%"][border="0"]"#).map_err(|e| LauyerError::Parse {
            message: format!("bad table selector: {e:?}"),
            source_url: url.to_owned(),
        })?;
    let sel_row = Selector::parse(r#"tr[valign="top"]"#).map_err(|e| LauyerError::Parse {
        message: format!("bad tr selector: {e:?}"),
        source_url: url.to_owned(),
    })?;
    let sel_cell = Selector::parse("td").map_err(|e| LauyerError::Parse {
        message: format!("bad td selector: {e:?}"),
        source_url: url.to_owned(),
    })?;
    let sel_font_blue =
        Selector::parse(r##"font[color="#000080"]"##).map_err(|e| LauyerError::Parse {
            message: format!("bad font selector: {e:?}"),
            source_url: url.to_owned(),
        })?;

    // Find the main data table — the one that contains label cells with
    // bgcolor="#71B2CF". Multiple tables may match width/border; we pick the
    // first one that has at least one such label cell.
    let sel_label_cell =
        Selector::parse(r##"td[bgcolor="#71B2CF"]"##).map_err(|e| LauyerError::Parse {
            message: format!("bad label cell selector: {e:?}"),
            source_url: url.to_owned(),
        })?;

    let table = document
        .select(&sel_table)
        .find(|t| t.select(&sel_label_cell).next().is_some())
        .ok_or_else(|| LauyerError::Parse {
            message: "decision table not found".to_owned(),
            source_url: url.to_owned(),
        })?;

    let mut fields: HashMap<String, String> = HashMap::new();

    for row in table.select(&sel_row) {
        let cells: Vec<_> = row.select(&sel_cell).collect();
        if cells.len() < 2 {
            continue;
        }
        let label_td = &cells[0];
        let value_td = &cells[1];

        // Skip separator rows — both cells have bgcolor="#FFFFFF".
        let label_bg = label_td.value().attr("bgcolor").unwrap_or("");
        let value_bg = value_td.value().attr("bgcolor").unwrap_or("");
        if label_bg == "#FFFFFF" && value_bg == "#FFFFFF" {
            continue;
        }

        // Label cell must be the blue header colour.
        if label_bg != "#71B2CF" {
            continue;
        }

        let label =
            label_td.text().collect::<String>().trim().trim_end_matches(':').trim().to_owned();

        if label.is_empty() {
            continue;
        }

        // Extract the value.  For fields that may contain <br> line breaks
        // (multi-line values like Sumário and Descritores) we convert the
        // inner HTML to a newline-separated string first, then strip tags.
        // For simple single-value fields we just collect text from the blue
        // <font> elements.
        let value = extract_value(value_td, &sel_font_blue);

        fields.insert(label, value);
    }

    Ok(build_decision(fields, url))
}

// ---------------------------------------------------------------------------
// Helper: extract the text value from a value cell
// ---------------------------------------------------------------------------

fn extract_value(value_td: &scraper::ElementRef<'_>, sel_font_blue: &Selector) -> String {
    // Collect all blue-font elements within this cell.
    let blue_fonts: Vec<_> = value_td.select(sel_font_blue).collect();

    if blue_fonts.is_empty() {
        // Fallback: plain text of the whole cell.
        return value_td.text().collect::<String>().trim().to_owned();
    }

    // Convert inner HTML of the first matching font to a newline-aware string.
    // We use the outer HTML of the element and process <br> tags.
    let inner_html = blue_fonts[0].inner_html();
    br_html_to_text(&inner_html).trim().to_owned()
}

// ---------------------------------------------------------------------------
// Helper: replace <br> / <br/> / <br /> with \n and strip remaining tags
// ---------------------------------------------------------------------------

fn br_html_to_text(html: &str) -> String {
    // Replace <br> variants with a newline marker then strip all remaining tags.
    let with_newlines = replace_br(html);
    crate::compact::strip_html_tags(&with_newlines)
}

fn replace_br(s: &str) -> String {
    // Character-level scan: replace <br…> with '\n'; discard all other tags.
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch != '<' {
            out.push(ch);
            continue;
        }
        // Consume the tag body up to '>'.
        let mut tag = String::new();
        for c in chars.by_ref() {
            if c == '>' {
                break;
            }
            tag.push(c);
        }
        let tag_lower = tag.trim().to_lowercase();
        if tag_lower == "br" || tag_lower == "br/" || tag_lower.starts_with("br ") {
            out.push('\n');
        }
        // All other tags are silently dropped.
    }
    out
}

// ---------------------------------------------------------------------------
// Helper: map parsed fields → DgsiDecision
// ---------------------------------------------------------------------------

fn build_decision(mut fields: HashMap<String, String>, url: &str) -> DgsiDecision {
    let processo = take_field(&mut fields, "Processo");
    let relator = take_field(&mut fields, "Relator");

    let descritores_raw = take_field(&mut fields, "Descritores");
    let descritores: Vec<String> = descritores_raw
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();

    let data_acordao = {
        let raw = take_field_any(&mut fields, &["Data do Acordão", "Data do Acordao"]);
        if raw.is_empty() {
            None
        } else {
            chrono::NaiveDate::parse_from_str(raw.trim(), "%m/%d/%Y").ok()
        }
    };

    let votacao = take_field_any(&mut fields, &["Votação", "Votacao"]);
    let meio_processual = take_field(&mut fields, "Meio Processual");
    let decisao = take_field_any(&mut fields, &["Decisão", "Decisao"]);
    let sumario = take_field_any(&mut fields, &["Sumário", "Sumario", "Sumário :", "Sumario :"]);
    let texto_integral = take_field(&mut fields, "Texto Integral");
    let legislacao_nacional =
        take_field_any(&mut fields, &["Legislação Nacional", "Legislacao Nacional"]);
    let jurisprudencia_nacional =
        take_field_any(&mut fields, &["Jurisprudência Nacional", "Jurisprudencia Nacional"]);
    let doutrina = take_field(&mut fields, "Doutrina");

    DgsiDecision {
        processo,
        relator,
        descritores,
        data_acordao,
        votacao,
        meio_processual,
        decisao,
        sumario,
        texto_integral,
        legislacao_nacional,
        jurisprudencia_nacional,
        doutrina,
        url: url.to_owned(),
        extra_fields: fields,
    }
}

fn take_field(fields: &mut HashMap<String, String>, key: &str) -> String {
    fields.remove(key).unwrap_or_default()
}

fn take_field_any(fields: &mut HashMap<String, String>, keys: &[&str]) -> String {
    for &key in keys {
        if let Some(v) = fields.remove(key) {
            return v;
        }
    }
    // Case-insensitive fallback.
    let key_lower_set: Vec<String> = keys.iter().map(|k| k.to_lowercase()).collect();
    let found_key =
        fields.keys().find(|k| key_lower_set.iter().any(|kl| kl == &k.to_lowercase())).cloned();
    found_key.and_then(|k| fields.remove(&k)).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Format helpers (used by markdown.rs)
// ---------------------------------------------------------------------------

/// Format `data_acordao` as `YYYY-MM-DD` or `"(unknown date)"` if absent.
pub(super) fn format_date(date: Option<chrono::NaiveDate>) -> String {
    date.map_or_else(
        || "(unknown date)".to_owned(),
        |d| {
            let mut s = String::new();
            let _ = write!(s, "{d}");
            s
        },
    )
}
