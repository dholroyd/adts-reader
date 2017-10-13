extern crate adts_reader;
extern crate hexdump;

use adts_reader::AdtsHeader;
use adts_reader::AdtsHeaderError;
use std::env;
use std::fs::File;
use std::io;

fn parse(buf: &[u8], count: &mut u64) -> Result<usize, AdtsHeaderError> {
    let mut pos = 0;
    while pos < buf.len() {
        let h = AdtsHeader::from_bytes(&buf[pos..])?;
        let new_pos = pos + h.frame_length() as usize;
        if new_pos > buf.len() {
            return Ok(buf.len() - pos);
        }
        println!("{}:{:#x} {:?} {:?} {:?} {:?} private_bit={} {:?} {:?} home={} copyright_bit={} id_start={:?} frame_length={} buffer_fullness={} blocks={}",
                 count,
                 pos,
                 h.mpeg_version(),
                 h.protection(),
                 h.audio_object_type(),
                 h.sampling_frequency(),
                 h.private_bit(),
                 h.channel_configuration(),
                 h.originality(),
                 h.home(),
                 h.copyright_identification_bit(),
                 h.copyright_identification_start(),
                 h.frame_length(),
                 h.adts_buffer_fullness(),
                 h.number_of_raw_data_blocks_in_frame());
        if let Ok(payload) = h.payload() {
            hexdump::hexdump(payload);
        }
        pos = new_pos;
        *count += 1;
    }
    Ok(0)
}

fn run<R>(mut r: R) -> io::Result<()>
    where R: io::Read, R: Sized
{
    const LEN: usize = 1024*1024;
    let mut buf = [0u8; LEN];
    let reading = true;
    let mut start = 0;
    let mut count = 0;
    while reading {
        match r.read(&mut buf[start..])? {
            0 => break,
            n => {
                start = parse(&buf[0..n], &mut count).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
                let (head, tail) = buf[..].split_at_mut(start);
                head.copy_from_slice(&tail[LEN-start..LEN])
            },
        };
    }
    Ok(())
}

fn main() {
    let mut args = env::args();
    args.next();
    let name = args.next().unwrap();
    let f = File::open(&name).expect(&format!("file not found: {}", &name));
    run(f).expect(&format!("error reading {}", &name));
}

