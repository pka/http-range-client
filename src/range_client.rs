use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::str;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub(crate) trait HttpRangeClient {
    fn new() -> Self;
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub(crate) trait HttpRangeClient {
    fn new() -> Self;
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
}

/// HTTP client for HTTP Range requests (https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests)
pub(crate) struct GenericHttpRangeClient<T: HttpRangeClient> {
    client: T,
    url: String,
    stats: RequestStats,
}

#[derive(Default)]
struct RequestStats {
    requests_ever_made: usize,
    bytes_ever_requested: usize,
}

impl RequestStats {
    fn log_get_range(&mut self, _begin: usize, length: usize, range: &str) {
        self.requests_ever_made += 1;
        self.bytes_ever_requested += length;
        debug!(
            "request: #{}, bytes: (this_request: {length}, ever: {}), Range: {range}",
            self.requests_ever_made, self.bytes_ever_requested,
        );
    }
}

impl<T: HttpRangeClient> GenericHttpRangeClient<T> {
    pub fn new(url: &str) -> Self {
        GenericHttpRangeClient {
            client: T::new(),
            url: url.to_string(),
            stats: RequestStats::default(),
        }
    }
    pub async fn get_range(&mut self, begin: usize, length: usize) -> Result<Bytes> {
        let range = format!("bytes={}-{}", begin, begin + length - 1);
        self.stats.log_get_range(begin, length, &range);
        self.client.get_range(&self.url, &range).await
    }
}
