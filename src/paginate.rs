use crate::error::{Error, Result};
use futures::Stream;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A paginated response from the Massive API.
#[derive(Debug, serde::Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Option<Vec<T>>,
    #[serde(rename = "next_url")]
    pub next_url: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "request_id")]
    pub request_id: Option<String>,
    pub count: Option<i64>,
}

/// A stream that automatically follows `next_url` pagination.
pub struct PaginatedStream<T> {
    client: reqwest::Client,
    headers: HeaderMap,
    next_url: Option<String>,
    buffer: Vec<T>,
    pending: Option<Pin<Box<dyn Future<Output = Result<PaginatedResponse<T>>> + Send>>>,
    /// Whether the stream follows `next_url` cursors (false in single-page mode).
    follow: bool,
}

impl<T: DeserializeOwned + Send + 'static> PaginatedStream<T> {
    pub(crate) fn new(client: reqwest::Client, initial_url: String, headers: HeaderMap) -> Self {
        Self {
            client,
            headers,
            next_url: Some(initial_url),
            buffer: Vec::new(),
            pending: None,
            follow: true,
        }
    }

    pub(crate) fn single_page(client: reqwest::Client, url: String, headers: HeaderMap) -> Self {
        Self {
            client,
            headers,
            next_url: Some(url),
            buffer: Vec::new(),
            pending: None,
            follow: false,
        }
    }
}

impl<T: DeserializeOwned + Send + Unpin + 'static> Stream for PaginatedStream<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Every field of `PaginatedStream<T>` is `Unpin` when `T` is, so the
        // pinned reference can be safely projected into a mutable one.
        let this = self.as_mut().get_mut();
        loop {
            // Return buffered items first
            if let Some(item) = this.buffer.pop() {
                return Poll::Ready(Some(Ok(item)));
            }

            // No more pages
            if this.next_url.is_none() && this.pending.is_none() {
                return Poll::Ready(None);
            }

            // Start a new request if we have a URL and no pending request
            if let Some(url) = this.next_url.take() {
                let client = this.client.clone();
                let headers = this.headers.clone();
                this.pending = Some(Box::pin(async move {
                    let resp = client
                        .get(&url)
                        .headers(headers)
                        .send()
                        .await
                        .map_err(Error::from)?;
                    let status = resp.status();
                    if !status.is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        return Err(Error::Http { status, body });
                    }
                    resp.json::<PaginatedResponse<T>>()
                        .await
                        .map_err(Error::from)
                }));
            }

            // Poll the pending request
            if let Some(ref mut fut) = this.pending {
                match fut.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(e)) => {
                        this.pending = None;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Ready(Ok(page)) => {
                        this.pending = None;
                        // In single-page mode the cursor is discarded.
                        this.next_url = if this.follow { page.next_url } else { None };
                        if let Some(mut results) = page.results {
                            results.reverse(); // so we can pop from the end
                            this.buffer = results;
                        }
                    }
                }
            }
        }
    }
}
