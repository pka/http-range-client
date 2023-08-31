use crate::error::{HttpError, Result};
use bytes::Bytes;
use std::io::Read;
use std::time::Duration;

#[cfg(feature = "ureq-sync")]
pub(crate) mod sync {
    use super::*;
    use crate::range_client::SyncHttpRangeClient;

    impl SyncHttpRangeClient for ureq::Agent {
        fn get_range(&self, url: &str, range: &str) -> Result<Bytes> {
            let response = self.get(url).set("Range", range).call()?;
            if response.status() < 200 || response.status() > 299 {
                return Err(HttpError::HttpStatus(response.status()));
            }
            // Direct read via Bytes::from_iter ?
            let mut bytes: Vec<u8> = Vec::new();
            response
                .into_reader()
                .take(10_000_000)
                .read_to_end(&mut bytes)
                .map_err(|e| HttpError::HttpError(e.to_string()))?;
            Ok(Bytes::from(bytes))
        }
    }

    /// Sync HTTP client for HTTP Range requests with a buffer optimized for sequential requests.
    pub type UreqHttpReader = crate::SyncBufferedHttpRangeClient<ureq::Agent>;

    impl UreqHttpReader {
        pub fn new(url: &str) -> Self {
            let agent = ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build();
            Self::with(agent, url)
        }
    }
}

impl From<ureq::Error> for HttpError {
    fn from(error: ureq::Error) -> Self {
        use ureq::Error::*;
        match error {
            Status(status, _resp) => HttpError::HttpStatus(status),
            Transport(e) => HttpError::HttpError(e.to_string()),
        }
    }
}
