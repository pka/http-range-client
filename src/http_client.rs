use crate::error::Result;
use crate::HttpClient;
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use std::cmp::max;
use std::str;

#[async_trait]
pub trait HttpRangeClient {
    fn new() -> Self;
    async fn get_range(&self, url: &str, range: &str) -> Result<Bytes>;
}

/// HTTP client for HTTP Range requests (https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests)
pub(crate) struct GenericHttpRangeClient<T: HttpRangeClient> {
    client: T,
    url: String,
    requests_ever_made: usize,
    bytes_ever_requested: usize,
}

impl<T: HttpRangeClient> GenericHttpRangeClient<T> {
    fn new(url: &str) -> Self {
        GenericHttpRangeClient {
            client: T::new(),
            url: url.to_string(),
            requests_ever_made: 0,
            bytes_ever_requested: 0,
        }
    }
    async fn get_range(&mut self, begin: usize, length: usize) -> Result<Bytes> {
        self.requests_ever_made += 1;
        self.bytes_ever_requested += length;
        let range = format!("bytes={}-{}", begin, begin + length - 1);
        debug!(
            "request: #{}, bytes: (this_request: {}, ever: {}), Range: {}",
            self.requests_ever_made, length, self.bytes_ever_requested, range
        );
        self.client.get_range(&self.url, &range).await
    }
}

/// HTTP client for HTTP Range requests with a buffer optimized for sequential requests
pub struct BufferedHttpRangeClient {
    http_client: HttpClient,
    buf: BytesMut,
    /// Lower index of buffer relative to input stream
    head: usize,
}

impl BufferedHttpRangeClient {
    pub fn new(url: &str) -> Self {
        BufferedHttpRangeClient {
            http_client: HttpClient::new(url),
            buf: BytesMut::new(),
            head: 0,
        }
    }

    pub async fn get_range(
        &mut self,
        begin: usize,
        length: usize,
        min_req_size: usize,
    ) -> Result<&[u8]> {
        //
        //            head  begin    tail
        //       +------+-----+---+---+------------+
        // File  |      |     |   |   |            |
        //       +------+-----+---+---+------------+
        // buf          |     |   |   |
        //              +-----+---+---+
        // Request            |   |
        //                    +---+
        //                    length

        // Download additional bytes if requested range is not in buffer
        if begin + length > self.tail() || begin < self.head {
            // Remove bytes before new begin
            if begin > self.head && begin < self.tail() {
                let _ = self.buf.split_to(begin - self.head);
                self.head = begin;
            } else if begin >= self.tail() || begin < self.head {
                self.buf.clear();
                self.head = begin;
            }

            // Read additional bytes into buffer
            let range_begin = max(begin, self.tail());
            let range_length = max(begin + length - range_begin, min_req_size);
            let bytes = self
                .http_client
                .get_range(range_begin, range_length)
                .await?;
            self.buf.put(bytes);
        }

        // Return slice from buffer
        let lower = begin - self.head;
        let upper = begin + length - self.head;
        Ok(&self.buf[lower..upper])
    }

    fn tail(&self) -> usize {
        self.head + self.buf.len()
    }
}
