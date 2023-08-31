# http-range-client

[![CI build](https://github.com/pka/http-range-client/workflows/CI/badge.svg)](https://github.com/pka/http-range-client/actions)
[![crates.io version](https://img.shields.io/crates/v/http-range-client.svg)](https://crates.io/crates/http-range-client)
[![docs.rs docs](https://docs.rs/http-range-client/badge.svg)](https://docs.rs/http-range-client)

HTTP client for HTTP Range requests with a buffer optimized for sequential requests.

Implements Seek+Read for blocking clients, which makes it a drop-in replacement for local files.

Usage examples:

    use http_range_client::*;

    let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
    let bytes = client.min_req_size(256).get_range(0, 3).await?;
    assert_eq!(bytes, b"fgb");
    let version = client.get_bytes(1).await?; // From buffer - no HTTP request!
    assert_eq!(version, [3]);

    let mut client =
        HttpReader::new("https://www.rust-lang.org/static/images/favicon-32x32.png");
    client.seek(SeekFrom::Start(1)).ok();
    let mut bytes = [0; 3];
    client.read_exact(&mut bytes)?;
    assert_eq!(&bytes, b"PNG");
