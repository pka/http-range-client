use crate::error::Result;
use bytes::{BufMut, BytesMut};
use log::{debug, trace};
use std::cmp::{max, min};
use std::str;

/// Buffer for Range request reader (https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests)
struct HttpRangeBuffer {
    buf: BytesMut,
    min_req_size: usize,
    /// Current position for Read+Seek implementation
    offset: usize,
    /// Lower index of buffer relative to input stream
    head: usize,
}

impl HttpRangeBuffer {
    pub fn new() -> Self {
        HttpRangeBuffer {
            buf: BytesMut::new(),
            min_req_size: 1024,
            offset: 0,
            head: 0,
        }
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

        trace!("read begin: {begin}, Length: {length}");
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

pub(crate) mod nonblocking {
    use super::*;
    use crate::range_client::AsyncHttpRangeClient;

    /// HTTP client adapter for HTTP Range requests with a buffer optimized for sequential requests
    pub struct AsyncBufferedHttpRangeClient<T: AsyncHttpRangeClient> {
        http_client: T,
        url: String,
        buffer: HttpRangeBuffer,
        stats: stats::RequestStats,
    }

    impl<T: AsyncHttpRangeClient> AsyncBufferedHttpRangeClient<T> {
        pub fn with(http_client: T, url: &str) -> AsyncBufferedHttpRangeClient<T> {
            AsyncBufferedHttpRangeClient {
                http_client,
                url: url.to_string(),
                buffer: HttpRangeBuffer::new(),
                stats: stats::RequestStats::default(),
            }
        }

        /// Set minimal request size.
        pub fn set_min_req_size(&mut self, size: usize) {
            self.buffer.min_req_size = size;
        }

        /// Set minimal request size.
        pub fn min_req_size(&mut self, size: usize) -> &mut Self {
            self.set_min_req_size(size);
            self
        }

        fn range(&mut self, begin: usize, length: usize) -> String {
            let range = format!("bytes={begin}-{}", begin + length - 1);
            self.stats.log_get_range(begin, length, &range);
            range
        }

        /// Get `length` bytes with offset `begin`.
        pub async fn get_range(&mut self, begin: usize, length: usize) -> Result<&[u8]> {
            let slice_len = if let Some((range_begin, range_length)) =
                self.buffer.get_request_range(begin, length)
            {
                let range = self.range(range_begin, range_length);
                let bytes = self.http_client.get_range(&self.url, &range).await?;
                let len = bytes.len();
                self.buffer.buf.put(bytes);
                min(len, length)
            } else {
                length
            };
            self.buffer.offset = begin + slice_len;
            // Return slice from buffer
            let lower = begin - self.buffer.head;
            Ok(&self.buffer.buf[lower..lower + slice_len])
        }

        /// Get `length` bytes from current offset.
        pub async fn get_bytes(&mut self, length: usize) -> Result<&[u8]> {
            self.get_range(self.buffer.offset, length).await
        }
    }
}

pub(crate) mod stats {
    use super::*;

    #[derive(Default)]
    pub(crate) struct RequestStats {
        requests_ever_made: usize,
        bytes_ever_requested: usize,
    }

    impl RequestStats {
        pub fn log_get_range(&mut self, _begin: usize, length: usize, range: &str) {
            self.requests_ever_made += 1;
            self.bytes_ever_requested += length;
            debug!(
                "request: #{}, bytes: (this_request: {length}, ever: {}), Range: {range}",
                self.requests_ever_made, self.bytes_ever_requested,
            );
        }
    }
}

pub(crate) mod sync {
    use super::*;
    use crate::range_client::SyncHttpRangeClient;
    use crate::HttpError;
    use bytes::Buf;
    use std::io::{Read, Seek, SeekFrom};

    /// HTTP client adapter for HTTP Range requests with a buffer optimized for sequential requests
    pub struct SyncBufferedHttpRangeClient<T: SyncHttpRangeClient> {
        http_client: T,
        url: String,
        buffer: HttpRangeBuffer,
        stats: stats::RequestStats,
    }

    impl<T: SyncHttpRangeClient> SyncBufferedHttpRangeClient<T> {
        pub fn with(http_client: T, url: &str) -> SyncBufferedHttpRangeClient<T> {
            SyncBufferedHttpRangeClient {
                http_client,
                url: url.to_string(),
                buffer: HttpRangeBuffer::new(),
                stats: stats::RequestStats::default(),
            }
        }

        /// Set minimal request size.
        pub fn set_min_req_size(&mut self, size: usize) {
            self.buffer.min_req_size = size;
        }

        /// Set minimal request size.
        pub fn min_req_size(&mut self, size: usize) -> &mut Self {
            self.set_min_req_size(size);
            self
        }

        fn range(&mut self, begin: usize, length: usize) -> String {
            let range = format!("bytes={begin}-{}", begin + length - 1);
            self.stats.log_get_range(begin, length, &range);
            range
        }

        /// Get `length` bytes with offset `begin`.
        pub fn get_range(&mut self, begin: usize, length: usize) -> Result<&[u8]> {
            let slice_len = if let Some((range_begin, range_length)) =
                self.buffer.get_request_range(begin, length)
            {
                let range = self.range(range_begin, range_length);
                let bytes = self.http_client.get_range(&self.url, &range)?;
                let len = bytes.len();
                self.buffer.buf.put(bytes);
                min(len, length)
            } else {
                length
            };
            self.buffer.offset = begin + slice_len;
            // Return slice from buffer
            let lower = begin - self.buffer.head;
            Ok(&self.buffer.buf[lower..lower + slice_len])
        }

        /// Get `length` bytes from current offset.
        pub fn get_bytes(&mut self, length: usize) -> Result<&[u8]> {
            self.get_range(self.buffer.offset, length)
        }
    }

    impl<T: SyncHttpRangeClient> Read for SyncBufferedHttpRangeClient<T> {
        fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
            let length = buf.len();
            let mut bytes = self
                .get_range(self.buffer.offset, length)
                .map_err(|e| match e {
                    HttpError::HttpStatus(416) => {
                        std::io::Error::from(std::io::ErrorKind::UnexpectedEof)
                    }
                    e => std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                })?;
            // TODO: return  for HTTP status 416
            bytes.copy_to_slice(&mut buf[0..bytes.len()]);
            Ok(length)
        }
    }

    impl<T: SyncHttpRangeClient> Seek for SyncBufferedHttpRangeClient<T> {
        fn seek(&mut self, pos: SeekFrom) -> std::result::Result<u64, std::io::Error> {
            match pos {
                SeekFrom::Start(p) => {
                    self.buffer.offset = p as usize;
                    Ok(p)
                }
                SeekFrom::End(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Request size unkonwn",
                )),
                SeekFrom::Current(p) => {
                    self.buffer.offset = self.buffer.offset.saturating_add_signed(p as isize);
                    Ok(self.buffer.offset as u64)
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "reqwest-async")]
mod test_async {
    use crate::{AsyncBufferedHttpRangeClient, BufferedHttpRangeClient, Result};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn http_read_async() -> Result<()> {
        init_logger();
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.min_req_size(256).get_range(0, 3).await?;
        assert_eq!(bytes, b"fgb");
        let version = client.get_bytes(1).await?;
        assert_eq!(version, [3]);
        Ok(())
    }

    #[tokio::test]
    async fn read_over_min_req_size() -> Result<()> {
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.min_req_size(4).get_range(0, 8).await?;
        assert_eq!(bytes, [b'f', b'g', b'b', 3, b'f', b'g', b'b', 0]);
        Ok(())
    }

    #[tokio::test]
    async fn zero_range() -> Result<()> {
        init_logger();
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.get_range(100, 0).await?;
        assert_eq!(bytes, []);
        Ok(())
    }

    #[tokio::test]
    async fn after_end() -> Result<()> {
        init_logger();
        // countries.fgb has 205680 bytes
        let mut client =
            BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.get_range(205670, 10).await?;
        assert_eq!(bytes, [78, 192, 205, 204, 204, 204, 204, 236, 73, 192]);

        let bytes = client.get_bytes(10).await;
        assert_eq!(&bytes.unwrap_err().to_string(), "http status 416");

        let bytes = client.get_range(205670, 20).await?;
        assert_eq!(bytes, [78, 192, 205, 204, 204, 204, 204, 236, 73, 192]);

        Ok(())
    }

    #[tokio::test]
    async fn custom_headers() -> Result<()> {
        init_logger();
        let http_client = reqwest::Client::builder()
            .user_agent("rust-client")
            .build()
            .unwrap();
        let mut client = AsyncBufferedHttpRangeClient::with(
            http_client,
            "https://flatgeobuf.org/test/data/countries.fgb",
        );
        let bytes = client.min_req_size(256).get_range(0, 3).await?;
        assert_eq!(bytes, b"fgb");
        Ok(())
    }
}

#[cfg(test)]
#[cfg(any(feature = "reqwest-sync", feature = "ureq-sync"))]
mod test_sync {
    #[cfg(feature = "reqwest-sync")]
    use crate::HttpReader;
    use crate::Result;
    #[cfg(all(feature = "ureq-sync", not(feature = "reqwest-sync")))]
    use crate::UreqHttpReader as HttpReader;
    use std::io::{Read, Seek, SeekFrom};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn http_read_sync() -> Result<()> {
        init_logger();
        let mut client = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.min_req_size(256).get_range(0, 3)?;
        assert_eq!(bytes, b"fgb");

        let version = client.get_bytes(1)?;
        assert_eq!(version, [3]);

        let bytes = client.get_bytes(3)?;
        assert_eq!(bytes, b"fgb");
        Ok(())
    }

    #[test]
    fn http_read_sync_zero_range() -> Result<()> {
        init_logger();
        let mut client = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
        let bytes = client.min_req_size(256).get_range(0, 0)?;
        assert_eq!(bytes, []);
        Ok(())
    }

    #[test]
    fn io_read() -> std::io::Result<()> {
        init_logger();
        let mut reader = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
        reader.seek(SeekFrom::Start(3)).ok();
        let mut version = [0; 1];
        reader.min_req_size(256).read_exact(&mut version)?;
        assert_eq!(version, [3]);

        let mut bytes = [0; 3];
        reader.read_exact(&mut bytes)?;
        assert_eq!(&bytes, b"fgb");
        Ok(())
    }

    #[test]
    fn io_read_over_min_req_size() -> std::io::Result<()> {
        init_logger();
        let mut reader = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
        let mut bytes = [0; 8];
        reader.min_req_size(4).read_exact(&mut bytes)?;
        assert_eq!(bytes, [b'f', b'g', b'b', 3, b'f', b'g', b'b', 0]);
        Ok(())
    }

    #[test]
    fn io_read_non_exact() -> std::io::Result<()> {
        init_logger();
        let mut reader = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
        let mut bytes = [0; 8];
        // We could only read 4 bytes in this case
        reader.min_req_size(4).read(&mut bytes)?;
        assert_eq!(bytes, [b'f', b'g', b'b', 3, b'f', b'g', b'b', 0]);
        Ok(())
    }

    #[test]
    fn after_end() -> std::io::Result<()> {
        init_logger();
        // countries.fgb has 205680 bytes
        let mut reader = HttpReader::new("https://flatgeobuf.org/test/data/countries.fgb");
        reader.seek(SeekFrom::Start(205670)).ok();
        let mut bytes = [0; 10];
        reader.read_exact(&mut bytes)?;
        assert_eq!(bytes, [78, 192, 205, 204, 204, 204, 204, 236, 73, 192]);

        let result = reader.read_exact(&mut bytes);
        assert_eq!(result.unwrap_err().to_string(), "unexpected end of file");

        reader.seek(SeekFrom::Start(205670)).ok();
        let mut bytes = [0; 20];
        reader.read_exact(&mut bytes)?;
        assert_eq!(
            bytes,
            [78, 192, 205, 204, 204, 204, 204, 236, 73, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        Ok(())
    }

    #[test]
    fn remote_png() -> std::io::Result<()> {
        init_logger();
        let mut reader =
            HttpReader::new("https://www.rust-lang.org/static/images/favicon-32x32.png");
        reader.seek(SeekFrom::Start(1)).ok();
        let mut bytes = [0; 3];
        reader.read_exact(&mut bytes)?;
        assert_eq!(&bytes, b"PNG");
        Ok(())
    }
}
