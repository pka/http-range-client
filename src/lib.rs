//! HTTP client for HTTP Range requests with a buffer optimized for sequential requests.

//! ## Usage example
//!
//! ```
//! use http_range_client::*;
//!
//! // Async API (without feature `sync`):
//! # #[cfg(not(feature = "sync"))]
//! # async fn get() -> Result<()> {
//! let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
//! let bytes = client.get_range(0, 3, 256).await?;
//! assert_eq!(bytes, "fgb".as_bytes());
//! # Ok(())
//! # }
//!
//! // Blocking API (with feature `sync`):
//! # #[cfg(feature = "sync")]
//! # fn get() -> Result<()> {
//! let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
//! let bytes = client.get_range(0, 3, 256)?;
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
#[cfg(all(feature = "reqwest", not(feature = "sync")))]
pub(crate) use reqwest_client::nonblocking::HttpClient;
#[cfg(all(feature = "reqwest", feature = "sync"))]
pub(crate) use reqwest_client::sync::HttpClient;
