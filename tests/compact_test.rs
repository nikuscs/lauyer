use lauyer::compact::{compact_text, strip_boilerplate, strip_html_tags, strip_stopwords};

#[test]
fn compact_collapses_blank_lines() {
    let input = "line one\n\n\n\nline two\n\n\nline three";
    let result = compact_text(input);
    assert_eq!(result, "line one\n\nline two\n\nline three");
}

#[test]
fn compact_strips_html_tags() {
    let input = "<b>Acórdão</b> do <em>tribunal</em>";
    let result = compact_text(input);
    assert_eq!(result, "Acórdão do tribunal");
}

#[test]
fn compact_collapses_spaces() {
    let input = "one   two\t\tthree";
    let result = compact_text(input);
    assert_eq!(result, "one two three");
}

#[test]
fn compact_trims_lines() {
    let input = "   leading\ntrailing   \n   both   ";
    let result = compact_text(input);
    assert_eq!(result, "leading\ntrailing\nboth");
}

#[test]
fn strip_html_nested() {
    assert_eq!(strip_html_tags("<div><p>text</p></div>"), "text");
}

#[test]
fn stop_words_removed() {
    let result = strip_stopwords("o contrato de trabalho");
    assert!(!result.split_whitespace().any(|w| w == "o"));
    assert!(!result.split_whitespace().any(|w| w == "de"));
    assert!(result.contains("contrato"));
    assert!(result.contains("trabalho"));
}

#[test]
fn stop_words_preserve_never_remove() {
    let result = strip_stopwords("não existe nenhum direito sem lei");
    for word in &["não", "nenhum", "sem"] {
        assert!(result.contains(word), "Expected '{word}' to be preserved in: {result}");
    }
}

#[test]
fn stop_words_case_insensitive_matching() {
    let result = strip_stopwords("O réu");
    assert!(
        !result.trim_start().starts_with("O "),
        "uppercase stop-word 'O' should be removed: {result}"
    );
    assert!(result.contains("réu"));
}

#[test]
fn stop_words_partial_words_kept() {
    let result = strip_stopwords("algumas umas palavras");
    assert!(result.contains("algumas"), "'algumas' must not be stripped: {result}");
}

#[test]
fn boilerplate_stripping() {
    let input =
        "Acordam no Tribunal da Relação de Lisboa os seguintes juízes\nSobre o mérito da causa";
    let result = strip_boilerplate(input);
    assert!(!result.contains("Acordam no Tribunal"));
    assert!(result.contains("Sobre o mérito"));
}
