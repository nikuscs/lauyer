use std::fmt::Write as _;

use crate::format::Renderable;

use super::decision::{DgsiDecision, format_date};
use super::search::DgsiSearchResult;

// ---------------------------------------------------------------------------
// Renderable for DgsiSearchResult
// ---------------------------------------------------------------------------

impl Renderable for DgsiSearchResult {
    fn to_markdown(&self) -> String {
        let mut out = String::new();
        let date = self.date.to_string();
        let descriptors = self.descriptors.join(", ");
        let _ = write!(
            out,
            "### Processo {processo} ({date}) — Rel. {relator}\n\
             **Relevance:** {relevance}%\n\
             **Descritores:** {descriptors}",
            processo = self.processo,
            relator = self.relator,
            relevance = self.relevance,
        );
        out
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "processo": self.processo,
            "date": self.date.to_string(),
            "relator": self.relator,
            "relevance": self.relevance,
            "descriptors": self.descriptors,
            "url": self.doc_url,
        })
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        let headers = vec!["Date", "Processo", "Relator", "Descritores"];
        let values = vec![
            self.date.to_string(),
            self.processo.clone(),
            self.relator.clone(),
            self.descriptors.join(", "),
        ];
        Some((headers, values))
    }
}

// ---------------------------------------------------------------------------
// Renderable for DgsiDecision
// ---------------------------------------------------------------------------

impl Renderable for DgsiDecision {
    fn to_markdown(&self) -> String {
        let mut out = String::new();
        let date = format_date(self.data_acordao);

        let _ = write!(
            out,
            "# Processo {processo}\n\
             **Data:** {date} | **Relator:** {relator} | **Votação:** {votacao}",
            processo = self.processo,
            relator = self.relator,
            votacao = self.votacao,
        );

        push_section(&mut out, "Sumário", &self.sumario);
        push_section(&mut out, "Decisão", &self.decisao);

        if !self.texto_integral.is_empty() && self.texto_integral != "N" {
            push_section(&mut out, "Texto Integral", &self.texto_integral);
        }

        push_field(&mut out, "Meio Processual", &self.meio_processual);
        push_field(&mut out, "Descritores", &self.descritores.join(", "));
        push_field(&mut out, "Legislação Nacional", &self.legislacao_nacional);
        push_field(&mut out, "Jurisprudência Nacional", &self.jurisprudencia_nacional);
        push_field(&mut out, "Doutrina", &self.doutrina);

        out
    }

    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        insert_non_empty(&mut map, "processo", &self.processo);
        map.insert("date".to_owned(), serde_json::Value::String(format_date(self.data_acordao)));
        insert_non_empty(&mut map, "relator", &self.relator);
        insert_non_empty(&mut map, "votacao", &self.votacao);
        insert_non_empty(&mut map, "meio_processual", &self.meio_processual);
        insert_non_empty(&mut map, "decisao", &self.decisao);
        insert_non_empty(&mut map, "sumario", &self.sumario);

        if !self.texto_integral.is_empty() && self.texto_integral != "N" {
            insert_non_empty(&mut map, "texto_integral", &self.texto_integral);
        }

        insert_non_empty(&mut map, "legislacao_nacional", &self.legislacao_nacional);
        insert_non_empty(&mut map, "jurisprudencia_nacional", &self.jurisprudencia_nacional);
        insert_non_empty(&mut map, "doutrina", &self.doutrina);

        if !self.descritores.is_empty() {
            map.insert(
                "descritores".to_owned(),
                serde_json::Value::Array(
                    self.descritores.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
                ),
            );
        }

        insert_non_empty(&mut map, "url", &self.url);

        for (k, v) in &self.extra_fields {
            if !v.is_empty() {
                map.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
        }

        serde_json::Value::Object(map)
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        let headers = vec!["Date", "Processo", "Relator", "Descritores"];
        let values = vec![
            format_date(self.data_acordao),
            self.processo.clone(),
            self.relator.clone(),
            self.descritores.join(", "),
        ];
        Some((headers, values))
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn push_section(out: &mut String, heading: &str, content: &str) {
    if content.is_empty() {
        return;
    }
    let _ = write!(out, "\n\n## {heading}\n{content}");
}

fn push_field(out: &mut String, label: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    let _ = write!(out, "\n**{label}:** {value}");
}

fn insert_non_empty(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: &str) {
    if !value.is_empty() {
        map.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
    }
}
