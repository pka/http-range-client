use crate::error::{HttpError, Result};
use bytes::Bytes;

#[cfg(feature = "reqwest-async")]
pub(crate) mod nonblocking {
    use super::*;
    use crate::range_client::AsyncHttpRangeClient;
    use async_trait::async_trait;

    #[cfg(not(target_arch = "wasm32"))]
    #[async_trait]
    impl AsyncHttpRangeClient for reqwest::Client {
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
    impl AsyncHttpRangeClient for reqwest::Client {
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

    /// Async HTTP client for HTTP Range requests with a buffer optimized for sequential requests.
    pub type BufferedHttpRangeClient = crate::AsyncBufferedHttpRangeClient<reqwest::Client>;

    impl BufferedHttpRangeClient {
        pub fn new(url: &str) -> Self {
            Self::with(reqwest::Client::new(), url)
        }
    }
}

#[cfg(feature = "reqwest-sync")]
pub(crate) mod sync {
    use super::*;
    use crate::range_client::SyncHttpRangeClient;

    impl SyncHttpRangeClient for reqwest::blocking::Client {
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

    /// Sync HTTP client for HTTP Range requests with a buffer optimized for sequential requests.
    pub type HttpReader = crate::SyncBufferedHttpRangeClient<reqwest::blocking::Client>;

    impl HttpReader {
        pub fn new(url: &str) -> Self {
            Self::with(reqwest::blocking::Client::new(), url)
        }
    }
}
