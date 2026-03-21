use lawyerr::config::{Config, load_config};
use std::io::Write as _;

#[test]
fn default_config_values() {
    let cfg = Config::default();

    assert!(cfg.dgsi.courts.is_empty());

    assert_eq!(cfg.dr.content_types, vec!["atos-1", "atos-2", "decisoes"]);
    assert!(cfg.dr.act_types.is_empty());

    assert!(cfg.http.proxy.is_none());
    assert_eq!(cfg.http.delay_ms, 100);
    assert_eq!(cfg.http.max_concurrent, 10);
    assert_eq!(cfg.http.timeout_secs, 30);
    assert_eq!(cfg.http.retries, 3);

    assert_eq!(cfg.output.format, "markdown");
    assert!(cfg.output.compact);
    assert!(!cfg.output.strip_stopwords);

    assert_eq!(cfg.server.host, "0.0.0.0");
    assert_eq!(cfg.server.port, 3000);
}

#[test]
fn load_config_no_file_returns_defaults() {
    let tmp = std::path::Path::new("/tmp/lawyerr_nonexistent_xyz.toml");
    let cfg = load_config(Some(tmp));
    assert_eq!(cfg.server.port, 3000);
}

#[test]
fn load_config_parses_toml() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"
[server]
port = 9090
host = "127.0.0.1"

[http]
delay_ms = 250
"#
    )
    .unwrap();

    let cfg = load_config(Some(f.path()));
    assert_eq!(cfg.server.port, 9090);
    assert_eq!(cfg.server.host, "127.0.0.1");
    assert_eq!(cfg.http.delay_ms, 250);
    assert_eq!(cfg.http.retries, 3);
}

#[test]
fn load_config_none_path_returns_defaults() {
    let cfg = load_config(None);
    assert!(cfg.server.port > 0);
}

// ---------------------------------------------------------------------------
// config.rs uncovered branches
// ---------------------------------------------------------------------------

#[test]
fn load_config_invalid_toml() {
    // Write a file with invalid TOML — load_config must return defaults (lines 197-199)
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "this is not [ valid toml {{ at all").unwrap();
    let cfg = load_config(Some(f.path()));
    // Should not panic; must fall back to built-in defaults.
    assert_eq!(cfg.server.port, 3000, "invalid TOML should fall back to default port");
    assert_eq!(cfg.http.delay_ms, 100, "invalid TOML should fall back to default delay_ms");
}

#[test]
fn load_config_explicit_path_not_found_returns_defaults() {
    // Explicit path that does not exist → Ok(None) branch (lines 190-195)
    use std::path::Path;
    let path = Path::new("/tmp/lawyerr_definitely_missing_file_xyz987.toml");
    let cfg = load_config(Some(path));
    assert_eq!(cfg.server.port, 3000, "missing explicit path should fall back to default port");
}

#[test]
fn load_config_explicit_path_valid_toml() {
    // Explicit path with valid TOML — verify fields are loaded (lines 186-188)
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r"
[server]
port = 7777

[http]
delay_ms = 500
retries = 1
"
    )
    .unwrap();
    let cfg = load_config(Some(f.path()));
    assert_eq!(cfg.server.port, 7777);
    assert_eq!(cfg.http.delay_ms, 500);
    assert_eq!(cfg.http.retries, 1);
    // Unspecified fields should retain defaults
    assert_eq!(cfg.http.max_concurrent, 10);
}
