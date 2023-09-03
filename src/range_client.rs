use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::str;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
/// Async HTTP client for Range requests
pub trait AsyncHttpRangeClient {
    /// Send a GET range request
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
    /// Send a HEAD request and return response header value
    async fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
/// Async HTTP client for Range requests
pub trait AsyncHttpRangeClient {
    /// Send a GET range request
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
    /// Send a HEAD request and return response header value
    async fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>>;
}

/// Sync HTTP client for Range requests
pub trait SyncHttpRangeClient {
    /// Send a GET range request
    fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
    /// Send a HEAD request and return response header value
    fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>>;
}
