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
    requests_ever_made: usize,
    bytes_ever_requested: usize,
}

impl<T: HttpRangeClient> GenericHttpRangeClient<T> {
    pub fn new(url: &str) -> Self {
        GenericHttpRangeClient {
            client: T::new(),
            url: url.to_string(),
            requests_ever_made: 0,
            bytes_ever_requested: 0,
        }
    }
    pub async fn get_range(&mut self, begin: usize, length: usize) -> Result<Bytes> {
        self.requests_ever_made += 1;
        self.bytes_ever_requested += length;
        let range = format!("bytes={}-{}", begin, begin + length - 1);
        debug!(
            "request: #{}, bytes: (this_request: {}, ever: {}), Range: {}",
            self.requests_ever_made, length, self.bytes_ever_requested, range
        );
        self.client.get_range(&self.url, &range).await
    }
}
