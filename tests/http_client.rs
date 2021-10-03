use http_range_client::{BufferedHttpRangeClient, Result};
use tokio::runtime::Runtime;

async fn http_read_async() -> Result<()> {
    let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
    let mut client = BufferedHttpRangeClient::new(url);
    let bytes = client.get_range(0, 8, 16).await?;
    assert_eq!(bytes, [102, 103, 98, 3, 102, 103, 98, 0]);
    Ok(())
}

#[test]
fn http_read() {
    assert!(Runtime::new().unwrap().block_on(http_read_async()).is_ok());
}
