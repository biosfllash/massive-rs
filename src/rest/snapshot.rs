use crate::client::{Client, RequestOptions};
use crate::error::{Error, Result};
use crate::models::{TickerSnapshot, UniversalSnapshot};
use std::future::Future;

/// Snapshot API — the current market state for a ticker.
pub trait SnapshotApi {
    /// Get the current stock snapshot (day, previous day, last trade/quote,
    /// and today's change) for a ticker.
    fn get_ticker_snapshot(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> impl Future<Output = Result<TickerSnapshot>> + Send;

    /// Get the current universal snapshot (stocks, options, forex, crypto)
    /// for a ticker.
    fn get_universal_snapshot(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> impl Future<Output = Result<UniversalSnapshot>> + Send;
}

impl SnapshotApi for Client {
    async fn get_ticker_snapshot(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<TickerSnapshot> {
        let path = format!("/v2/snapshot/locale/us/markets/stocks/tickers/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp {
            results: Option<TickerSnapshot>,
        }
        let resp: Resp = self.get(&path, None, options).await?;
        resp.results
            .ok_or_else(|| Error::EmptyResults(format!("no snapshot data for ticker {ticker}")))
    }

    async fn get_universal_snapshot(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<UniversalSnapshot> {
        let path = format!("/v3/snapshot/locale/us/markets/stocks/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp {
            results: Option<UniversalSnapshot>,
        }
        let resp: Resp = self.get(&path, None, options).await?;
        resp.results
            .ok_or_else(|| Error::EmptyResults(format!("no snapshot data for ticker {ticker}")))
    }
}
