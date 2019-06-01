# adts-reader
A Rust parser for the [Audio Data Transport Stream](https://wiki.multimedia.cx/index.php/ADTS)
framing format used to carry encoded AAC audio data.

[![crates.io version](https://img.shields.io/crates/v/adts-reader.svg)](https://crates.io/crates/adts-reader)
[![Documentation](https://docs.rs/adts-reader/badge.svg)](https://docs.rs/adts-reader)

👉 **NB** This is not an AAC decoder, nor is it able to parse the syntax of the AAC bitstream within the ADTS payload.

Calling code should,
 - Provide an implementation of `AdtsConsumer` which will recieve callbacks as ADTS frame payloads are found
 - Pass buffers containing ADTS data into the `AdtsParser::push()` method

## Incremental parsing
The byte slice passed to `push()` need not end exactly at the boundry of an ADTS frame.  Partial ADTS data
remaining at the end of the given slice will be buffered internally in the parser, and the continuation of the ADTS
data must be provided in the subsequent call to `push()`.  This construction is intended to make it convinient to pass
payloads of the individual _MPEG Transport Stream_ packets in which ADTS data is commonly embedded, without having to
pay the cost of reassembling entire _PES packets_.

## Encoder configuration
ADTS frames include header data indicating the AAC encoder configuration, which will be made available to the calling
code through the provided implementation of `AdtsConsumer::new_config()`.

Configuration data is provided at stream start, and to simplify calling code the parser will only call
`AdtsConsumer::new_config()` again if and when the audio configuration is found to change.

## Supported ADTS syntax

 * Fixed header fields
   * [x] `mpeg_version`
   * [x] `protection`
   * [x] `audio_object_type`
   * [x] `sampling_frequency`
   * [x] `private_bit`
   * [x] `channel_configuration`
   * [x] `originality`
   * [x] `home`
 * Variable header fields data
   * [ ] `copyright_identifier` / `copyright_number` - deriving these values is not supported, since I've not seen any
     example bitstreams that use them
   * [x] `buffer_fullness`
   * [x] `number_of_blocks`
   * [ ] `crc` - not currently available (also, the CRC does not apply to all payload bytes; _cyclic reduncancy check_
     requires AAC bitstream parsing)
 * AAC payload data
   * A `&[u8]` byte slice containing a complete ADTS frame payload (which might be composed of one or more AAC blocks,
     per _number_of_blocks_)
