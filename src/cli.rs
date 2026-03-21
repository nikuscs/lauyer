use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum SortOrder {
    #[default]
    Relevance,
    Date,
}

#[derive(Parser)]
#[command(
    name = "lauyer",
    about = "Fast CLI for searching Portuguese legal jurisprudence and legislation"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to config file
    #[arg(long, env = "LAUYER_CONFIG")]
    pub config: Option<PathBuf>,

    /// Proxy URL (e.g., socks5://host:port)
    #[arg(long)]
    pub proxy: Option<String>,

    /// Output format (auto-detected from --output extension if not set)
    #[arg(long, value_enum)]
    pub format: Option<crate::format::OutputFormat>,

    /// Write output to file
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Disable compact mode
    #[arg(long)]
    pub no_compact: bool,

    /// Enable stop word removal
    #[arg(long)]
    pub strip_stopwords: bool,

    /// Suppress progress output
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search and fetch DGSI court jurisprudence
    Dgsi {
        #[command(subcommand)]
        command: DgsiCommands,
    },
    /// Search and fetch Diário da República legislation
    Dr {
        #[command(subcommand)]
        command: DrCommands,
    },
    /// Start an HTTP server exposing the CLI as a REST API
    Serve(ServeArgs),
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum DgsiCommands {
    /// Search court decisions
    Search(DgsiSearchArgs),
    /// Fetch a single court decision by URL
    Fetch {
        /// URL of the court decision to fetch
        url: String,
    },
    /// List available courts
    Courts,
}

#[derive(Args)]
pub struct DgsiSearchArgs {
    /// Search query
    pub query: String,

    /// Filter by court (may be repeated)
    #[arg(long)]
    pub court: Vec<String>,

    /// Earliest date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Latest date (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,

    /// Relative recency window (e.g., "30d", "1y")
    #[arg(long)]
    pub recent: Option<String>,

    /// Maximum number of results to return
    #[arg(long, default_value_t = 50)]
    pub limit: u32,

    /// Sort order (relevance, date)
    #[arg(long, value_enum, default_value_t = SortOrder::Relevance)]
    pub sort: SortOrder,

    /// Filter field name for structured search
    #[arg(long)]
    pub field: Option<String>,

    /// Filter field value for structured search
    #[arg(long)]
    pub value: Option<String>,

    /// Fetch full text for each result
    #[arg(long)]
    pub fetch_full: bool,

    /// Maximum concurrent requests when fetching full text
    #[arg(long)]
    pub max_concurrent: Option<usize>,

    /// Delay between requests in milliseconds
    #[arg(long)]
    pub delay_ms: Option<u64>,
}

#[derive(Subcommand)]
pub enum DrCommands {
    /// Search Diário da República acts
    Search(DrSearchArgs),
    /// Show acts published today
    Today(DrTodayArgs),
    /// List available act types
    Types,
}

#[derive(Args)]
pub struct DrSearchArgs {
    /// Search query (full-text)
    pub query: Option<String>,

    /// Filter by act type (may be repeated; e.g., lei, decreto-lei)
    #[arg(long = "type")]
    pub act_type: Vec<String>,

    /// Filter by content type (may be repeated)
    #[arg(long)]
    pub content: Vec<String>,

    /// Earliest date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Latest date (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,

    /// Relative recency window (e.g., "30d", "1y")
    #[arg(long)]
    pub recent: Option<String>,

    /// Maximum number of results to return
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Args)]
pub struct DrTodayArgs {
    /// Filter by act type (may be repeated)
    #[arg(long = "type")]
    pub act_type: Vec<String>,
}

#[derive(Args)]
pub struct ServeArgs {
    /// Host address to bind to
    #[arg(long, default_value = "0.0.0.0", env = "LAUYER_HOST")]
    pub host: String,

    /// Port to listen on
    #[arg(long, default_value_t = 3000, env = "LAUYER_PORT")]
    pub port: u16,
}
