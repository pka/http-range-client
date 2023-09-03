use crate::{SyncBufferedHttpRangeClient, SyncHttpRangeClient};
use bytes::Bytes;
use parquet::errors::Result;
use parquet::file::reader::{ChunkReader, Length};
use std::io::{Seek, SeekFrom};

impl<T: SyncHttpRangeClient + Clone> Length for SyncBufferedHttpRangeClient<T> {
    fn len(&self) -> u64 {
        self.length_info.unwrap_or(Some(0)).unwrap_or(0)
    }
}

impl<T: SyncHttpRangeClient + Clone + Send + Sync> ChunkReader for SyncBufferedHttpRangeClient<T> {
    type T = SyncBufferedHttpRangeClient<T>;
    fn get_read(&self, start: u64) -> Result<Self::T> {
        let mut client = (*self).clone();
        client.seek(SeekFrom::Start(start)).unwrap();
        Ok(client)
    }
    fn get_bytes(&self, start: u64, length: usize) -> Result<Bytes> {
        let mut client = self.clone();
        let bytes = client.get_range(start as usize, length).unwrap();
        Ok(Bytes::copy_from_slice(bytes))
    }
}
