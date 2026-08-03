use crate::error::{Error, Result};
use futures::Stream;
use serde::de::DeserializeOwned;
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
    next_url: Option<String>,
    buffer: Vec<T>,
    pending: Option<Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send>>>,
}

use std::future::Future;

impl<T: DeserializeOwned + Send + 'static> PaginatedStream<T> {
    pub(crate) fn new(client: reqwest::Client, initial_url: String) -> Self {
        Self {
            client,
            next_url: Some(initial_url),
            buffer: Vec::new(),
            pending: None,
        }
    }

    pub(crate) fn single_page(client: reqwest::Client, url: String) -> Self {
        Self {
            client,
            next_url: Some(url),
            buffer: Vec::new(),
            pending: None,
        }
    }
}

impl<T: DeserializeOwned + Send + 'static> Stream for PaginatedStream<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Return buffered items first
            if let Some(item) = self.buffer.pop() {
                return Poll::Ready(Some(Ok(item)));
            }

            // No more pages
            if self.next_url.is_none() && self.pending.is_none() {
                return Poll::Ready(None);
            }

            // Start a new request if we have a URL and no pending request
            if let Some(url) = self.next_url.take() {
                let client = self.client.clone();
                self.pending = Some(Box::pin(async move {
                    client.get(&url).send().await.map_err(Error::from)
                }));
            }

            // Poll the pending request
            if let Some(ref mut fut) = self.pending {
                match fut.as_mut().poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(e)) => {
                        self.pending = None;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Ready(Ok(resp)) => {
                        self.pending = None;
                        let status = resp.status();
                        if !status.is_success() {
                            let body = resp.text().await.unwrap_or_default();
                            return Poll::Ready(Some(Err(Error::Http { status, body })));
                        }
                        let page: PaginatedResponse<T> = match resp.json().await {
                            Ok(p) => p,
                            Err(e) => return Poll::Ready(Some(Err(Error::from(e)))),
                        };
                        self.next_url = page.next_url;
                        if let Some(mut results) = page.results {
                            results.reverse(); // so we can pop from the end
                            self.buffer = results;
                        }
                        // If no results and no next_url, stream ends next loop
                    }
                }
            }
        }
    }
}
