# massive-rs

Async Rust client for the [Massive](https://massive.com) (formerly Polygon.io)
REST and WebSocket APIs.

- Typed REST endpoints for aggregates, trades, and quotes
- Automatic pagination: `list_*` methods return a `Stream` that follows
  `next_url` for you
- Real-time WebSocket streaming with automatic ping/pong keep-alive
- `rustls`-only TLS (no OpenSSL needed)
- Fully offline test suite (`wiremock` HTTP mocks + a local WebSocket server)

## Status

| Area       | Trait / type                | Endpoints                                                       |
| ---------- | --------------------------- | --------------------------------------------------------------- |
| Aggregates | `AggsApi`                   | bars (`get_aggs` / `list_aggs`), grouped daily, open/close, prev |
| Snapshots  | `SnapshotApi`               | current ticker snapshot, universal snapshot                      |
| Trades     | `TradesApi`                 | history (`list_trades`), last trade (`get_last_trade`)          |
| Quotes     | `QuotesApi`                 | history (`list_quotes`), last NBBO quote (`get_last_quote`)     |
| WebSocket  | `WebSocketClient`           | connect / auth / subscribe / receive raw JSON frames             |
| Models     | `models`                    | `aggs`, `trades`, `quotes`, `snapshot`, `tickers`, `financials` |

## Getting started

### 1. Set your API key

The client reads `MASSIVE_API_KEY` via `Client::from_env()`. Put the key in a
`.env` file in the project root (see `.env.example`):

```sh
# .env
MASSIVE_API_KEY=your_key_here
```

The `.env` file is loaded automatically when you use `from_env()` — no need to
`export` it in your shell.

### 2. Run the demo binary

```sh
cargo run -- AAPL
```

This prints the last 30 days of daily bars for `AAPL`, the **current snapshot**
(last trade, today's change, today's day bar), the latest trade, the latest NBBO
quote, and a few trades from the paginated stream. Pass any ticker (defaults to
`AAPL`):

```sh
cargo run -- MSFT
cargo run -- TSLA
```

### Realtime WebSocket

Subscribe to trades (`T`), quotes (`Q`), and aggregates (`A`) for specific
tickers:

```sh
cargo run -- ws AAPL MSFT
```

Or everything at once via wildcards (no tickers / `--all` also work):

```sh
cargo run -- ws --all
```

The stream prints raw JSON frames until you press Ctrl+C.

## REST API

Import the client and the trait for the endpoints you need:

```rust
use futures::StreamExt;
use massive::{AggsApi, Client, QuotesApi, TradesApi};

let client = Client::from_env()?; // reads MASSIVE_API_KEY
// or: Client::new("your_api_key")?
```

The default base URL is `https://api.massive.com` and is used automatically.
Use `client.with_base(...)` only to point at a custom endpoint (e.g. a test
server or proxy).

### Aggregates (`AggsApi`)

```rust
// Single page of daily bars
let bars = client
    .get_aggs("AAPL", 1, "day", "2024-01-01", "2024-01-31",
              None, Some("desc"), Some(5), None)
    .await?;

// Full history as a paginated stream
let mut bars = client
    .list_aggs("AAPL", 1, "day", "2020-01-01", "2024-01-01",
               None, None, None, None);
while let Some(bar) = bars.next().await {
    println!("{:?}", bar?);
}
```

Other methods: `get_grouped_daily_aggs(date, ...)`,
`get_daily_open_close_agg(ticker, date, ...)`, `get_previous_close_agg(ticker, ...)`.

### Snapshots (`SnapshotApi`)

The quickest way to get the **current market state** for a ticker:

```rust
use massive::SnapshotApi;

// Current stock snapshot: day bar, prev day, last trade/quote, today's change
let snap = client.get_ticker_snapshot("AAPL", None).await?;
println!("last: {:?}", snap.last_trade.as_ref().and_then(|t| t.price));
println!("today's change: {:?}", snap.todays_change);

// Universal snapshot (stocks, options, forex, crypto)
let usnap = client.get_universal_snapshot("AAPL", None).await?;
println!("session price: {:?}", usnap.session.as_ref().and_then(|s| s.price));
```

### Trades (`TradesApi`)

```rust
let last = client.get_last_trade("AAPL", None).await?;

let mut trades = client.list_trades("AAPL", None, None, Some(100), None, None);
while let Some(trade) = trades.next().await {
    println!("{:?}", trade?);
}
```

### Quotes (`QuotesApi`)

```rust
let last = client.get_last_quote("AAPL", None).await?;

let mut quotes = client.list_quotes("AAPL", None, None, Some(100), None, None);
while let Some(quote) = quotes.next().await {
    println!("{:?}", quote?);
}
```

### Pagination & per-request options

- `list_*` methods return a `Stream<Item = Result<T>>` that automatically
  follows the `next_url` cursor across pages (requires `futures::StreamExt`).
- `client.with_pagination(false)` disables automatic pagination (single page).
- `RequestOptions` adds per-request headers, e.g. the Massive Launchpad edge
  headers:

```rust
use massive::RequestOptions;

let opts = RequestOptions::with_edge_headers("edge-id", "203.0.113.9", None);
let bars = client
    .get_aggs("AAPL", 1, "day", "2024-01-01", "2024-01-31",
              None, None, None, Some(&opts))
    .await?;
```

## WebSocket API

The real-time feed lives at `wss://socket.massive.com/stocks`. Connect, then
authenticate, then subscribe to topics:

```rust
use massive::WebSocketClient;

let mut ws = WebSocketClient::connect("wss://socket.massive.com/stocks").await?;

// 1. Authenticate with your API key
ws.auth("your_api_key").await?;

// 2. Subscribe to topics (wildcards supported), e.g. all quotes + AAPL trades
ws.subscribe(&["Q.*", "T.AAPL"]).await?;

// 3. Consume messages as raw JSON text
while let Some(msg) = ws.next().await {
    println!("{}", msg?);
}
```

Details:

- `connect(endpoint)` opens the connection. `STOCKS_ENDPOINT` is a constant
  for `wss://socket.massive.com/stocks`.
- `auth(api_key)` sends `{"action":"auth","params":"<api_key>"}`.
- `subscribe(&["Q.*", "T.AAPL"])` sends
  `{"action":"subscribe","params":"Q.*,T.AAPL"}`.
- `next()` returns one raw JSON frame per call and answers server pings
  automatically so the connection stays alive; it returns `None` when the
  connection closes.
- `send(text)` sends any custom frame (e.g. an `unsubscribe`).

## Development

```sh
cargo check --all-targets
cargo test   # offline: wiremock HTTP tests + local WebSocket server test
```

## License

MIT OR Apache-2.0
