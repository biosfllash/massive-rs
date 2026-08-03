use crate::client::Client;
use crate::rest::{AggsApi, SnapshotApi};
use crate::websocket::WebSocketClient;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn paginated_stream_sends_auth_and_follows_next_url() {
    let server = MockServer::start().await;

    // First page: two results plus a `next_url` cursor.
    Mock::given(method("GET"))
        .and(path(
            "/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-02",
        ))
        .and(query_param("limit", "2"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"o": 1.0, "h": 2.0, "l": 0.5, "c": 1.5, "v": 100.0, "t": 1704067200000i64, "n": 10},
                {"o": 2.0, "h": 3.0, "l": 1.5, "c": 2.5, "v": 200.0, "t": 1704153600000i64, "n": 20}
            ],
            "next_url": format!("{}/v2/next/2?cursor=2", server.uri())
        })))
        .expect(1)
        .mount(&server)
        .await;

    // Second page: reached via `next_url`, must also carry the auth header.
    Mock::given(method("GET"))
        .and(path("/v2/next/2"))
        .and(query_param("cursor", "2"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"o": 3.0, "h": 4.0, "l": 2.5, "c": 3.5, "v": 300.0, "t": 1704240000000i64, "n": 30}
            ],
            "next_url": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::new("test-key").unwrap().with_base(server.uri());

    let mut stream = client.list_aggs(
        "AAPL",
        1,
        "day",
        "2024-01-01",
        "2024-01-02",
        None,
        None,
        Some(2),
        None,
    );

    let mut aggs: Vec<_> = Vec::new();
    while let Some(agg) = stream.next().await {
        aggs.push(agg.unwrap());
    }

    assert_eq!(aggs.len(), 3);
    assert_eq!(aggs[0].open, Some(1.0));
    assert_eq!(aggs[1].open, Some(2.0));
    assert_eq!(aggs[2].open, Some(3.0));
    assert_eq!(aggs[2].timestamp, Some(1704240000000i64));
}

#[tokio::test]
async fn paginated_stream_surfaces_http_errors() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    let client = Client::new("test-key").unwrap().with_base(server.uri());

    let mut stream = client.list_aggs(
        "AAPL",
        1,
        "day",
        "2024-01-01",
        "2024-01-02",
        None,
        None,
        None,
        None,
    );

    let err = stream.next().await.unwrap().unwrap_err();
    assert!(
        matches!(err, crate::error::Error::Http { status, .. } if status.as_u16() == 500),
        "expected HTTP 500 error, got: {err}"
    );
}

#[tokio::test]
async fn client_get_sends_auth_header() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/open-close/AAPL/2024-01-02"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "afterHours": 1.0,
            "close": 1.5,
            "open": 1.0,
            "status": "OK",
            "symbol": "AAPL",
            "volume": 1000.0
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::new("test-key").unwrap().with_base(server.uri());

    let res = client
        .get_daily_open_close_agg("AAPL", "2024-01-02", None, None)
        .await
        .unwrap();

    assert_eq!(res.close, Some(1.5));
    assert_eq!(res.symbol, Some("AAPL".to_owned()));
}

#[tokio::test]
async fn ticker_snapshot_parses_results_envelope() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/snapshot/locale/us/markets/stocks/tickers/AAPL"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "OK",
            "request_id": "abc",
            "ticker": "AAPL",
            "results": {
                "ticker": "AAPL",
                "todaysChange": 1.25,
                "todaysChangePerc": 0.83,
                "updated": 1704067200000i64,
                "day": {"o": 150.0, "h": 152.0, "l": 149.0, "c": 151.25, "v": 1000000.0, "t": 1704067200000i64},
                "lastTrade": {"p": 151.25, "s": 10.0, "t": 1704067200000i64},
                "lastQuote": {"P": 151.26, "p": 151.24, "t": 1704067200000i64}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::new("test-key").unwrap().with_base(server.uri());

    let snap = client.get_ticker_snapshot("AAPL", None).await.unwrap();
    assert_eq!(snap.todays_change, Some(1.25));
    assert_eq!(snap.todays_change_percent, Some(0.83));
    assert_eq!(snap.day.as_ref().and_then(|d| d.close), Some(151.25));
    assert_eq!(snap.last_trade.as_ref().and_then(|t| t.price), Some(151.25));
    assert_eq!(
        snap.last_quote.as_ref().and_then(|q| q.ask_price),
        Some(151.26)
    );
}

#[tokio::test]
async fn websocket_auth_subscribe_and_receive() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Scripted server: expect the auth frame, then the subscribe frame, then
    // send a data frame, a ping, and a final data frame before closing.
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(tcp).await.unwrap();

        let auth = ws.next().await.expect("auth message").unwrap();
        let Message::Text(text) = auth else {
            panic!("expected text auth message, got {auth:?}");
        };
        assert!(text.contains(r#""action":"auth""#) && text.contains("test-key"));

        let sub = ws.next().await.expect("subscribe message").unwrap();
        let Message::Text(text) = sub else {
            panic!("expected text subscribe message, got {sub:?}");
        };
        assert!(text.contains("T.AAPL") && text.contains("T.MSFT"));

        ws.send(Message::Text("{\"ev\":\"T\"}".into()))
            .await
            .unwrap();
        ws.send(Message::Ping(vec![1, 2, 3].into())).await.unwrap();
        ws.send(Message::Text("{\"ev\":\"Q\"}".into()))
            .await
            .unwrap();
    });

    let mut client = WebSocketClient::connect(&format!("ws://{addr}"))
        .await
        .unwrap();
    client.auth("test-key").await.unwrap();
    client.subscribe(&["T.AAPL", "T.MSFT"]).await.unwrap();

    assert_eq!(client.next().await.unwrap().unwrap(), "{\"ev\":\"T\"}");
    // The ping is answered transparently, so the next frame is delivered.
    assert_eq!(client.next().await.unwrap().unwrap(), "{\"ev\":\"Q\"}");

    server.await.unwrap();
}
