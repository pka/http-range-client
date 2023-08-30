use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::str;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
/// Async HTTP client for Range requests
pub trait AsyncHttpRangeClient {
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
/// Async HTTP client for Range requests
pub trait AsyncHttpRangeClient {
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
}

/// Sync HTTP client for Range requests
pub trait SyncHttpRangeClient {
    fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
}
