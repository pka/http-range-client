use crate::error::{HttpError, Result};
use bytes::Bytes;
use std::io::Read;
use std::iter::FromIterator;
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
            // TODO: return error instead of dropping remaining bytes
            let bytes = response.into_reader().bytes().map_while(|val| val.ok());
            Ok(Bytes::from_iter(bytes))
        }
        fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>> {
            let response = self.head(url).call()?;
            Ok(response.header(header).map(|val| val.to_string()))
        }
    }

    /// Sync HTTP client for HTTP Range requests with a buffer optimized for sequential reading.
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
