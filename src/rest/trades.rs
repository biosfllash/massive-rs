use crate::client::{Client, RequestOptions};
use crate::error::Result;
use crate::models::{LastTrade, Trade};
use futures::Stream;
use std::future::Future;

/// Trades API.
pub trait TradesApi {
    /// List trades for a ticker (paginated stream).
    fn list_trades(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> impl Stream<Item = Result<Trade>>;

    /// Get the last trade for a ticker.
    fn get_last_trade(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> impl Future<Output = Result<LastTrade>> + Send;
}

impl TradesApi for Client {
    fn list_trades(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> impl Stream<Item = Result<Trade>> {
        let path = format!("/v3/trades/{}", ticker);
        let mut params: Vec<(String, String)> = Vec::new();
        if let Some(t) = timestamp {
            params.push(("timestamp".into(), t.to_string()));
        }
        if let Some(o) = order {
            params.push(("order".into(), o.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit".into(), l.to_string()));
        }
        if let Some(s) = sort {
            params.push(("sort".into(), s.to_string()));
        }
        if self.pagination {
            self.paginate::<Trade>(&path, Some(&params), options)
        } else {
            self.single_page::<Trade>(&path, Some(&params), options)
        }
    }

    async fn get_last_trade(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<LastTrade> {
        let path = format!("/v2/last/trade/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp {
            results: LastTrade,
        }
        let resp: Resp = self.get(&path, None, options).await?;
        Ok(resp.results)
    }
}
