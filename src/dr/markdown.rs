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
        })
    }

    fn table_row(&self) -> Option<(Vec<&str>, Vec<String>)> {
        let date_str = self
            .data_publicacao
            .map_or_else(|| "s/d".to_owned(), |d| d.format("%Y-%m-%d").to_string());

        Some((
            vec!["Date", "Tipo", "Número", "Emissor"],
            vec![date_str, self.tipo.clone(), self.numero.clone(), self.emissor.clone()],
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn sample_result() -> DrSearchResult {
        DrSearchResult {
            title: "Portaria n.º 122/2026/1".to_owned(),
            tipo: "Portaria".to_owned(),
            numero: "122/2026/1".to_owned(),
            data_publicacao: NaiveDate::from_ymd_opt(2026, 3, 20),
            emissor: "Economia e Coesão Territorial".to_owned(),
            sumario: "<p>Reconhece a <a href=\"#\">Associação</a> Empresarial de Águeda</p>"
                .to_owned(),
            serie: "I".to_owned(),
            db_id: "42".to_owned(),
        }
    }

    #[test]
    fn markdown_strips_html() {
        let md = sample_result().to_markdown();
        assert!(!md.contains('<'));
        assert!(!md.contains('>'));
        assert!(md.contains("Reconhece a Associação Empresarial de Águeda"));
    }

    #[test]
    fn markdown_format() {
        let md = sample_result().to_markdown();
        assert!(md.starts_with("### Portaria n.º 122/2026/1 (2026-03-20)"));
        assert!(md.contains("**Emissor:** Economia e Coesão Territorial"));
    }

    #[test]
    fn json_output() {
        let json = sample_result().to_json();
        assert_eq!(json["tipo"], "Portaria");
        assert_eq!(json["numero"], "122/2026/1");
        assert_eq!(json["data_publicacao"], "2026-03-20");
        // HTML should be stripped
        let sumario = json["sumario"].as_str().unwrap_or("");
        assert!(!sumario.contains('<'));
    }

    #[test]
    fn table_row_headers() {
        let result = sample_result();
        let (headers, values) = result.table_row().unwrap();
        assert_eq!(headers, vec!["Date", "Tipo", "Número", "Emissor"]);
        assert_eq!(values[0], "2026-03-20");
        assert_eq!(values[1], "Portaria");
    }
}
