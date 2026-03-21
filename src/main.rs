use anyhow::Context as _;
use clap::Parser as _;
use lawyerr::{cli, config, dgsi, dr, http, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = cli::Cli::parse();
    let cfg = config::load_config(cli.config.as_deref());

    match cli.command {
        cli::Commands::Dgsi { command } => match command {
            cli::DgsiCommands::Search(args) => {
                let fetcher = http::HttpClient::new(
                    cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                    cfg.http.timeout_secs,
                    cfg.http.retries,
                )
                .context("Failed to build HTTP client")?;
                let results = dgsi::search(&fetcher, &args.court, &args.query).await?;
                tracing::info!(count = results.len(), "dgsi search: not fully implemented yet");
            }
            cli::DgsiCommands::Fetch { url } => {
                let fetcher = http::HttpClient::new(
                    cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                    cfg.http.timeout_secs,
                    cfg.http.retries,
                )
                .context("Failed to build HTTP client")?;
                let _result = dgsi::fetch_decision(&fetcher, &url).await?;
                tracing::info!("dgsi fetch: not fully implemented yet");
            }
            cli::DgsiCommands::Courts => {
                tracing::info!("dgsi courts: not fully implemented yet");
            }
        },
        cli::Commands::Dr { command } => match command {
            cli::DrCommands::Search(_args) => {
                let _results = dr::search().await?;
                tracing::info!("dr search: not fully implemented yet");
            }
            cli::DrCommands::Fetch { url: _ } => {
                tracing::info!("dr fetch: not fully implemented yet");
            }
            cli::DrCommands::Today(_args) => {
                tracing::info!("dr today: not fully implemented yet");
            }
            cli::DrCommands::Types => {
                tracing::info!("dr types: not fully implemented yet");
            }
        },
        cli::Commands::Serve(args) => {
            server::start(&args.host, args.port).await.context("Server failed")?;
        }
    }

    Ok(())
}
