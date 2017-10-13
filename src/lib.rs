//! A Rust parser for the [Audio Data Transport Stream](https://wiki.multimedia.cx/index.php/ADTS)
//! framing format often used to carry encoded AAC audio data.
//!
//! [`AdtsHeader`](struct.AdtsHeader.html) is the primary type provided by this crate.
//!
//! Given a buffer containing some number of ADTS frames, the first frame may be inspected by
//! constructing a header instance with,
//!
//! ```rust
//! use adts_reader::AdtsHeader;
//! # let buf: Vec<u8> = vec!(0xff, 0xf0, 0, 0, 0, 0, 0, 0, 0);
//! // let buf = ...;
//! match AdtsHeader::from_bytes(&buf) {
//!     Ok(header) => println!("length (headers+payload) is {}", header.frame_length()),
//!     Err(e) => panic!("failed to read header: {:?}", e),
//! }
//! ```
//!
//! # TODO
//!
//!  - CRC handling
//!  - Iterator over frames within a given buffer

#[cfg(test)]
extern crate bitstream_io;

#[derive(Debug)]
pub enum AdtsHeaderError {
    /// Indicates that the given buffer did not start with the required sequence of 12 '1'-bits
    /// (`0xfff`).
    BadSyncWord(u16),
    NotEnoughData {
        expected: usize,
        actual: usize,
    }
}

/// Error indicating that not enough data was provided to `AdtsHeader` to be able to extract the
/// whole ADTS payload following the header fields.
#[derive(Debug,PartialEq)]
pub struct PayloadError {
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug,PartialEq)]
pub enum MpegVersion {
    Mpeg2,
    Mpeg4,
}

#[derive(Debug,PartialEq)]
pub enum AudioObjectType {
    AacMain,
    AacLC,
    AacSSR,
    AacLTP,
}

#[derive(Debug,PartialEq)]
pub enum ProtectionIndicator {
    CrcPresent,
    CrcAbsent,
}

#[derive(Debug,PartialEq)]
pub enum SamplingFrequency {
    Freq96000 = 0x0,
    Freq88200 = 0x1,
    Freq64000 = 0x2,
    Freq48000 = 0x3,
    Freq44100 = 0x4,
    Freq32000 = 0x5,
    Freq24000 = 0x6,
    Freq22050 = 0x7,
    Freq16000 = 0x8,
    Freq12000 = 0x9,
    Freq11025 = 0xa,
    Freq8000  = 0xb,
    FreqReserved0xc = 0xc,
    FreqReserved0xd = 0xd,
    FreqReserved0xe = 0xe,
    FreqReserved0xf = 0xf,
}

impl SamplingFrequency {
    fn from(value: u8) -> SamplingFrequency {
        match value {
            0x0 => SamplingFrequency::Freq96000,
            0x1 => SamplingFrequency::Freq88200,
            0x2 => SamplingFrequency::Freq64000,
            0x3 => SamplingFrequency::Freq48000,
            0x4 => SamplingFrequency::Freq44100,
            0x5 => SamplingFrequency::Freq32000,
            0x6 => SamplingFrequency::Freq24000,
            0x7 => SamplingFrequency::Freq22050,
            0x8 => SamplingFrequency::Freq16000,
            0x9 => SamplingFrequency::Freq12000,
            0xa => SamplingFrequency::Freq11025,
            0xb => SamplingFrequency::Freq8000,
            0xc => SamplingFrequency::FreqReserved0xc,
            0xd => SamplingFrequency::FreqReserved0xd,
            0xe => SamplingFrequency::FreqReserved0xe,
            0xf => SamplingFrequency::FreqReserved0xf,
            _ => panic!("invalud value {:x}", value),
        }
    }

    pub fn freq(&self) -> Option<u32> {
        match self {
            &SamplingFrequency::Freq96000 => Some(96000),
            &SamplingFrequency::Freq88200 => Some(88200),
            &SamplingFrequency::Freq64000 => Some(64000),
            &SamplingFrequency::Freq48000 => Some(48000),
            &SamplingFrequency::Freq44100 => Some(44100),
            &SamplingFrequency::Freq32000 => Some(32000),
            &SamplingFrequency::Freq24000 => Some(24000),
            &SamplingFrequency::Freq22050 => Some(22050),
            &SamplingFrequency::Freq16000 => Some(16000),
            &SamplingFrequency::Freq12000 => Some(12000),
            &SamplingFrequency::Freq11025 => Some(11025),
            &SamplingFrequency::Freq8000  => Some(8000),
            &SamplingFrequency::FreqReserved0xc => None,
            &SamplingFrequency::FreqReserved0xd => None,
            &SamplingFrequency::FreqReserved0xe => None,
            &SamplingFrequency::FreqReserved0xf => None,
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum ChannelConfiguration {
    ObjectTypeSpecificConfig = 0x0,
    Mono = 0x1,
    Stereo = 0x2,
    Three = 0x3,
    Four = 0x4,
    Five = 0x5,
    FiveOne = 0x6,
    SevenOne = 0x7,
}
impl ChannelConfiguration {
    fn from(value: u8) -> ChannelConfiguration {
        match value {
            0x0 => ChannelConfiguration::ObjectTypeSpecificConfig,
            0x1 => ChannelConfiguration::Mono,
            0x2 => ChannelConfiguration::Stereo,
            0x3 => ChannelConfiguration::Three,
            0x4 => ChannelConfiguration::Four,
            0x5 => ChannelConfiguration::Five,
            0x6 => ChannelConfiguration::FiveOne,
            0x7 => ChannelConfiguration::SevenOne,
            _ => panic!("invalid value {}", value),
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum Originality {
    Original,
    Copy,
}

#[derive(Debug,PartialEq)]
pub enum CopyrightIdentificationStart {
    Start,
    Other,
}

pub struct AdtsHeader<'buf> {
    buf: &'buf[u8],
}
impl<'buf> AdtsHeader<'buf> {
    /// Note that this function returns `Err` if there is not enough data to parse the whole
    /// header, but it can return `Ok` even if there is not enough data in the given buffer to hold
    /// the whole of the payload that the header indicates should be present (however _if_ there is
    /// not enough data to hold the payload, then [`payload()`](#method.payload) will return
    /// `None`).
    pub fn from_bytes(buf: &'buf[u8]) -> Result<AdtsHeader, AdtsHeaderError> {
        let header_len = 7;
        Self::check_len(header_len, buf.len())?;
        let header = AdtsHeader {
            buf,
        };
        if header.sync_word() != 0xfff {
            return Err(AdtsHeaderError::BadSyncWord(header.sync_word()));
        }
        let crc_len = 2;
        if header.protection() == ProtectionIndicator::CrcPresent {
            Self::check_len(header_len+crc_len, buf.len())?;
        }
        Ok(header)
    }

    fn check_len(expected: usize, actual: usize) -> Result<(), AdtsHeaderError> {
        if actual < expected {
            Err(AdtsHeaderError::NotEnoughData{
                expected,
                actual,
            })
        } else {
            Ok(())
        }
    }

    fn header_length(&self) -> u16 {
        let fixed_len = 7;
        if self.protection() == ProtectionIndicator::CrcPresent {
            fixed_len + 2
        } else {
            fixed_len
        }
    }

    fn sync_word(&self) -> u16 {
        u16::from(self.buf[0]) << 4 | u16::from(self.buf[1] >> 4)
    }

    pub fn mpeg_version(&self) -> MpegVersion {
        if self.buf[1] & 0b0000_1000 != 0 {
            MpegVersion::Mpeg2
        } else {
            MpegVersion::Mpeg4
        }
    }

    pub fn protection(&self) -> ProtectionIndicator {
        if self.buf[1] & 0b0000_0001 != 0 {
            ProtectionIndicator::CrcAbsent
        } else {
            ProtectionIndicator::CrcPresent
        }
    }

    // Indicates what type of AAC data this stream contains
    pub fn audio_object_type(&self) -> AudioObjectType {
        match self.buf[2] & 0b1100_0000 {
            0b0000_0000 => AudioObjectType::AacMain,
            0b0100_0000 => AudioObjectType::AacLC,
            0b1000_0000 => AudioObjectType::AacSSR,
            0b1100_0000 => AudioObjectType::AacLTP,
            v => panic!("impossible value {:#b}", v),
        }
    }

    pub fn sampling_frequency(&self) -> SamplingFrequency {
        SamplingFrequency::from(self.buf[2] >> 2 & 0b1111)
    }

    /// either 1 or 0
    pub fn private_bit(&self) -> u8 {
        self.buf[2] & 1
    }

    pub fn channel_configuration(&self) -> ChannelConfiguration {
        ChannelConfiguration::from(self.buf[2] << 2 & 0b0100 | self.buf[3] >> 6)
    }

    pub fn originality(&self) -> Originality {
        if self.buf[3] & 0b0010_0000 != 0 {
            Originality::Copy
        } else {
            Originality::Original
        }
    }

    /// either 1 or 0
    pub fn home(&self) -> u8 {
        self.buf[3] >> 4 & 1
    }

    /// either 1 or 0
    pub fn copyright_identification_bit(&self) -> u8 {
        self.buf[3] >> 3 & 1
    }

    pub fn copyright_identification_start(&self) -> CopyrightIdentificationStart {
        if self.buf[3] & 0b0000_0100 != 0 {
            CopyrightIdentificationStart::Start
        } else {
            CopyrightIdentificationStart::Other
        }
    }

    /// length of this frame, including the length of the header.
    pub fn frame_length(&self) -> u16 {
        u16::from(self.buf[3] & 0b11) << 11 |
        u16::from(self.buf[4]       ) << 3  |
        u16::from(self.buf[5]       ) >> 5 
    }

    /// Calculates the length of the frame payload from the `frame_length` header value, and the
    /// total size of headers in this frame -- returning `None` if the `frame_length` header had a
    /// value too small to even include the headers
    pub fn payload_length(&self) -> Option<u16> {
        let diff = self.frame_length() as i16 - self.header_length() as i16;
        if diff >= 0 {
            Some(diff as u16)
        } else {
            None
        }
    }

    pub fn adts_buffer_fullness(&self) -> u16 {
        u16::from(self.buf[5] & 0b00000011) << 6 |
        u16::from(self.buf[6]) >> 2
    }

    /// The number of data blocks in the frame, a value between 1 and 4 inclusive.
    ///
    /// (Note that in the serialised ADTS data stores the _number of blocks - 1_.  This method
    /// returns the actual number of blocks by adding one to the serialised value.)
    ///
    /// Most streams store a single block per ADTS frame
    pub fn number_of_raw_data_blocks_in_frame(&self) -> u8 {
        (self.buf[6] & 0b11) + 1
    }

    /// The payload AAC data inside this ADTS frame
    pub fn payload(&self) -> Result<&'buf[u8], PayloadError> {
        let len = self.frame_length() as usize;
        if self.buf.len() < len {
            Err(PayloadError { expected: len, actual: self.buf.len() })
        } else {
            Ok(&self.buf[self.header_length() as usize..len])
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use bitstream_io::{BE, BitWriter};
    use AdtsHeader;
    use MpegVersion;
    use AudioObjectType;
    use ProtectionIndicator;
    use SamplingFrequency;
    use ChannelConfiguration;
    use *;

    fn make_test_data<F>(builder: F) -> Vec<u8>
    where
        F: Fn(BitWriter<BE>)->Result<(), io::Error>
    {
        let mut data: Vec<u8> = Vec::new();
        builder(BitWriter::<BE>::new(&mut data)).unwrap();
        data
    }


    #[test]
    fn no_crc() {
        let header_data = make_test_data(|mut w| {
            w.write(12, 0xfff)?;// sync_word
            w.write(1, 0)?;     // mpeg_version
            w.write(2, 0)?;     // layer
            w.write(1, 1)?;     // protection_absent
            w.write(2, 0)?;     // object_type
            w.write(4, 0b0011)?;// sampling_frequency_index
            w.write(1, 1)?;     // private_bit
            w.write(3, 2)?;     // channel_configuration
            w.write(1, 1)?;     // original_copy
            w.write(1, 0)?;     // home
            w.write(1, 0)?;     // copyright_identification_bit
            w.write(1, 1)?;     // copyright_identification_start
            w.write(13, 8)?;    // frame_length
            w.write(11, 123)?;  // adts_buffer_fullness
            w.write(2, 0)?;     // number_of_raw_data_blocks_in_frame
            w.write(8, 0b10000001)  // 1 byte of payload data
        });
        let header = AdtsHeader::from_bytes(&header_data[..]).unwrap();
        assert_eq!(header.mpeg_version(), MpegVersion::Mpeg4);
        assert_eq!(header.protection(), ProtectionIndicator::CrcAbsent);
        assert_eq!(header.audio_object_type(), AudioObjectType::AacMain);
        assert_eq!(header.sampling_frequency(), SamplingFrequency::Freq48000);
        assert_eq!(header.sampling_frequency().freq(), Some(48000));
        assert_eq!(header.channel_configuration(), ChannelConfiguration::Stereo);
        assert_eq!(header.originality(), Originality::Copy);
        assert_eq!(header.home(), 0);
        assert_eq!(header.copyright_identification_bit(), 0);
        assert_eq!(header.copyright_identification_start(), CopyrightIdentificationStart::Start);
        assert_eq!(header.frame_length(), 8);
        assert_eq!(header.payload_length(), Some(8 - 7));
        assert_eq!(header.adts_buffer_fullness(), 123);
        assert_eq!(header.number_of_raw_data_blocks_in_frame(), 1);
        assert_eq!(header.payload(), Ok(&[0b10000001][..]));
    }
}
