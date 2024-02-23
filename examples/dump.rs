extern crate adts_reader;
extern crate hexdump;

use adts_reader::*;
use std::env;
use std::fs::File;
use std::io;

struct DumpAdtsConsumer {
    frame_count: usize,
}
impl AdtsConsumer for DumpAdtsConsumer {
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
        println!("New ADTS configuration found");
        println!(
            "{:?} {:?} {:?} {:?} private_bit={} {:?} {:?} home={}",
            mpeg_version, protection, aot, freq, private_bit, channels, originality, home
        );
    }
    fn payload(&mut self, buffer_fullness: u16, number_of_blocks: u8, buf: &[u8]) {
        println!(
            "ADTS Frame buffer_fullness={} blocks={}",
            buffer_fullness, number_of_blocks
        );
        hexdump::hexdump(buf);
        self.frame_count += 1;
    }
    fn error(&mut self, err: AdtsParseError) {
        println!("Error: {:?}", err);
    }
}

fn run<R>(mut r: R) -> io::Result<()>
where
    R: io::Read,
    R: Sized,
{
    const LEN: usize = 1024 * 1024;
    let mut buf = [0u8; LEN];
    let mut byte_count = 0;
    let mut parser = AdtsParser::new(DumpAdtsConsumer { frame_count: 0 });
    loop {
        match r.read(&mut buf[..])? {
            0 => break,
            n => {
                let target = &mut buf[0..n];
                parser.push(target);
                byte_count += n;
            }
        };
    }
    println!(
        "Processed {} bytes, {} ADTS frames",
        byte_count, parser.consumer.frame_count
    );
    Ok(())
}

fn main() {
    let mut args = env::args();
    args.next();
    let name = args.next().unwrap();
    let f = File::open(&name).expect(&format!("file not found: {}", &name));
    run(f).expect(&format!("error reading {}", &name));
}
