use std::fmt::Write as FmtWrite;
use std::io::Write as _;
use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser as _;
use futures::stream::{FuturesUnordered, StreamExt as _};
use lawyerr::format::Renderable;
use lawyerr::{cli, config, dgsi, dr, format, http, server};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = cli::Cli::parse();
    let cfg = config::load_config(cli.config.as_deref());

    let compact = !cli.no_compact;
    let strip_sw = cli.strip_stopwords;
    let output_path = cli.output.as_deref();
    let fmt = &cli.format;

    match cli.command {
        cli::Commands::Dgsi { command } => match command {
            cli::DgsiCommands::Search(args) => {
                let fetcher = Arc::new(
                    http::HttpClient::new(
                        cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                        cfg.http.timeout_secs,
                        cfg.http.retries,
                    )
                    .context("Failed to build HTTP client")?,
                );

                // Resolve courts
                let courts =
                    dgsi::resolve_courts(&args.court).context("Failed to resolve court aliases")?;

                // Resolve date range
                let since = match (&args.recent, &args.since) {
                    (Some(recent), _) => {
                        Some(format::parse_recent(recent).map_err(anyhow::Error::msg)?)
                    }
                    (None, Some(s)) => Some(
                        s.parse::<chrono::NaiveDate>()
                            .with_context(|| format!("Invalid --since date: '{s}'"))?,
                    ),
                    (None, None) => None,
                };
                let until = match &args.until {
                    Some(u) => Some(
                        u.parse::<chrono::NaiveDate>()
                            .with_context(|| format!("Invalid --until date: '{u}'"))?,
                    ),
                    None => None,
                };

                let field_filter = args.field.as_deref().zip(args.value.as_deref());
                let query = dgsi::build_query(&args.query, since, until, field_filter);

                let sort_by_date = args.sort == "date";
                let max_concurrent = args.max_concurrent.unwrap_or(3);

                let court_results = dgsi::search_all_courts(
                    fetcher.as_ref(),
                    &courts,
                    &query,
                    args.limit,
                    sort_by_date,
                    max_concurrent,
                )
                .await;

                let mut full_output = String::new();

                for (court, result) in court_results {
                    match result {
                        Err(e) => {
                            tracing::warn!(court = court.alias(), error = %e, "Skipping court");
                        }
                        Ok((total, results)) => {
                            if args.fetch_full && !results.is_empty() {
                                let sem = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
                                let mut tasks: FuturesUnordered<_> = results
                                    .iter()
                                    .map(|r| {
                                        let url = r.doc_url.clone();
                                        let sem = Arc::clone(&sem);
                                        let fetcher_arc = Arc::clone(&fetcher);
                                        async move {
                                            let _permit =
                                                sem.acquire().await.expect("semaphore closed");
                                            if let Some(ms) = args.delay_ms {
                                                tokio::time::sleep(
                                                    std::time::Duration::from_millis(ms),
                                                )
                                                .await;
                                            }
                                            dgsi::fetch_full_decision(fetcher_arc.as_ref(), &url)
                                                .await
                                        }
                                    })
                                    .collect();

                                let mut full_renderables: Vec<Box<dyn Renderable>> = Vec::new();
                                while let Some(dec_result) = tasks.next().await {
                                    match dec_result {
                                        Ok(dec) => full_renderables.push(Box::new(dec)),
                                        Err(e) => {
                                            tracing::warn!(error = %e, "Failed to fetch decision");
                                        }
                                    }
                                }

                                let response = format::SearchResponse {
                                    source: court.display_name().to_owned(),
                                    query: query.clone(),
                                    total,
                                    results: full_renderables,
                                };
                                full_output
                                    .push_str(&format::render(&response, fmt, compact, strip_sw));
                            } else {
                                let renderables: Vec<Box<dyn Renderable>> = results
                                    .into_iter()
                                    .map(|r| Box::new(r) as Box<dyn Renderable>)
                                    .collect();
                                let response = format::SearchResponse {
                                    source: court.display_name().to_owned(),
                                    query: query.clone(),
                                    total,
                                    results: renderables,
                                };
                                full_output
                                    .push_str(&format::render(&response, fmt, compact, strip_sw));
                            }
                        }
                    }
                }

                format::write_output(&full_output, output_path)?;
            }

            cli::DgsiCommands::Fetch { url } => {
                let fetcher = http::HttpClient::new(
                    cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                    cfg.http.timeout_secs,
                    cfg.http.retries,
                )
                .context("Failed to build HTTP client")?;

                let decision = dgsi::fetch_full_decision(&fetcher, &url).await?;
                let response = format::SearchResponse {
                    source: "DGSI".to_owned(),
                    query: url.clone(),
                    total: 1,
                    results: vec![Box::new(decision) as Box<dyn Renderable>],
                };
                let rendered = format::render(&response, fmt, compact, strip_sw);
                format::write_output(&rendered, output_path)?;
            }

            cli::DgsiCommands::Courts => {
                let courts = dgsi::list_courts();
                let mut out = String::new();
                for (alias, name) in courts {
                    let _ = writeln!(out, "{alias:<20} {name}");
                }
                std::io::stdout().write_all(out.as_bytes())?;
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
