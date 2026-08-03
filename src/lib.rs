//! Async Rust client for the Massive.com (formerly Polygon.io) REST API.

pub mod client;
pub mod error;
pub mod models;
pub mod rest;
pub mod websocket;

pub use client::{Client, RequestOptions};
pub use error::{Error, Result};
pub use rest::{AggsApi, QuotesApi, SnapshotApi, TradesApi};
pub use websocket::{WebSocketClient, STOCKS_ENDPOINT};

#[cfg(test)]
mod tests;
