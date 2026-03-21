use lawyerr::config::{Config, load_config};
use lawyerr::format::OutputFormat;
use std::io::Write as _;

#[test]
fn default_config_values() {
    let cfg = Config::default();

    assert!(cfg.http.proxy.is_none());
    assert_eq!(cfg.http.delay_ms, 100);
    assert_eq!(cfg.http.max_concurrent, 10);
    assert_eq!(cfg.http.timeout_secs, 30);
    assert_eq!(cfg.http.retries, 3);

    assert_eq!(cfg.output.format, OutputFormat::Markdown);
    assert!(cfg.output.compact);
    assert!(!cfg.output.strip_stopwords);

    assert_eq!(cfg.server.host, "0.0.0.0");
    assert_eq!(cfg.server.port, 3000);
}

#[test]
fn load_config_no_file_returns_error() {
    let tmp = std::path::Path::new("/tmp/lawyerr_nonexistent_xyz.toml");
    let result = load_config(Some(tmp));
    assert!(result.is_err(), "explicit path to missing file should return error");
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

    let cfg = load_config(Some(f.path())).unwrap();
    assert_eq!(cfg.server.port, 9090);
    assert_eq!(cfg.server.host, "127.0.0.1");
    assert_eq!(cfg.http.delay_ms, 250);
    assert_eq!(cfg.http.retries, 3);
}

#[test]
fn load_config_none_path_returns_defaults() {
    let cfg = load_config(None).unwrap();
    assert!(cfg.server.port > 0);
}

// ---------------------------------------------------------------------------
// config.rs uncovered branches
// ---------------------------------------------------------------------------

#[test]
fn load_config_invalid_toml() {
    // Write a file with invalid TOML — load_config must return an error for explicit paths
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "this is not [ valid toml {{ at all").unwrap();
    let result = load_config(Some(f.path()));
    assert!(result.is_err(), "invalid TOML with explicit path should return error");
}

#[test]
fn load_config_explicit_path_not_found_returns_error() {
    // Explicit path that does not exist → error
    use std::path::Path;
    let path = Path::new("/tmp/lawyerr_definitely_missing_file_xyz987.toml");
    let result = load_config(Some(path));
    assert!(result.is_err(), "missing explicit path should return error");
}

#[test]
fn load_config_explicit_path_valid_toml() {
    // Explicit path with valid TOML — verify fields are loaded
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
    let cfg = load_config(Some(f.path())).unwrap();
    assert_eq!(cfg.server.port, 7777);
    assert_eq!(cfg.http.delay_ms, 500);
    assert_eq!(cfg.http.retries, 1);
    // Unspecified fields should retain defaults
    assert_eq!(cfg.http.max_concurrent, 10);
}
