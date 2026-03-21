use crate::compact::strip_html_tags;
use crate::format::Renderable;

use super::search::DrSearchResult;

impl Renderable for DrSearchResult {
    fn to_markdown(&self) -> String {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        let clean_sumario = strip_html_tags(&self.sumario);

        format!(
            "### {} n.º {} ({})\n**Emissor:** {}\n**Sumário:** {}",
            self.tipo, self.numero, date_str, self.emissor, clean_sumario
        )
    }

    fn to_json(&self) -> serde_json::Value {
        let date_str = self.data_publicacao.map_or_else(
            || serde_json::Value::Null,
            |d| serde_json::Value::String(d.format("%Y-%m-%d").to_string()),
        );

        serde_json::json!({
            "title": self.title,
            "tipo": self.tipo,
            "numero": self.numero,
            "data_publicacao": date_str,
            "emissor": self.emissor,
            "sumario": strip_html_tags(&self.sumario),
            "serie": self.serie,
            "db_id": self.db_id,
            "file_id": self.file_id,
            "tipo_conteudo": self.tipo_conteudo,
            "ano": self.ano,
        })
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        let clean_sumario = strip_html_tags(&self.sumario);
        let truncated_sumario = if clean_sumario.len() > 60 {
            let mut end = 57;
            while end > 0 && !clean_sumario.is_char_boundary(end) {
                end -= 1;
            }
            let mut s = clean_sumario[..end].to_owned();
            s.push_str("...");
            s
        } else {
            clean_sumario
        };

        Some((
            vec!["Date", "Tipo", "Número", "Emissor", "Sumário"],
            vec![
                date_str,
                self.tipo.clone(),
                self.numero.clone(),
                self.emissor.clone(),
                truncated_sumario,
            ],
        ))
    }
}
