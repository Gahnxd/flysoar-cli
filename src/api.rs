use anyhow::{Context, Result, bail};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};

use crate::models::{Offer, SearchRequest, SseError};
use crate::sse::{SseEvent, SseParser};

const FLYSOAR_URL: &str = "https://flysoar.ai/api/search/stream";
const FLYSOAR_BASE: &str = "https://flysoar.ai";
const USER_AGENT_STR: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

pub fn build_headers(
    origin: &str,
    destination: &str,
    date: &str,
    return_date: Option<&str>,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ORIGIN, HeaderValue::from_static(FLYSOAR_BASE));

    // Build referer: /flights/<origin_lower>/<dest_lower>/<yyMMdd>/[<return_yyMMdd>/]
    let yy_date = date.get(2..).unwrap_or(date).replace('-', "");
    let mut referer = format!(
        "{}/flights/{}/{}/{}/",
        FLYSOAR_BASE,
        origin.to_lowercase(),
        destination.to_lowercase(),
        yy_date
    );
    if let Some(ret) = return_date {
        let yy_ret = ret.get(2..).unwrap_or(ret).replace('-', "");
        referer = format!("{}{}/", referer, yy_ret);
    }
    headers.insert(
        REFERER,
        HeaderValue::from_str(&referer).unwrap_or_else(|_| HeaderValue::from_static(FLYSOAR_BASE)),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers
}

pub fn build_headers_multicity() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ORIGIN, HeaderValue::from_static(FLYSOAR_BASE));
    headers.insert(REFERER, HeaderValue::from_static(FLYSOAR_BASE));
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers
}

/// Stream offers from FlySoar. Returns a vector of offers.
/// If `max_offers` > 0, stops after collecting that many.
pub async fn search(
    client: &reqwest::Client,
    request: &SearchRequest,
    headers: &HeaderMap,
    max_offers: usize,
    timeout_secs: u64,
    quiet: bool,
) -> Result<Vec<Offer>> {
    let pb = if quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb.set_style(
            ProgressStyle::with_template("{spinner} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        pb.set_message("Searching FlySoar.ai...");
        pb
    };

    let response = client
        .post(FLYSOAR_URL)
        .headers(headers.clone())
        .json(request)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
        .context("Failed to send request to FlySoar")?;

    let status = response.status();

    // If HTTP 400, read body as plain JSON error
    if status.as_u16() == 400 {
        let body = response.text().await.unwrap_or_default();
        pb.finish_and_clear();
        bail!("FlySoar error (400): {}", body);
    }

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        pb.finish_and_clear();
        bail!("FlySoar error ({}): {}", status, body);
    }

    // Stream the SSE response
    let mut byte_stream = response.bytes_stream();
    let mut buffer = Vec::new();
    let mut parser = SseParser::new();
    let mut offers = Vec::new();

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = chunk_result.context("Error reading SSE stream")?;
        buffer.extend_from_slice(&chunk);

        // Process complete lines from buffer
        while let Some(nl_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer[..nl_pos].to_vec();
            buffer = buffer[nl_pos + 1..].to_vec();

            let line = String::from_utf8_lossy(&line_bytes);
            let trimmed = line.trim_end_matches('\r');

            if let Some(event) = parser.push_line(trimmed)
                && handle_event(event, &mut offers, &pb, max_offers)?
            {
                return Ok(offers);
            }
        }
    }

    // Stream ended without explicit done — flush remaining
    if !buffer.is_empty() {
        let line = String::from_utf8_lossy(&buffer);
        if let Some(event) = parser.push_line(line.trim_end_matches('\r'))
            && handle_event(event, &mut offers, &pb, max_offers)?
        {
            return Ok(offers);
        }
    }

    if let Some(event) = parser.flush()
        && handle_event(event, &mut offers, &pb, max_offers)?
    {
        return Ok(offers);
    }

    pb.finish_with_message(format!("Received {} offers", offers.len()));
    Ok(offers)
}

fn handle_event(
    event: SseEvent,
    offers: &mut Vec<Offer>,
    progress: &ProgressBar,
    max_offers: usize,
) -> Result<bool> {
    match event.event.as_str() {
        "offer" => {
            if let Ok(offer) = serde_json::from_str::<Offer>(&event.data) {
                offers.push(offer);
                progress.set_message(format!("Found {} offers...", offers.len()));
                if max_offers > 0 && offers.len() >= max_offers {
                    progress.finish_and_clear();
                    return Ok(true);
                }
            }
        }
        "error" => {
            if let Ok(err) = serde_json::from_str::<SseError>(&event.data) {
                progress.finish_and_clear();
                bail!(
                    "FlySoar search error: {} (status {})",
                    err.error,
                    err.status.unwrap_or(0)
                );
            }
        }
        "done" => {
            progress.finish_with_message(format!("Received {} offers", offers.len()));
            return Ok(true);
        }
        _ => {} // created, batch — ignore
    }
    Ok(false)
}
