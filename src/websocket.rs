//! Real-time WebSocket streaming support.
//!
//! The Massive real-time feed is a WebSocket at `wss://socket.massive.com/stocks`.
//! After connecting you must authenticate, then subscribe to topics:
//!
//! ```no_run
//! use massive::WebSocketClient;
//! # async fn run() -> massive::Result<()> {
//! let mut ws = WebSocketClient::connect("wss://socket.massive.com/stocks").await?;
//! ws.auth("your_api_key").await?;
//! ws.subscribe(&["Q.*", "T.AAPL"]).await?;
//! while let Some(msg) = ws.next().await {
//!     println!("{}", msg?);
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{Error, Result};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Default Massive stocks real-time feed endpoint.
pub const STOCKS_ENDPOINT: &str = "wss://socket.massive.com/stocks";

/// A streaming WebSocket connection to a Massive real-time feed.
#[derive(Debug)]
pub struct WebSocketClient {
    stream: WsStream,
}

impl WebSocketClient {
    /// Connect to a real-time feed, e.g. [`STOCKS_ENDPOINT`].
    pub async fn connect(endpoint: &str) -> Result<Self> {
        let (stream, _) = connect_async(endpoint)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;
        Ok(Self { stream })
    }

    /// Authenticate with your API key:
    /// `{"action":"auth","params":"<api_key>"}`.
    pub async fn auth(&mut self, api_key: &str) -> Result<()> {
        self.send(&format!(r#"{{"action":"auth","params":"{api_key}"}}"#))
            .await
    }

    /// Send an arbitrary text message to the server (e.g. an `unsubscribe`).
    pub async fn send(&mut self, text: &str) -> Result<()> {
        self.stream
            .send(Message::Text(text.into()))
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))
    }

    /// Subscribe to one or more channels. Wildcards are supported, e.g.
    /// `["Q.*", "T.AAPL", "AM"]`.
    pub async fn subscribe(&mut self, channels: &[&str]) -> Result<()> {
        let params = channels.join(",");
        self.send(&format!(r#"{{"action":"subscribe","params":"{params}"}}"#))
            .await
    }

    /// Receive the next message as raw JSON text.
    ///
    /// Returns `None` when the connection closes. `Ping` frames are answered
    /// automatically so the server keeps the connection open; binary and
    /// other control frames are skipped.
    pub async fn next(&mut self) -> Option<Result<String>> {
        loop {
            match self.stream.next().await {
                None => return None,
                Some(Err(e)) => return Some(Err(Error::WebSocket(e.to_string()))),
                Some(Ok(Message::Text(text))) => return Some(Ok(text.to_string())),
                Some(Ok(Message::Ping(payload))) => {
                    // Answer pings so the server keeps the connection open.
                    let _ = self.stream.send(Message::Pong(payload)).await;
                }
                Some(Ok(Message::Close(_))) => return None,
                Some(Ok(_)) => {}
            }
        }
    }
}
