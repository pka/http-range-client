use crate::error::Result;
use crate::HttpClient;
use bytes::{BufMut, BytesMut};
use std::cmp::max;
use std::str;

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

    fn get_request_range(
        &mut self,
        begin: usize,
        length: usize,
        min_req_size: usize,
    ) -> Option<(usize, usize)> {
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
            Some((range_begin, range_length))
        } else {
            None
        }
    }

    fn tail(&self) -> usize {
        self.head + self.buf.len()
    }

    /// Get `length` bytes with offset `begin`.
    ///
    /// When not already in buffer, request at least `min_req_size` bytes.
    #[cfg(not(feature = "sync"))]
    pub async fn get_range(
        &mut self,
        begin: usize,
        length: usize,
        min_req_size: usize,
    ) -> Result<&[u8]> {
        if let Some((range_begin, range_length)) =
            self.get_request_range(begin, length, min_req_size)
        {
            let bytes = self
                .http_client
                .get_range(range_begin, range_length)
                .await?;
            self.buf.put(bytes);
        }
        // Return slice from buffer
        let lower = begin - self.head;
        Ok(&self.buf[lower..lower + length])
    }

    /// Get `length` bytes with offset `begin`.
    ///
    /// When not already in buffer, request at least `min_req_size` bytes.
    #[cfg(feature = "sync")]
    pub fn get_range(&mut self, begin: usize, length: usize, min_req_size: usize) -> Result<&[u8]> {
        if let Some((range_begin, range_length)) =
            self.get_request_range(begin, length, min_req_size)
        {
            let bytes = self.http_client.get_range(range_begin, range_length)?;
            self.buf.put(bytes);
        }
        // Return slice from buffer
        let lower = begin - self.head;
        Ok(&self.buf[lower..lower + length])
    }
}

#[cfg(test)]
mod test {
    use crate::{BufferedHttpRangeClient, Result};

    #[cfg(not(feature = "sync"))]
    #[tokio::test]
    async fn http_read_async() -> Result<()> {
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.get_range(0, 3, 256).await?;
        assert_eq!(bytes, "fgb".as_bytes());
        Ok(())
    }

    #[cfg(feature = "sync")]
    #[test]
    fn http_read_sync() -> Result<()> {
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.get_range(0, 3, 256)?;
        assert_eq!(bytes, "fgb".as_bytes());
        Ok(())
    }
}
