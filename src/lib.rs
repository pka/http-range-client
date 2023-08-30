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
//! let bytes = client.min_req_size(256).get_range(0, 3).await?;
//! assert_eq!(bytes, b"fgb");
//! let version = client.get_bytes(1).await?; // From buffer - no HTTP request!
//! assert_eq!(version, [3]);
//! # Ok(())
//! # }
//!
//! // Blocking API (with feature `sync`):
//! # #[cfg(feature = "sync")]
//! # fn get() -> Result<()> {
//! let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
//! let bytes = client.min_req_size(256).get_range(0, 3)?;
//! assert_eq!(bytes, b"fgb");
//! let version = client.get_bytes(1)?; // From buffer - no HTTP request!
//! assert_eq!(version, [3]);
//! # Ok(())
//! # }
//!
//! // Seek+Read API (with feature `sync`):
//! # #[cfg(feature = "sync")]
//! # fn read() -> std::io::Result<()> {
//! use std::io::{Read, Seek, SeekFrom};
//! let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
//! client.seek(SeekFrom::Start(3)).ok();
//! let mut version = [0; 1];
//! client.min_req_size(256).read_exact(&mut version)?;
//! assert_eq!(version, [3]);
//! let mut bytes = [0; 3];
//! client.read_exact(&mut bytes)?;
//! assert_eq!(&bytes, b"fgb");
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
