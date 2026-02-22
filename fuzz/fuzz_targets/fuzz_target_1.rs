#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate adts_reader;

use adts_reader::*;

struct NullConsumer {}

impl AdtsConsumer for NullConsumer {
    fn new_config(
        &mut self,
        mpeg_version: MpegVersion,
        protection: ProtectionIndicator,
        aot: AudioObjectType,
        freq: SamplingFrequency,
        private_bit: u8,
        channels: ChannelConfiguration,
        originality: Originality,
        home: u8,
    ) {
    }
    fn payload(&mut self, buffer_fullness: BufferFullness, number_of_blocks: u8, buf: &[u8]) {}
    fn error(&mut self, err: AdtsParseError) {}
}

fuzz_target!(|data: &[u8]| {
    let mut p = AdtsParser::new(NullConsumer {});
    p.push(data);
});
