use crate::error::Result;
use crate::HttpClient;
use bytes::{BufMut, BytesMut};
use std::cmp::max;
use std::str;

/// HTTP client for HTTP Range requests with a buffer optimized for sequential requests
pub struct BufferedHttpRangeClient {
    http_client: HttpClient,
    buf: BytesMut,
    min_req_size: usize,
    /// Current position for Read+Seek implementation
    offset: usize,
    /// Lower index of buffer relative to input stream
    head: usize,
}

impl BufferedHttpRangeClient {
    pub fn new(url: &str) -> Self {
        BufferedHttpRangeClient {
            http_client: HttpClient::new(url),
            buf: BytesMut::new(),
            min_req_size: 1024,
            offset: 0,
            head: 0,
        }
    }

    /// Set minimal request size.
    pub fn set_min_req_size(&mut self, size: usize) {
        self.min_req_size = size;
    }

    /// Set minimal request size.
    pub fn min_req_size(&mut self, size: usize) -> &mut Self {
        self.set_min_req_size(size);
        self
    }

    fn tail(&self) -> usize {
        self.head + self.buf.len()
    }

    fn get_request_range(&mut self, begin: usize, length: usize) -> Option<(usize, usize)> {
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
            let range_length = max(begin + length - range_begin, self.min_req_size);
            Some((range_begin, range_length))
        } else {
            None
        }
    }
}

#[cfg(not(feature = "sync"))]
mod nonblocking {
    use super::*;

    impl BufferedHttpRangeClient {
        /// Get `length` bytes with offset `begin`.
        pub async fn get_range(&mut self, begin: usize, length: usize) -> Result<&[u8]> {
            if let Some((range_begin, range_length)) = self.get_request_range(begin, length) {
                let bytes = self
                    .http_client
                    .get_range(range_begin, range_length)
                    .await?;
                self.buf.put(bytes);
            }
            self.offset = begin + length;
            // Return slice from buffer
            let lower = begin - self.head;
            Ok(&self.buf[lower..lower + length])
        }

        /// Get `length` bytes from current offset.
        pub async fn get_bytes(&mut self, length: usize) -> Result<&[u8]> {
            self.get_range(self.offset, length).await
        }
    }
}

#[cfg(feature = "sync")]
mod sync {
    use super::*;
    use bytes::Buf;
    use std::io::{Read, Seek, SeekFrom};

    impl BufferedHttpRangeClient {
        /// Get `length` bytes with offset `begin`.
        pub fn get_range(&mut self, begin: usize, length: usize) -> Result<&[u8]> {
            if let Some((range_begin, range_length)) = self.get_request_range(begin, length) {
                let bytes = self.http_client.get_range(range_begin, range_length)?;
                self.buf.put(bytes);
            }
            self.offset = begin + length;
            // Return slice from buffer
            let lower = begin - self.head;
            Ok(&self.buf[lower..lower + length])
        }

        /// Get `length` bytes from current offset.
        pub fn get_bytes(&mut self, length: usize) -> Result<&[u8]> {
            self.get_range(self.offset, length)
        }
    }

    impl Read for BufferedHttpRangeClient {
        fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
            let length = buf.len();
            debug!("Offset: {}, Length: {length}", self.offset);
            let mut bytes = self.get_range(self.offset as usize, length).unwrap();
            bytes.copy_to_slice(buf);
            Ok(length)
        }
    }

    impl Seek for BufferedHttpRangeClient {
        fn seek(&mut self, pos: SeekFrom) -> std::result::Result<u64, std::io::Error> {
            match pos {
                SeekFrom::Start(p) => {
                    self.offset = p as usize;
                    Ok(p)
                }
                SeekFrom::End(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Request size unkonwn",
                )),
                SeekFrom::Current(p) => {
                    self.offset = self.offset.saturating_add_signed(p as isize);
                    Ok(self.offset as u64)
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "sync"))]
mod test_async {
    use crate::{BufferedHttpRangeClient, Result};

    #[tokio::test]
    async fn http_read_async() -> Result<()> {
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.min_req_size(256).get_range(0, 3).await?;
        assert_eq!(bytes, "fgb".as_bytes());
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "sync")]
mod test_sync {
    use crate::{BufferedHttpRangeClient, Result};
    use std::io::{Read, Seek, SeekFrom};

    #[test]
    fn http_read_sync() -> Result<()> {
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.min_req_size(256).get_range(0, 3)?;
        assert_eq!(bytes, "fgb".as_bytes());

        let version = client.get_bytes(1)?;
        assert_eq!(version, &[3]);

        let bytes = client.get_bytes(3)?;
        assert_eq!(bytes, "fgb".as_bytes());
        Ok(())
    }

    #[test]
    fn io_read() -> std::io::Result<()> {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();

        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        client.seek(SeekFrom::Start(3)).ok();
        let mut version = [0; 1];
        client.min_req_size(256).read_exact(&mut version)?;
        assert_eq!(&version, &[3]);

        let mut bytes = [0; 3];
        client.read_exact(&mut bytes)?;
        assert_eq!(&bytes, b"fgb");
        Ok(())
    }
}
