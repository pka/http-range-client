## 0.8.0 (2024-08-12)

* Update reqwest to 0.12

## 0.7.2 (2023-09-10)

* Add head_response_header to public API
* Impl BufRead for SyncBufferedHttpRangeClient

## 0.7.1 (2023-09-02)

* Add ureq client support
* Return UnexpectedEof when read() causes HTTP error 416
* Support SeekFrom::End by sending a HEAD request
* Fix overlapping buffer request

## 0.7.0 (2023-08-31)

* Add sync API with blocking::reqwest
* Implement Seek+Read for sync reader
* Breaking: Replace `min_req_size` parameter in `get_range` with separate methods
* Add `get_bytes` method for sequential reading
* Export traits and adapter for external implementations
* Fix panic when requesting data after EOF
* Add feature flags

## 0.6.0 (2021-11-10)

* First publication on crates.io after extraction from FlatGeobuf
