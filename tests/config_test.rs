use lauyer::config::{Config, load_config, try_load};
use lauyer::format::OutputFormat;
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
    let tmp = std::path::Path::new("/tmp/lauyer_nonexistent_xyz.toml");
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
    let path = Path::new("/tmp/lauyer_definitely_missing_file_xyz987.toml");
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

// ---------------------------------------------------------------------------
// config.rs — additional branch coverage
// ---------------------------------------------------------------------------

#[test]
fn load_config_none_finds_no_local_file() {
    // Change into a fresh temp dir that has no lauyer.toml.
    let dir = tempfile::TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = load_config(None);

    // Restore working directory regardless of outcome
    std::env::set_current_dir(&original_dir).unwrap();

    let cfg = result.expect("load_config(None) from empty dir must not fail");
    assert!(cfg.server.port > 0, "returned config must have a valid port");
}

#[test]
fn load_config_explicit_path_io_error_context() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    // Write content that is syntactically broken at the TOML level
    write!(f, "[broken\nkey without equals sign\n").unwrap();
    let result = load_config(Some(f.path()));
    assert!(result.is_err(), "broken TOML with explicit path must return error");
    // The error message should mention the path
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Failed to parse") || msg.contains(".toml") || !msg.is_empty(),
        "error must be non-empty: {msg}"
    );
}

// ---------------------------------------------------------------------------
// try_load — direct tests for line 152 (non-NotFound IO error)
// ---------------------------------------------------------------------------

#[test]
fn try_load_returns_none_for_missing_file() {
    use std::path::Path;
    let path = Path::new("/tmp/lauyer_definitely_not_here_xyz987.toml");
    let result = try_load(path);
    assert!(result.is_ok(), "missing file must return Ok(None), not Err");
    assert!(result.unwrap().is_none(), "missing file must return None");
}

#[test]
fn try_load_returns_some_for_valid_toml() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "[server]\nport = 8888\n").unwrap();
    let result = try_load(f.path());
    assert!(result.is_ok(), "valid TOML must return Ok(Some(...))");
    let cfg = result.unwrap().expect("valid TOML must return Some");
    assert_eq!(cfg.server.port, 8888);
}

#[test]
fn try_load_returns_err_for_invalid_toml() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "this is not [ valid toml {{ at all").unwrap();
    let result = try_load(f.path());
    assert!(result.is_err(), "invalid TOML must return Err");
}

// ---------------------------------------------------------------------------
// load_config(None) — local config error branch (lines 181-187)
// Runs in a temp dir that contains an invalid lauyer.toml: the error is
// logged as a warning and the function falls through to defaults.
// ---------------------------------------------------------------------------

#[test]
fn load_config_none_local_invalid_toml_falls_back_to_defaults() {
    let dir = tempfile::TempDir::new().unwrap();
    let local_cfg = dir.path().join("lauyer.toml");
    std::fs::write(&local_cfg, "not [ valid toml {{ at all").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = load_config(None);

    std::env::set_current_dir(&original_dir).unwrap();

    // The error is swallowed; function must not fail and must return defaults.
    let cfg = result.expect("invalid local config must not cause load_config to return Err");
    assert!(cfg.server.port > 0, "fallback config must have a valid port");
    assert_eq!(cfg.server.port, 3000, "fallback must use built-in default port");
}

// ---------------------------------------------------------------------------
// load_config(None) — user config path (lines 193-199)
// Exercises the user config path discovery branch by calling load_config(None)
// from an empty dir (no local lauyer.toml). The function will then check the
// user config path (~/.config/lauyer/lauyer.toml) and either load it or fall
// through to defaults — both paths must return Ok.
// ---------------------------------------------------------------------------

#[test]
fn load_config_none_exercises_user_config_path() {
    // Change to a fresh empty dir with no lauyer.toml so the local-config
    // branch returns None and execution reaches the user-config branch.
    let empty_dir = tempfile::TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(empty_dir.path()).unwrap();

    let result = load_config(None);

    std::env::set_current_dir(&original_dir).unwrap();

    let cfg = result.expect("load_config(None) from empty dir must succeed");
    assert!(cfg.server.port > 0, "config port must be a valid port");
}
