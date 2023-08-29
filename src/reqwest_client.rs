use crate::error::{HttpError, Result};
use crate::range_client::{GenericHttpRangeClient, HttpRangeClient};
use bytes::Bytes;

#[cfg(not(feature = "sync"))]
pub(crate) mod nonblocking {
    use super::*;
    use async_trait::async_trait;

    #[cfg(not(target_arch = "wasm32"))]
    #[async_trait]
    impl HttpRangeClient for reqwest::Client {
        fn new() -> Self {
            Self::new()
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
            Self::new()
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
}

#[cfg(feature = "sync")]
pub(crate) mod sync {
    use super::*;

    impl HttpRangeClient for reqwest::blocking::Client {
        fn new() -> Self {
            Self::new()
        }
        fn get_range(&self, url: &str, range: &str) -> Result<Bytes> {
            let response = self
                .get(url)
                .header("Range", range)
                .send()
                .map_err(|e| HttpError::HttpError(e.to_string()))?;
            if !response.status().is_success() {
                return Err(HttpError::HttpStatus(response.status().as_u16()));
            }
            response
                .bytes()
                .map_err(|e| HttpError::HttpError(e.to_string()))
        }
    }

    pub(crate) type HttpClient = GenericHttpRangeClient<reqwest::blocking::Client>;
}
