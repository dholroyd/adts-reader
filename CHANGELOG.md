# Change Log

## Unreleased

### Fixed
 - `private_bit()` was reading the MSB of `channel_configuration` instead of the private bit
 - Fixed `originality()` logic, which previously did the opposite of what the spec wants

### Changed
 - Switched to Rust 2021 edition
