#[macro_use]
extern crate log;

mod error;
mod http_client;
mod reqwest_client;

pub use error::Result;
pub use http_client::BufferedHttpRangeClient;
pub(crate) use reqwest_client::HttpClient;
