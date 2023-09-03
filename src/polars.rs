use crate::{SyncBufferedHttpRangeClient, SyncHttpRangeClient};
use polars_io::mmap::MmapBytesReader;

impl<T: SyncHttpRangeClient + Send + Sync> MmapBytesReader for SyncBufferedHttpRangeClient<T> {}
// Polars requires MmapBytesReader::to_bytes or to_file, which doesn't make sense for SyncHttpRangeClient
