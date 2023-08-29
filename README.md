# http-range-client

HTTP client for HTTP Range requests with a buffer optimized for sequential requests.


Usage example:

    use http_range_client::*;

    let mut client = BufferedHttpRangeClient::new("https://flatgeobuf.org/test/data/countries.fgb");
    let bytes = client.min_req_size(256).get_range(0, 3).await?;
    assert_eq!(bytes, "fgb".as_bytes());
    let version = client.get_bytes(1).await?; // From buffer - no HTTP request!
    assert_eq!(version, &[3]);
