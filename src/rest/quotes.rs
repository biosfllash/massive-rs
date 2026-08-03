use crate::client::{Client, RequestOptions};
use crate::error::Result;
use crate::models::{LastQuote, Quote};
use crate::paginate::PaginatedStream;
use futures::Stream;

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
    async fn get_last_quote(
        &self,
        ticker: &str,
        options: Option<&RequestOptions>,
    ) -> Result<LastQuote>;
}

impl QuotesApi for Client {
    fn list_quotes(
        &self,
        ticker: &str,
        timestamp: Option<&str>,
        order: Option<&str>,
        limit: Option<i64>,
        sort: Option<&str>,
        _options: Option<&RequestOptions>,
    ) -> impl Stream<Item = Result<Quote>> {
        let path = format!("/v3/quotes/{}", ticker);
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
            self.paginate::<Quote>(&path, Some(&params))
        } else {
            self.single_page::<Quote>(&path, Some(&params))
        }
    }

    async fn get_last_quote(
        &self,
        ticker: &str,
        _options: Option<&RequestOptions>,
    ) -> Result<LastQuote> {
        let path = format!("/v2/last/nbbo/{}", ticker);
        #[derive(serde::Deserialize)]
        struct Resp { results: LastQuote }
        let resp: Resp = self.get(&path, None, None).await?;
        Ok(resp.results)
    }
}
