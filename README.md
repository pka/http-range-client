# http-range-client

HTTP client for HTTP Range requests with a buffer optimized for sequential requests.


Usage example:

    use http_range_client::*;

    let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
    let bytes = client.get_range(0, 3, 256).await?;
    assert_eq!(bytes, "fgb".as_bytes());
