#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate adts_reader;

use adts_reader::*;

struct NullConsumer {}

impl AdtsConsumer for NullConsumer {
    fn new_config(
        &mut self,
        _mpeg_version: MpegVersion,
        _protection: ProtectionIndicator,
        _aot: AudioObjectType,
        _freq: SamplingFrequencyIndex,
        _private_bit: u8,
        _channels: ChannelConfiguration,
        _originality: Originality,
        _home: u8,
    ) {
    }
    fn payload(&mut self, _buffer_fullness: BufferFullness, _number_of_blocks: u8, _buf: &[u8]) {}
    fn error(&mut self, _err: AdtsParseError) {}
}

fuzz_target!(|data: &[u8]| {
    let mut p = AdtsParser::new(NullConsumer {});
    p.push(data);
});
