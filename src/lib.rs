//! HTTP client for HTTP Range requests with a buffer optimized for sequential requests.

//! ## Usage example
//!
//! ```
//! use http_range_client::*;
//!
//! # async fn get() -> Result<()> {
//! let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
//! let bytes = client.get_range(0, 3, 256).await?;
//! assert_eq!(bytes, "fgb".as_bytes());
//! # Ok(())
//! # }
//! ```

#[macro_use]
extern crate log;

mod buffered_range_client;
mod error;
mod range_client;
#[cfg(feature = "reqwest")]
mod reqwest_client;

pub use buffered_range_client::BufferedHttpRangeClient;
pub use error::*;
#[cfg(feature = "reqwest")]
pub(crate) use reqwest_client::HttpClient;
