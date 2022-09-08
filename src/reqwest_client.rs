use crate::error::{HttpError, Result};
use crate::range_client::{GenericHttpRangeClient, HttpRangeClient};
use async_trait::async_trait;
use bytes::Bytes;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl HttpRangeClient for reqwest::Client {
    fn new() -> Self {
        reqwest::Client::new()
    }
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes> {
        let response = self
            .get(url)
            .header("Range", range)
            .send()
            .await
            .map_err(|e| HttpError::HttpError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(HttpError::HttpStatus(response.status().as_u16()));
        }
        response
            .bytes()
            .await
            .map_err(|e| HttpError::HttpError(e.to_string()))
    }
}


#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl HttpRangeClient for reqwest::Client {
    fn new() -> Self {
        reqwest::Client::new()
    }
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes> {
        let response = self
            .get(url)
            .header("Range", range)
            .send()
            .await
            .map_err(|e| HttpError::HttpError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(HttpError::HttpStatus(response.status().as_u16()));
        }
        response
            .bytes()
            .await
            .map_err(|e| HttpError::HttpError(e.to_string()))
    }
}

pub(crate) type HttpClient = GenericHttpRangeClient<reqwest::Client>;
