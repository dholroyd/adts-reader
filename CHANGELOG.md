# Change Log

## Unreleased

### Fixed
 - `private_bit()` was reading the MSB of `channel_configuration` instead of the private bit
 - Fixed `originality()` logic, which previously did the opposite of what the spec wants
 - Fixed `adts_buffer_fullness()` incorrectly dropping the 3 most significant bits of the 11-bit field
 - `AdtsParser` config change detection should now detect changes in all config fields, rather than just a subset
 - `AdtsParser` no longer enters a permanent error state when accumulating a partial header across multiple `push()` calls (e.g. CRC header split across buffers)

### Changed
 - **Breaking:** `AdtsConsumer` trait replaces `payload()` with streaming
   callbacks: `frame_start()`, `frame_body()`, `frame_complete()`. Payload
   data is now delivered as zero-copy sub-slices of the input buffer, and
   `frame_body()` may be called multiple times per frame when data spans
   `push()` boundaries.  The more complex API supports better performance
   in typical MPEG-TS parsing workloads.
 - **Breaking:** `adts_buffer_fullness()` now returns `BufferFullness` enum
   instead of `u16`, distinguishing VBR from the other (CBR) values.
 - **Breaking:** Replaced local `AudioObjectType`, `ChannelConfiguration`, and
  `SamplingFrequency` definitions with re-exports of similar but incompatible
   types from `mpeg4-audio-const` .
 - Switched to Rust 2021 edition
