# Change Log

## Unreleased

### Fixed
 - `private_bit()` was reading the MSB of `channel_configuration` instead of the private bit
 - Fixed `originality()` logic, which previously did the opposite of what the spec wants
 - Fixed `adts_buffer_fullness()` incorrectly dropping the 3 most significant bits of the 11-bit field

### Changed
 - Switched to Rust 2021 edition
