use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser as _;
use futures::stream::{FuturesUnordered, StreamExt as _};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lauyer::format::Renderable;
use lauyer::{cli, config, dgsi, dr, format, http, server};

#[tokio::main]
#[allow(clippy::too_many_lines, clippy::literal_string_with_formatting_args)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = cli::Cli::parse();
    let cfg = config::load_config(cli.config.as_deref())?;

    let compact = if cli.no_compact { false } else { cfg.output.compact };
    let strip_sw = cli.strip_stopwords || cfg.output.strip_stopwords;
    let output_path = cli.output.as_deref();
    let quiet = cli.quiet;

    // Resolve output format: explicit --format wins; otherwise infer from
    // --output extension; otherwise fall back to config value.
    let fmt = cli.format.unwrap_or_else(|| {
        output_path
            .and_then(format::format_from_extension)
            .unwrap_or_else(|| cfg.output.format.clone())
    });

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
                if args.recent.is_some() && args.since.is_some() {
                    anyhow::bail!("--recent and --since are mutually exclusive");
                }
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

                let sort_by_date = matches!(args.sort, cli::SortOrder::Date);
                let max_concurrent = args.max_concurrent.unwrap_or(cfg.http.max_concurrent).max(1);

                // --- Progress: set up per-court spinners ---
                let mp = MultiProgress::new();
                let spinner_style = ProgressStyle::with_template("{prefix} {spinner} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner());

                let court_spinners: Vec<ProgressBar> = if quiet {
                    Vec::new()
                } else {
                    courts
                        .iter()
                        .map(|c| {
                            let pb = mp.add(ProgressBar::new_spinner());
                            pb.set_style(spinner_style.clone());
                            pb.set_prefix(format!("[{}]", c.alias()));
                            pb.set_message("Searching...");
                            pb.enable_steady_tick(std::time::Duration::from_millis(100));
                            pb
                        })
                        .collect()
                };

                let court_results = dgsi::search_all_courts(
                    fetcher.as_ref(),
                    &courts,
                    &query,
                    args.limit,
                    sort_by_date,
                    max_concurrent,
                    args.delay_ms,
                )
                .await;

                // --- Progress: update spinners with results ---
                if !quiet {
                    for (court, result) in &court_results {
                        if let Some(pb) = court_spinners
                            .iter()
                            .find(|pb| pb.prefix() == format!("[{}]", court.alias()))
                        {
                            match result {
                                Ok((total, _)) => {
                                    pb.finish_with_message(format!("done, {total} results"));
                                }
                                Err(e) => {
                                    let msg = e.to_string();
                                    let short = if msg.contains("timeout") {
                                        "timeout".to_owned()
                                    } else {
                                        truncate_msg(&msg, 60)
                                    };
                                    pb.finish_with_message(format!("error: {short}"));
                                }
                            }
                        }
                    }
                }

                let mut full_output = String::new();

                for (court, result) in court_results {
                    match result {
                        Err(e) => {
                            tracing::warn!(court = court.alias(), error = %e, "Skipping court");
                        }
                        Ok((total, results)) => {
                            if args.fetch_full && !results.is_empty() {
                                // --- Progress: fetch-full progress bar ---
                                let fetch_pb = if quiet {
                                    None
                                } else {
                                    let pb = mp.add(ProgressBar::new(results.len() as u64));
                                    pb.set_style(
                                        ProgressStyle::with_template(
                                            "[{bar:30}] {pos}/{len} Fetching decisions...",
                                        )
                                        .unwrap_or_else(|_| ProgressStyle::default_bar()),
                                    );
                                    pb.set_prefix(format!("[{}]", court.alias()));
                                    Some(pb)
                                };

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
                                    if let Some(ref pb) = fetch_pb {
                                        pb.inc(1);
                                    }
                                    match dec_result {
                                        Ok(dec) => full_renderables.push(Box::new(dec)),
                                        Err(e) => {
                                            tracing::warn!(error = %e, "Failed to fetch decision");
                                        }
                                    }
                                }

                                if let Some(pb) = fetch_pb {
                                    pb.finish_and_clear();
                                }

                                let response = format::SearchResponse {
                                    source: court.display_name().to_owned(),
                                    query: query.clone(),
                                    total,
                                    results: full_renderables,
                                };
                                full_output
                                    .push_str(&format::render(&response, &fmt, compact, strip_sw));
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
                                    .push_str(&format::render(&response, &fmt, compact, strip_sw));
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

                let pb = if quiet {
                    None
                } else {
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::with_template("{spinner} {msg}")
                            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                    );
                    pb.set_message("Fetching decision...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                    Some(pb)
                };

                let decision = dgsi::fetch_full_decision(&fetcher, &url).await?;

                if let Some(pb) = pb {
                    pb.finish_and_clear();
                }

                let response = format::SearchResponse {
                    source: "DGSI".to_owned(),
                    query: url.clone(),
                    total: 1,
                    results: vec![Box::new(decision) as Box<dyn Renderable>],
                };
                let rendered = format::render(&response, &fmt, compact, strip_sw);
                format::write_output(&rendered, output_path)?;
            }

            cli::DgsiCommands::Courts => {
                let courts = dgsi::list_courts();
                let out = if fmt == format::OutputFormat::Json {
                    let items: Vec<serde_json::Value> = courts
                        .iter()
                        .map(|(alias, name)| serde_json::json!({"alias": alias, "name": name}))
                        .collect();
                    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_owned())
                } else {
                    let mut md = String::new();
                    let _ = writeln!(md, "| Alias | Court |");
                    let _ = writeln!(md, "|---|---|");
                    for (alias, name) in &courts {
                        let _ = writeln!(md, "| `{alias}` | {name} |");
                    }
                    md
                };
                format::write_output(&out, output_path)?;
            }
        },

        cli::Commands::Dr { command } => match command {
            cli::DrCommands::Search(args) => {
                let client = http::HttpClient::new(
                    cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                    cfg.http.timeout_secs,
                    cfg.http.retries,
                )
                .context("Failed to build HTTP client")?;

                let pb = if quiet {
                    None
                } else {
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::with_template("{spinner} {msg}")
                            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                    );
                    pb.set_message("Initializing DR session...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                    Some(pb)
                };

                let session =
                    dr::DrSession::new(client).await.context("Failed to initialize DR session")?;

                if let Some(ref pb) = pb {
                    pb.set_message("Searching Diário da República...");
                }

                // Resolve content types
                let content_aliases = if args.content.is_empty() {
                    vec!["atos-1".to_owned()]
                } else {
                    args.content.clone()
                };
                let content_types = dr::resolve_content_types(&content_aliases)
                    .context("Failed to resolve content types")?;

                // Resolve act types
                let mut act_types = Vec::new();
                for alias in &args.act_type {
                    let resolved = dr::resolve_act_type(alias)
                        .ok_or_else(|| anyhow::anyhow!("Unknown act type alias: '{alias}'"))?;
                    act_types.push(resolved);
                }

                // Resolve dates
                if args.recent.is_some() && args.since.is_some() {
                    anyhow::bail!("--recent and --since are mutually exclusive");
                }
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

                let params = dr::DrSearchParams {
                    content_types,
                    query: args.query.clone().unwrap_or_default(),
                    act_types,
                    series: vec![],
                    since,
                    until,
                    limit: args.limit,
                };

                let response = dr::search(&session, &params).await.context("DR search failed")?;
                let response = dr::apply_limit(response, args.limit);

                if let Some(pb) = pb {
                    pb.finish_with_message(format!("done, {} results", response.total));
                }

                let renderables: Vec<Box<dyn Renderable>> = response
                    .results
                    .into_iter()
                    .map(|r| Box::new(r) as Box<dyn Renderable>)
                    .collect();

                let search_response = format::SearchResponse {
                    source: "Diário da República".to_owned(),
                    query: params.query.clone(),
                    total: response.total,
                    results: renderables,
                };
                let rendered = format::render(&search_response, &fmt, compact, strip_sw);
                format::write_output(&rendered, output_path)?;
            }

            cli::DrCommands::Today(args) => {
                let client = http::HttpClient::new(
                    cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                    cfg.http.timeout_secs,
                    cfg.http.retries,
                )
                .context("Failed to build HTTP client")?;

                let pb = if quiet {
                    None
                } else {
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::with_template("{spinner} {msg}")
                            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                    );
                    pb.set_message("Initializing DR session...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                    Some(pb)
                };

                let session =
                    dr::DrSession::new(client).await.context("Failed to initialize DR session")?;

                if let Some(ref pb) = pb {
                    pb.set_message("Fetching today's publications...");
                }

                // Resolve content types from config defaults
                let content_types = dr::resolve_content_types(&[String::from("atos-1")])
                    .context("Failed to resolve content types")?;

                // Resolve act types
                let mut act_types = Vec::new();
                for alias in &args.act_type {
                    let resolved = dr::resolve_act_type(alias)
                        .ok_or_else(|| anyhow::anyhow!("Unknown act type alias: '{alias}'"))?;
                    act_types.push(resolved);
                }

                let today = chrono::Local::now().date_naive();
                let params = dr::DrSearchParams {
                    content_types,
                    query: String::new(),
                    act_types,
                    series: vec![],
                    since: Some(today),
                    until: Some(today),
                    limit: 50,
                };

                let response = dr::search(&session, &params).await.context("DR search failed")?;

                if let Some(pb) = pb {
                    pb.finish_with_message(format!("done, {} results", response.total));
                }

                let renderables: Vec<Box<dyn Renderable>> = response
                    .results
                    .into_iter()
                    .map(|r| Box::new(r) as Box<dyn Renderable>)
                    .collect();

                let search_response = format::SearchResponse {
                    source: "Diário da República — Today".to_owned(),
                    query: String::new(),
                    total: response.total,
                    results: renderables,
                };
                let rendered = format::render(&search_response, &fmt, compact, strip_sw);
                format::write_output(&rendered, output_path)?;
            }

            cli::DrCommands::Types => {
                let types = dr::list_act_types();
                let out = if fmt == format::OutputFormat::Json {
                    let items: Vec<serde_json::Value> = types
                        .iter()
                        .map(|(alias, name)| serde_json::json!({"alias": alias, "name": name}))
                        .collect();
                    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_owned())
                } else {
                    let mut md = String::new();
                    let _ = writeln!(md, "| Alias | Act Type |");
                    let _ = writeln!(md, "|---|---|");
                    for (alias, name) in &types {
                        let _ = writeln!(md, "| `{alias}` | {name} |");
                    }
                    md
                };
                format::write_output(&out, output_path)?;
            }
        },

        cli::Commands::Serve(args) => {
            let http_client = http::HttpClient::new(
                cli.proxy.as_deref().or(cfg.http.proxy.as_deref()),
                cfg.http.timeout_secs,
                cfg.http.retries,
            )
            .context("Failed to build HTTP client")?;

            server::start(&args.host, args.port, cfg, http_client)
                .await
                .context("Server failed")?;
        }
    }

    Ok(())
}

fn truncate_msg(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_owned()
    } else {
        let mut end = max.saturating_sub(3);
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        let mut t = s[..end].to_owned();
        t.push_str("...");
        t
    }
}
