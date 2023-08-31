//! HTTP client for HTTP Range requests with a buffer optimized for sequential requests.

//! ## Usage example
//!
//! ```
//! use http_range_client::*;
//!
//! // Async API
//! # #[cfg(feature = "reqwest-async")]
//! # async fn get_async() -> Result<()> {
//! let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
//! let bytes = client.min_req_size(256).get_range(0, 3).await?;
//! assert_eq!(bytes, b"fgb");
//! let version = client.get_bytes(1).await?; // From buffer - no HTTP request!
//! assert_eq!(version, [3]);
//! # Ok(())
//! # }
//!
//! // Blocking API
//! # #[cfg(feature = "reqwest-sync")]
//! # fn get_sync() -> Result<()> {
//! let mut client = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
//! let bytes = client.min_req_size(256).get_range(0, 3)?;
//! assert_eq!(bytes, b"fgb");
//! let version = client.get_bytes(1)?; // From buffer - no HTTP request!
//! assert_eq!(version, [3]);
//! # Ok(())
//! # }
//!
//! // Seek+Read API
//! # #[cfg(feature = "reqwest-sync")]
//! # fn read() -> std::io::Result<()> {
//! use std::io::{Read, Seek, SeekFrom};
//! let mut reader = HttpReader::new("https://www.rust-lang.org/static/images/favicon-32x32.png");
//! reader.seek(SeekFrom::Start(1)).ok();
//! let mut bytes = [0; 3];
//! reader.read_exact(&mut bytes)?;
//! assert_eq!(&bytes, b"PNG");
//! # Ok(())
//! # }
//! ```

mod buffered_range_client;
mod error;
mod range_client;
#[cfg(any(feature = "reqwest-async", feature = "reqwest-sync"))]
mod reqwest_client;

pub use buffered_range_client::nonblocking::AsyncBufferedHttpRangeClient;
pub use buffered_range_client::sync::SyncBufferedHttpRangeClient;
pub use error::*;
pub use range_client::*;

#[cfg(feature = "reqwest-async")]
pub use crate::reqwest_client::nonblocking::BufferedHttpRangeClient;
#[cfg(feature = "reqwest-sync")]
pub use crate::reqwest_client::sync::HttpReader;
