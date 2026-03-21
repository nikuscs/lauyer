use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::format::OutputFormat;

// ---------------------------------------------------------------------------
// Sub-configurations
// ---------------------------------------------------------------------------

const fn default_delay_ms() -> u64 {
    100
}
const fn default_max_concurrent() -> usize {
    10
}
const fn default_timeout_secs() -> u64 {
    30
}
const fn default_retries() -> u32 {
    3
}

#[derive(Debug, Deserialize)]
pub struct HttpConfig {
    #[serde(default)]
    pub proxy: Option<String>,

    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,

    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    #[serde(default = "default_retries")]
    pub retries: u32,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            delay_ms: default_delay_ms(),
            max_concurrent: default_max_concurrent(),
            timeout_secs: default_timeout_secs(),
            retries: default_retries(),
        }
    }
}

const fn default_format() -> OutputFormat {
    OutputFormat::Markdown
}
const fn default_compact() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: OutputFormat,

    #[serde(default = "default_compact")]
    pub compact: bool,

    #[serde(default)]
    pub strip_stopwords: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { format: default_format(), compact: default_compact(), strip_stopwords: false }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}
const fn default_port() -> u16 {
    3000
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { host: default_host(), port: default_port() }
    }
}

// ---------------------------------------------------------------------------
// Top-level Config
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub http: HttpConfig,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub server: ServerConfig,
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

/// Return the user-level config path (`~/.config/lauyer/lauyer.toml`).
fn user_config_path() -> Option<PathBuf> {
    dirs_home().map(|h| h.join(".config").join("lauyer").join("lauyer.toml"))
}

/// Minimal home-dir detection without pulling in the `dirs` crate.
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Try to read and deserialize a TOML config file.  Returns `None` when the
/// file does not exist; propagates I/O or parse errors as `anyhow::Error`.
pub fn try_load(path: &Path) -> anyhow::Result<Option<Config>> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let cfg: Config = toml::from_str(&contents)
                .map_err(|e| anyhow::anyhow!("Failed to parse {}: {e}", path.display()))?;
            Ok(Some(cfg))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow::Error::from(e)),
    }
}

/// Load configuration.
///
/// Resolution order:
/// 1. `path` (if supplied) -- errors are fatal for explicit paths
/// 2. `./lauyer.toml`
/// 3. `~/.config/lauyer/lauyer.toml`
/// 4. Compiled-in defaults
pub fn load_config(path: Option<&Path>) -> anyhow::Result<Config> {
    if let Some(explicit) = path {
        match try_load(explicit) {
            Ok(Some(cfg)) => {
                tracing::info!(path = %explicit.display(), "Loaded config from explicit path");
                return Ok(cfg);
            }
            Ok(None) => {
                return Err(anyhow::anyhow!("Config file not found: {}", explicit.display()));
            }
            Err(e) => {
                return Err(e.context(format!("Failed to load config from {}", explicit.display())));
            }
        }
    }

    let local = Path::new("lauyer.toml");
    match try_load(local) {
        Ok(Some(cfg)) => {
            tracing::info!(path = %local.display(), "Loaded config");
            return Ok(cfg);
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(error = %e, "Error reading local config, skipping");
        }
    }

    if let Some(user_path) = user_config_path() {
        match try_load(&user_path) {
            Ok(Some(cfg)) => {
                tracing::info!(path = %user_path.display(), "Loaded config");
                return Ok(cfg);
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, "Error reading user config, skipping");
            }
        }
    }

    tracing::debug!("No config file found, using built-in defaults");
    Ok(Config::default())
}
