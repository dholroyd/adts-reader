# Change Log

## Unreleased

### Fixed
 - `private_bit()` was reading the MSB of `channel_configuration` instead of the private bit
 - Fixed `originality()` logic, which previously did the opposite of what the spec wants
 - Fixed `adts_buffer_fullness()` incorrectly dropping the 3 most significant bits of the 11-bit field
 - `AdtsParser` config change detection should now detect changes in all config fields, rather than just a subset

### Changed
 - **Breaking:** `adts_buffer_fullness()` now returns `BufferFullness` enum
   instead of `u16`, distinguishing VBR from the other (CBR) values.
 - Switched to Rust 2021 edition
