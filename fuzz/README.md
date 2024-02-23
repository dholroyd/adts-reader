# Fuzz testing `adts-reader`

## Setup

If you don't already have [cargo-fuzz](https://rust-fuzz.github.io/book/introduction.html) installed.

```
cargo +nightly install cargo-fuzz
```

## Testing

To perform fuzz testing,

```
cargo +nightly fuzz run fuzz_target_1
```

(The fuzz test will keep running until it either finds a fault, or you kill the process.)