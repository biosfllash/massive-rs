use crate::client::{Client, RequestOptions};
use crate::error::Result;
use crate::models::{LastQuote, Quote};
use std::future::Future;

/// Quotes (NBBO) API.
pub trait QuotesApi {
    /// Get recent quotes for a ticker (single page, bounded by `limit`).
    fn get_quotes(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> impl Future<Output = Result<Vec<Quote>>> + Send;

    /// Get the last quote for a ticker.
    fn get_last_quote(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> impl Future<Output = Result<LastQuote>> + Send;
}

impl QuotesApi for Client {
    async fn get_quotes(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<Vec<Quote>> {
        let path = format!("/v3/quotes/{}", ticker);
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
        #[derive(serde::Deserialize)]
        struct Resp {
            results: Option<Vec<Quote>>,
        }
        let resp: Resp = self.get(&path, Some(&params), options).await?;
        Ok(resp.results.unwrap_or_default())
    }

    async fn get_last_quote(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<LastQuote> {
        let path = format!("/v2/last/nbbo/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp {
            results: LastQuote,
        }
        let resp: Resp = self.get(&path, None, options).await?;
        Ok(resp.results)
    }
}
