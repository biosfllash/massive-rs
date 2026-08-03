use crate::client::{Client, RequestOptions};
use crate::error::Result;
use crate::models::{LastTrade, Trade};
use crate::paginate::PaginatedStream;
use futures::Stream;

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
    async fn get_last_trade(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<LastTrade>;
}

impl TradesApi for Client {
    fn list_trades(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        _options: Option<&RequestOptions>,
    ) -> impl Stream<Item = Result<Trade>> {
        let path = format!("/v3/trades/{}", ticker);
        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(t) = timestamp {
            params.push(("timestamp", t));
        }
        if let Some(o) = order {
            params.push(("order", o));
        }
        if let Some(l) = limit {
            params.push(("limit", &l.to_string()));
        }
        if let Some(s) = sort {
            params.push(("sort", s));
        }
        if self.pagination {
            self.paginate::<Trade>(&path, Some(&params))
        } else {
            self.single_page::<Trade>(&path, Some(&params))
        }
    }

    async fn get_last_trade(
        &self,
        ticker: &str,
        _options: Option<&RequestOptions>,
    ) -> Result<LastTrade> {
        let path = format!("/v2/last/trade/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp { results: LastTrade }
        let resp: Resp = self.get(&path, None, None).await?;
        Ok(resp.results)
    }
}
