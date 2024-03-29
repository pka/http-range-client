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
            let response = self.get(url).header("Range", range).send().await?;
            if !response.status().is_success() {
                return Err(HttpError::HttpStatus(response.status().as_u16()));
            }
            response
                .bytes()
                .await
                .map_err(|e| HttpError::HttpError(e.to_string()))
        }
        async fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>> {
            let response = self.head(url).send().await?;
            if let Some(val) = response.headers().get(header) {
                let v = val
                    .to_str()
                    .map_err(|e| HttpError::HttpError(e.to_string()))?;
                Ok(Some(v.to_string()))
            } else {
                Ok(None)
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[async_trait(?Send)]
    impl AsyncHttpRangeClient for reqwest::Client {
        async fn get_range(&self, url: &str, range: &str) -> Result<Bytes> {
            let response = self.get(url).header("Range", range).send().await?;
            if !response.status().is_success() {
                return Err(HttpError::HttpStatus(response.status().as_u16()));
            }
            response
                .bytes()
                .await
                .map_err(|e| HttpError::HttpError(e.to_string()))
        }
        async fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>> {
            let response = self.head(url).send().await?;
            if let Some(val) = response.headers().get(header) {
                let v = val
                    .to_str()
                    .map_err(|e| HttpError::HttpError(e.to_string()))?;
                Ok(Some(v.to_string()))
            } else {
                Ok(None)
            }
        }
    }

    /// Async HTTP client for HTTP Range requests with a buffer optimized for sequential reading.
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
            let response = self.get(url).header("Range", range).send()?;
            if !response.status().is_success() {
                return Err(HttpError::HttpStatus(response.status().as_u16()));
            }
            response
                .bytes()
                .map_err(|e| HttpError::HttpError(e.to_string()))
        }
        fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>> {
            let response = self.head(url).send()?;
            if let Some(val) = response.headers().get(header) {
                let v = val
                    .to_str()
                    .map_err(|e| HttpError::HttpError(e.to_string()))?;
                Ok(Some(v.to_string()))
            } else {
                Ok(None)
            }
        }
    }

    /// Sync HTTP client for HTTP Range requests with a buffer optimized for sequential reading.
    pub type HttpReader = crate::SyncBufferedHttpRangeClient<reqwest::blocking::Client>;

    impl HttpReader {
        pub fn new(url: &str) -> Self {
            Self::with(reqwest::blocking::Client::new(), url)
        }
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            HttpError::HttpStatus(status.as_u16())
        } else {
            HttpError::HttpError(error.to_string())
        }
    }
}
