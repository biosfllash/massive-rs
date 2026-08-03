//! Demo binary for the `massive` crate.
//!
//! Usage: put your key in a `.env` file (`MASSIVE_API_KEY=...`) or set it in
//! the shell, then run:
//!   cargo run -- [TICKER]        # REST demo (defaults to AAPL)
//!   cargo run -- ws [TICKER...]  # realtime: trades/quotes/aggregates
//!   cargo run -- ws --all        # realtime for all symbols (wildcards)
//!
//! Examples:
//!   cargo run -- AAPL
//!   cargo run -- ws AAPL MSFT
//!   cargo run -- ws --all

use futures::StreamExt;
use massive::error::Result;
use massive::{
    AggsApi, Client, QuotesApi, SnapshotApi, TradesApi, WebSocketClient, STOCKS_ENDPOINT,
};
use std::io::Write;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let first = args.next();

    // `cargo run -- ws [TICKER...]` opens the realtime WebSocket demo.
    if first.as_deref() == Some("ws") {
        let tickers: Vec<String> = args.collect();
        return realtime(&tickers).await;
    }

    let ticker = first.unwrap_or_else(|| "AAPL".to_owned());
    let client = Client::from_env()?;
    rest_demo(&client, &ticker).await
}

/// REST demo: daily bars, current snapshot, last trade/quote, trade stream.
async fn rest_demo(client: &Client, ticker: &str) -> Result<()> {
    println!("== Daily aggregates for {} (last 30 days) ==", ticker);
    let today = chrono::Utc::now().date_naive();
    let from_date = (today - chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();
    let to_date = today.format("%Y-%m-%d").to_string();

    let aggs = client
        .get_aggs(
            &ticker,
            1,
            "day",
            &from_date,
            &to_date,
            None,
            Some("desc"),
            Some(10),
            None,
        )
        .await?;
    if aggs.is_empty() {
        println!("  (no data)");
    }
    for a in aggs {
        println!(
            "  {}  O={:.2} H={:.2} L={:.2} C={:.2} V={:.0}",
            fmt_day(a.timestamp),
            a.open.unwrap_or_default(),
            a.high.unwrap_or_default(),
            a.low.unwrap_or_default(),
            a.close.unwrap_or_default(),
            a.volume.unwrap_or_default(),
        );
    }

    println!("\n== Current snapshot ==");
    let snap = client.get_ticker_snapshot(&ticker, None).await?;
    println!(
        "  last={:.2}  today: {:.2} ({:.2}%)  updated {}",
        snap.last_trade
            .as_ref()
            .and_then(|t| t.price)
            .unwrap_or_default(),
        snap.todays_change.unwrap_or_default(),
        snap.todays_change_percent.unwrap_or_default(),
        fmt_ts(snap.updated),
    );
    if let Some(day) = &snap.day {
        println!(
            "  day: O={:.2} H={:.2} L={:.2} C={:.2} V={:.0}",
            day.open.unwrap_or_default(),
            day.high.unwrap_or_default(),
            day.low.unwrap_or_default(),
            day.close.unwrap_or_default(),
            day.volume.unwrap_or_default(),
        );
    }

    println!("\n== Last trade ==");
    let last = client.get_last_trade(&ticker, None).await?;
    println!(
        "  {}  price={:.2} size={:.0}",
        fmt_ts(last.sip_timestamp),
        last.price.unwrap_or_default(),
        last.size.unwrap_or_default(),
    );

    println!("\n== Last NBBO quote ==");
    let quote = client.get_last_quote(&ticker, None).await?;
    println!(
        "  bid={:.2} ask={:.2}  ({})",
        quote.bid_price.unwrap_or_default(),
        quote.ask_price.unwrap_or_default(),
        fmt_ts(quote.sip_timestamp),
    );

    println!("\n== First 3 trades via the paginated stream ==");
    let mut stream = client.list_trades(&ticker, None, None, Some(3), None, None);
    let mut seen = 0;
    while let Some(trade) = stream.next().await {
        let trade = trade?;
        println!(
            "  {}  price={:.2} size={:.0}",
            fmt_ts(trade.sip_timestamp),
            trade.price.unwrap_or_default(),
            trade.size.unwrap_or_default(),
        );
        seen += 1;
        if seen >= 3 {
            break;
        }
    }

    Ok(())
}

/// Realtime demo: subscribe to trades/quotes/aggregates for the given tickers
/// and print messages until Ctrl+C.
///
/// Channels per ticker: `T.*` trades, `Q.*` quotes, `A.*` aggregates.
async fn realtime(tickers: &[String]) -> Result<()> {
    let _ = dotenvy::dotenv();
    let key = std::env::var("MASSIVE_API_KEY").map_err(|_| massive::Error::MissingApiKey)?;

    // No tickers, `--all`, or `*` subscribes to everything via wildcards.
    let all = tickers.is_empty() || tickers.iter().any(|t| t == "--all" || t == "*");
    let channels: Vec<String> = if all {
        vec!["T.*".to_owned(), "Q.*".to_owned(), "A.*".to_owned()]
    } else {
        tickers
            .iter()
            .flat_map(|t| [format!("T.{t}"), format!("Q.{t}"), format!("A.{t}")])
            .collect()
    };

    let mut ws = WebSocketClient::connect(STOCKS_ENDPOINT).await?;
    ws.auth(&key).await?;

    let channel_refs: Vec<&str> = channels.iter().map(|c| c.as_str()).collect();
    ws.subscribe(&channel_refs).await?;

    println!("Subscribed to: {:?}", channels);
    println!("Waiting for messages... (Ctrl+C to stop)");

    let stdout = std::io::stdout();
    let mut count: u64 = 0;
    while let Some(msg) = ws.next().await {
        let msg = msg?;

        if all {
            // The all-symbols feed is extremely hot; only report every
            // Nth event, showing event time, receive time, and latency.
            count += 1;
            if count % ALL_SYMBOLS_LOG_EVERY == 0 {
                log_event(&msg, count);
            }
        } else {
            let mut out = stdout.lock();
            let _ = writeln!(out, "{msg}");
        }
    }

    Ok(())
}

/// When subscribed to all symbols, log one line every N received events.
const ALL_SYMBOLS_LOG_EVERY: u64 = 50_000;

/// Extract the event timestamp (epoch millis) from a raw feed message.
/// Trade/quote/aggregate events carry it in the `t` field (some channels
/// use `timestamp`); status messages have none.
fn event_ts_millis(msg: &str) -> Option<i64> {
    let v: serde_json::Value = serde_json::from_str(msg).ok()?;
    v.get("t")
        .or_else(|| v.get("timestamp"))
        .and_then(|t| t.as_i64().or_else(|| t.as_f64().map(|f| f as i64)))
}

/// Log the event's timestamp, the local system timestamp, and the latency
/// (how long after the event occurred we received it).
fn log_event(msg: &str, count: u64) {
    let now = chrono::Utc::now();
    let now_millis = now.timestamp_millis();
    match event_ts_millis(msg) {
        Some(ts) => println!(
            "[{count}] event_ts={} system_ts={} latency={}ms",
            fmt_ts_ms(Some(ts)),
            fmt_ts_ms(Some(now_millis)),
            now_millis - ts,
        ),
        None => println!(
            "[{count}] event_ts=- system_ts={} latency=-",
            fmt_ts_ms(Some(now_millis)),
        ),
    }
}

/// Format an epoch-millis timestamp as `YYYY-MM-DD`.
fn fmt_day(millis: Option<i64>) -> String {
    match millis.and_then(chrono::DateTime::from_timestamp_millis) {
        Some(dt) => dt.format("%Y-%m-%d").to_string(),
        None => "-".to_owned(),
    }
}

/// Format an epoch-millis timestamp as `YYYY-MM-DD HH:MM:SS.mmm`.
fn fmt_ts_ms(millis: Option<i64>) -> String {
    match millis.and_then(chrono::DateTime::from_timestamp_millis) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        None => "-".to_owned(),
    }
}

/// Format an epoch-millis timestamp as `YYYY-MM-DD HH:MM:SS`.
fn fmt_ts(millis: Option<i64>) -> String {
    match millis.and_then(chrono::DateTime::from_timestamp_millis) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "-".to_owned(),
    }
}
