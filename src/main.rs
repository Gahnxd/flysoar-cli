mod api;
mod models;
mod output;
mod sse;

use std::process::ExitCode;

use anyhow::{Result, bail};
use chrono::{Datelike, NaiveDate};
use clap::{Parser, Subcommand};
use models::{Offer, SearchRequest, SliceRequest};

/// CLI for searching flights via FlySoar.ai's anonymous public API.
///
/// No accounts, API keys, or authentication required.
/// Searches are performed against the live FlySoar.ai SSE endpoint.
#[derive(Parser, Debug)]
#[command(
    name = "flysoar",
    version,
    about = "Search flights via FlySoar.ai's public API",
    long_about = "CLI for searching flights via FlySoar.ai's anonymous public SSE endpoint.\n\
                  No accounts, API keys, or authentication required.\n\n\
                  Supports one-way, round-trip, open-jaw, and multi-city searches.\n\
                  Output formats: JSON (default), CSV, or table."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for flights
    #[command(
        name = "search",
        long_about = "Search for flight offers.\n\n\
                      Examples:\n  \
                        flysoar search -o SFO -d JFK -D 2026-07-15\n  \
                        flysoar search -o SFO -d JFK -D 2026-07-15 -r 2026-07-22 -c business\n  \
                        flysoar search --slices SFO,JFK,2026-07-15 LHR,SFO,2026-07-22 -c economy\n  \
                        flysoar search --input '{\"origin\":\"SFO\",\"destination\":\"JFK\",\"date\":\"2026-07-15\"}'"
    )]
    Search {
        /// Departure IATA airport or city code (e.g. SFO, JFK, NYC, LON)
        #[arg(short = 'o', long, conflicts_with_all = ["input", "slices"])]
        origin: Option<String>,

        /// Arrival IATA airport or city code
        #[arg(short = 'd', long, conflicts_with_all = ["input", "slices"])]
        destination: Option<String>,

        /// Outbound departure date (YYYY-MM-DD)
        #[arg(short = 'D', long, conflicts_with_all = ["input", "slices"])]
        date: Option<String>,

        /// Return date for round-trips (YYYY-MM-DD)
        #[arg(short = 'r', long, conflicts_with_all = ["input", "slices"])]
        return_date: Option<String>,

        /// Multi-city slices in format "ORIGIN,DEST,DATE" (repeatable)
        #[arg(long = "slices", num_args = 1.., conflicts_with_all = ["origin", "destination", "date", "return_date", "input"])]
        slices: Vec<String>,

        /// Accept a JSON search body directly (e.g. '{"origin":"SFO","destination":"JFK","date":"2026-07-15"}')
        #[arg(long = "input", conflicts_with_all = ["origin", "destination", "date", "return_date", "slices"])]
        input: Option<String>,

        /// Cabin class
        #[arg(short = 'c', long, default_value = "economy")]
        cabin: String,

        /// Number of passengers
        #[arg(short = 'p', long, default_value_t = 1)]
        passengers: i32,

        /// Output format
        #[arg(short = 'O', long, default_value = "json")]
        output: String,

        /// Maximum number of offers to return (0 = unlimited)
        #[arg(short = 'n', long, default_value_t = 0)]
        max_offers: usize,

        /// Request timeout in seconds
        #[arg(short = 't', long, default_value_t = 90)]
        timeout: u64,

        /// Sort by: price, depart_time, duration
        #[arg(short = 's', long, default_value = "price")]
        sort: String,

        /// Filter to nonstop flights only
        #[arg(long)]
        nonstop_only: bool,

        /// Suppress all progress output (for piping/agents)
        #[arg(long, short = 'q')]
        quiet: bool,

        /// Output raw unmodified API offers (full schema, not summaries)
        #[arg(long)]
        raw: bool,

        /// Save results to a file (JSON format)
        #[arg(long)]
        save: Option<String>,
    },

    /// Self-update: reinstall flysoar from source
    #[command(name = "update")]
    Update {
        /// Path to the flysoar source directory (defaults to current directory)
        #[arg(long, short = 'p')]
        path: Option<String>,
    },

    /// Show installation info
    #[command(name = "info")]
    Info,

    /// Uninstall the flysoar binary
    #[command(name = "uninstall")]
    Uninstall,
}

fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

fn validate_iata(code: &str) -> Result<()> {
    if code.len() != 3 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        bail!(
            "Invalid IATA code '{}': must be 3 alphabetic characters",
            code
        );
    }
    Ok(())
}

fn validate_date(date: &str) -> Result<()> {
    if date.len() != 10 || !date.is_ascii() {
        bail!("Invalid date '{}': must be YYYY-MM-DD", date);
    }

    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date '{}': must be YYYY-MM-DD", date))?;
    if parsed.format("%Y-%m-%d").to_string() != date {
        bail!("Invalid date '{}': must be YYYY-MM-DD", date);
    }
    if parsed.year() < 2025 {
        bail!("Invalid date '{}': out of range", date);
    }
    Ok(())
}

fn validate_search_request(request: &SearchRequest) -> Result<()> {
    let valid_cabins = ["economy", "premium_economy", "business", "first"];
    if !valid_cabins.contains(&request.cabin.as_str()) {
        bail!(
            "Invalid cabin '{}': must be one of {:?}",
            request.cabin,
            valid_cabins
        );
    }

    if let Some(slices) = &request.slices {
        if slices.is_empty() {
            bail!("At least one slice is required");
        }
        for slice in slices {
            validate_iata(&slice.origin)?;
            validate_iata(&slice.destination)?;
            validate_date(&slice.departure_date)?;
        }
    } else {
        let origin = request
            .origin
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("origin is required"))?;
        let destination = request
            .destination
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("destination is required"))?;
        let date = request
            .date
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("date is required"))?;

        validate_iata(origin)?;
        validate_iata(destination)?;
        validate_date(date)?;
        if let Some(return_date) = request.return_date.as_deref() {
            validate_date(return_date)?;
        }
    }

    Ok(())
}

fn parse_slice(s: &str) -> Result<SliceRequest> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        bail!("Invalid slice '{}': must be ORIGIN,DEST,DATE", s);
    }
    let origin = parts[0].trim().to_uppercase();
    let destination = parts[1].trim().to_uppercase();
    let departure_date = parts[2].trim().to_string();
    validate_iata(&origin)?;
    validate_iata(&destination)?;
    validate_date(&departure_date)?;
    Ok(SliceRequest {
        origin,
        destination,
        departure_date,
    })
}

fn sort_offers(offers: &mut [Offer], sort_by: &str) {
    match sort_by {
        "price" => offers.sort_by(|a, b| {
            let pa: f64 = a.total_amount.parse().unwrap_or(0.0);
            let pb: f64 = b.total_amount.parse().unwrap_or(0.0);
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "depart_time" => offers.sort_by(|a, b| {
            let da = a
                .slices
                .first()
                .and_then(|s| s.segments.first())
                .map(|seg| seg.departure.as_str())
                .unwrap_or("");
            let db = b
                .slices
                .first()
                .and_then(|s| s.segments.first())
                .map(|seg| seg.departure.as_str())
                .unwrap_or("");
            da.cmp(db)
        }),
        "duration" => offers.sort_by(|a, b| {
            let da = a
                .slices
                .first()
                .and_then(|s| s.duration.as_deref())
                .unwrap_or("");
            let db = b
                .slices
                .first()
                .and_then(|s| s.duration.as_deref())
                .unwrap_or("");
            match (duration_seconds(da), duration_seconds(db)) {
                (Some(da), Some(db)) => da.cmp(&db),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => da.cmp(db),
            }
        }),
        _ => {}
    }
}

fn duration_seconds(duration: &str) -> Option<u64> {
    let time = duration.strip_prefix("PT")?;
    let mut number = String::new();
    let mut seconds = 0_u64;
    let mut has_component = false;

    for character in time.chars() {
        if character.is_ascii_digit() {
            number.push(character);
            continue;
        }

        let value: u64 = number.parse().ok()?;
        number.clear();
        let multiplier = match character {
            'H' => 60 * 60,
            'M' => 60,
            'S' => 1,
            _ => return None,
        };
        seconds = seconds.checked_add(value.checked_mul(multiplier)?)?;
        has_component = true;
    }

    if has_component && number.is_empty() {
        Some(seconds)
    } else {
        None
    }
}

fn filter_nonstop(offers: Vec<Offer>) -> Vec<Offer> {
    offers
        .into_iter()
        .filter(|o| o.slices.iter().all(|s| s.segments.len() == 1))
        .collect()
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {:#}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Search {
            origin,
            destination,
            date,
            return_date,
            slices,
            input,
            cabin,
            passengers,
            output,
            max_offers,
            timeout,
            sort,
            nonstop_only,
            quiet,
            raw,
            save,
        } => {
            // Validate cabin
            let valid_cabins = ["economy", "premium_economy", "business", "first"];
            if !valid_cabins.contains(&cabin.as_str()) {
                bail!(
                    "Invalid cabin '{}': must be one of {:?}",
                    cabin,
                    valid_cabins
                );
            }

            // Build request — three modes: --input JSON, --slices, or flat flags
            let (request, headers) = if let Some(json_str) = input {
                let parsed: serde_json::Value = serde_json::from_str(&json_str)
                    .map_err(|e| anyhow::anyhow!("Invalid --input JSON: {}", e))?;

                if parsed.get("slices").is_some() {
                    let req: SearchRequest = serde_json::from_value(parsed)
                        .map_err(|e| anyhow::anyhow!("Invalid search body: {}", e))?;
                    validate_search_request(&req)
                        .map_err(|e| anyhow::anyhow!("Invalid search body: {}", e))?;
                    let headers = api::build_headers_multicity();
                    (req, headers)
                } else {
                    let req: SearchRequest = serde_json::from_value(parsed)
                        .map_err(|e| anyhow::anyhow!("Invalid search body: {}", e))?;
                    validate_search_request(&req)
                        .map_err(|e| anyhow::anyhow!("Invalid search body: {}", e))?;
                    let origin = req.origin.as_deref().expect("validated origin");
                    let destination = req.destination.as_deref().expect("validated destination");
                    let date = req.date.as_deref().expect("validated date");
                    let headers =
                        api::build_headers(origin, destination, date, req.return_date.as_deref());
                    (req, headers)
                }
            } else if !slices.is_empty() {
                let parsed_slices: Vec<SliceRequest> = slices
                    .iter()
                    .map(|s| parse_slice(s))
                    .collect::<Result<Vec<_>>>()?;

                let req = SearchRequest {
                    origin: None,
                    destination: None,
                    date: None,
                    return_date: None,
                    slices: Some(parsed_slices),
                    cabin: cabin.clone(),
                    passengers,
                };

                let headers = api::build_headers_multicity();
                (req, headers)
            } else {
                let origin = origin.ok_or_else(|| {
                    anyhow::anyhow!("--origin is required (or use --slices / --input)")
                })?;
                let destination = destination.ok_or_else(|| {
                    anyhow::anyhow!("--destination is required (or use --slices / --input)")
                })?;
                let date = date.ok_or_else(|| {
                    anyhow::anyhow!("--date is required (or use --slices / --input)")
                })?;

                let origin = origin.to_uppercase();
                let destination = destination.to_uppercase();

                validate_iata(&origin)?;
                validate_iata(&destination)?;
                validate_date(&date)?;
                if let Some(ref rd) = return_date {
                    validate_date(rd)?;
                }

                let req = SearchRequest {
                    origin: Some(origin.clone()),
                    destination: Some(destination.clone()),
                    date: Some(date.clone()),
                    return_date: return_date.clone(),
                    slices: None,
                    cabin: cabin.clone(),
                    passengers,
                };

                let headers =
                    api::build_headers(&origin, &destination, &date, return_date.as_deref());
                (req, headers)
            };

            let client = reqwest::Client::builder().build()?;

            let stream_max_offers = if nonstop_only { 0 } else { max_offers };
            let mut offers = api::search(
                &client,
                &request,
                &headers,
                stream_max_offers,
                timeout,
                quiet,
            )
            .await?;

            if nonstop_only {
                let before = offers.len();
                offers = filter_nonstop(offers);
                if !quiet {
                    eprintln!("Filtered to nonstop: {} -> {} offers", before, offers.len());
                }
            }

            sort_offers(&mut offers, &sort);

            if nonstop_only && max_offers > 0 {
                offers.truncate(max_offers);
            }

            let count = offers.len();
            if !quiet {
                eprintln!("Received {} offers", count);
            }

            if raw {
                let raw_json = serde_json::to_string_pretty(&offers)?;
                if let Some(ref save_path) = save {
                    std::fs::write(save_path, &raw_json)?;
                    if !quiet {
                        eprintln!("Saved {} raw offers to {}", count, save_path);
                    }
                } else {
                    println!("{}", raw_json);
                }
                return Ok(());
            }

            let query = serde_json::json!({
                "origin": request.origin,
                "destination": request.destination,
                "date": request.date,
                "return_date": request.return_date,
                "slices": request.slices,
                "cabin": request.cabin,
                "passengers": request.passengers,
            });

            let summaries: Vec<_> = offers.iter().map(|o| o.to_summary()).collect();
            let result = models::SearchResult {
                query,
                offers: summaries,
                count,
            };

            if let Some(ref save_path) = save {
                let json = serde_json::to_string_pretty(&result)?;
                std::fs::write(save_path, &json)?;
                if !quiet {
                    eprintln!("Saved {} offers to {}", count, save_path);
                }
            } else {
                match output.as_str() {
                    "json" => output::print_json(&result),
                    "csv" => output::print_csv(&result),
                    "table" => output::print_table(&result),
                    _ => bail!(
                        "Invalid output format '{}': must be json, csv, or table",
                        output
                    ),
                }
            }

            Ok(())
        }

        Commands::Update { path } => {
            let source_path = path.unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".to_string())
            });

            eprintln!("Reinstalling flysoar from {}...", source_path);

            let status = std::process::Command::new("cargo")
                .args(["install", "--path", ".", "--force"])
                .current_dir(&source_path)
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to run cargo install: {}", e))?;

            if !status.success() {
                bail!("cargo install failed with exit code {:?}", status.code());
            }

            eprintln!("flysoar updated successfully.");
            Ok(())
        }

        Commands::Info => {
            let cargo_bin = std::env::var("CARGO_HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| home_dir().join(".cargo"));
            let bin_path = cargo_bin.join("bin").join("flysoar");

            let version = env!("CARGO_PKG_VERSION");
            let installed = bin_path.exists();

            println!("flysoar v{}", version);
            println!("  binary:  {}", bin_path.display());
            println!(
                "  status:  {}",
                if installed {
                    "installed"
                } else {
                    "not installed"
                }
            );
            println!("  source:  {}", env!("CARGO_PKG_DESCRIPTION"));
            Ok(())
        }

        Commands::Uninstall => {
            let cargo_bin = std::env::var("CARGO_HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| home_dir().join(".cargo"));

            let bin_path = cargo_bin.join("bin").join("flysoar");

            if !bin_path.exists() {
                eprintln!("flysoar is not installed at {}", bin_path.display());
                return Ok(());
            }

            std::fs::remove_file(&bin_path)
                .map_err(|e| anyhow::anyhow!("Failed to remove {}: {}", bin_path.display(), e))?;
            println!("Uninstalled flysoar from {}", bin_path.display());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_input_dates_before_header_creation() {
        let request = SearchRequest {
            origin: Some("SFO".to_string()),
            destination: Some("JFK".to_string()),
            date: Some("x".to_string()),
            return_date: None,
            slices: None,
            cabin: "economy".to_string(),
            passengers: 1,
        };

        assert!(validate_search_request(&request).is_err());
    }

    #[test]
    fn rejects_non_ascii_dates_without_panicking() {
        assert!(validate_date("ééééé").is_err());
    }

    #[test]
    fn input_uses_documented_defaults() {
        let request: SearchRequest =
            serde_json::from_str(r#"{"origin":"SFO","destination":"JFK","date":"2026-07-15"}"#)
                .unwrap();

        assert_eq!(request.cabin, "economy");
        assert_eq!(request.passengers, 1);
    }

    #[test]
    fn sorts_iso_durations_numerically() {
        assert!(duration_seconds("PT2H").unwrap() < duration_seconds("PT10H").unwrap());
        assert_eq!(duration_seconds("PT1H30M"), Some(5_400));
    }
}
