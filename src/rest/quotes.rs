use crate::client::{Client, RequestOptions};
use crate::error::Result;
use crate::models::{LastQuote, Quote};
use futures::Stream;
use std::future::Future;

/// Quotes (NBBO) API.
pub trait QuotesApi {
    /// List quotes for a ticker (paginated stream).
    fn list_quotes(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> impl Stream<Item = Result<Quote>>;

    /// Get the last quote for a ticker.
    fn get_last_quote(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> impl Future<Output = Result<LastQuote>> + Send;
}

impl QuotesApi for Client {
    fn list_quotes(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> impl Stream<Item = Result<Quote>> {
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
        if self.pagination {
            self.paginate::<Quote>(&path, Some(&params), options)
        } else {
            self.single_page::<Quote>(&path, Some(&params), options)
        }
    }

    async fn get_last_quote(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<LastQuote> {
        let path = format!("/v2/last/nbbo/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp { results: LastQuote }
        let resp: Resp = self.get(&path, None, options).await?;
        Ok(resp.results)
    }
}
