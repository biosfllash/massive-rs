use reqwest::header::HeaderMap;

#[derive(Debug, Default, Clone)]
pub struct RequestOptions {
    pub headers: HeaderMap,
}
