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
//! # let buf: Vec<u8> = vec!(0xff, 0xf0, 0, 0, 1, 0x20, 0, 0, 0);
//! // let buf = ...;
//! match AdtsHeader::from_bytes(&buf) {
//!     Ok(header) => println!("length (headers+payload) is {}", header.frame_length()),
//!     Err(e) => panic!("failed to read header: {:?}", e),
//! }
//! ```
//!
//! # Unsupported
//!
//!  - Resynchronising `AdtsParser` after encountering bitstream error (we could search for
//!    sync-word)
//!  - Copyright identifiers (I don't have any example bitstreams to try)
//!  - CRC handling (probably needs to be implemented as part of AAC bitstream parsing)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, future_incompatible)]

// TODO: might be better to implement AdtsParser as an iterator, rather then doing callbacks into a
// trait implementation -- it looked hard to implement though!

use std::fmt;

#[derive(Debug)]
pub enum AdtsHeaderError {
    /// Indicates that the given buffer did not start with the required sequence of 12 '1'-bits
    /// (`0xfff`).
    BadSyncWord(u16),
    NotEnoughData {
        expected: usize,
        actual: usize,
    },
    /// The frame_length field stored in the ADTS header is invalid as it holds a value smaller
    /// than the size of the header fields
    BadFrameLength {
        minimum: usize,
        actual: usize,
    },
}

/// Error indicating that not enough data was provided to `AdtsHeader` to be able to extract the
/// whole ADTS payload following the header fields.
#[derive(Debug, PartialEq)]
pub struct PayloadError {
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug, PartialEq)]
pub enum MpegVersion {
    Mpeg2,
    Mpeg4,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AudioObjectType {
    /// 'Main' profile
    AacMain,
    /// 'Low Complexity' profile
    AacLC,
    /// 'Scalable Sample Rate' profile
    AacSSR,
    /// 'Long Term Prediction' profile
    AacLTP,
}

#[derive(Debug, PartialEq)]
pub enum ProtectionIndicator {
    CrcPresent,
    CrcAbsent,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SamplingFrequency {
    /// 96kHz
    Freq96000 = 0x0,
    /// 88.2kHz
    Freq88200 = 0x1,
    /// 64kHz
    Freq64000 = 0x2,
    /// 48kHz
    Freq48000 = 0x3,
    /// 44.1kHz
    Freq44100 = 0x4,
    /// 32kHz
    Freq32000 = 0x5,
    /// 24kHz
    Freq24000 = 0x6,
    /// 22.05kHz
    Freq22050 = 0x7,
    /// 16kHz
    Freq16000 = 0x8,
    /// 12kHz
    Freq12000 = 0x9,
    /// 11.025kHz
    Freq11025 = 0xa,
    /// 8kHz
    Freq8000 = 0xb,
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
            &SamplingFrequency::Freq8000 => Some(8000),
            &SamplingFrequency::FreqReserved0xc => None,
            &SamplingFrequency::FreqReserved0xd => None,
            &SamplingFrequency::FreqReserved0xe => None,
            &SamplingFrequency::FreqReserved0xf => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq)]
pub enum Originality {
    Original,
    Copy,
}

#[derive(Debug, PartialEq)]
pub enum CopyrightIdentificationStart {
    Start,
    Other,
}

/// Extract information for a single ADTS frame from the start of the given byte buffer .
pub struct AdtsHeader<'buf> {
    buf: &'buf [u8],
}
impl<'buf> AdtsHeader<'buf> {
    /// Construct an instance by borrowing the given byte buffer.  The given buffer may be longer
    /// then the ADTS frame, in which case the rest of the buffer is ignored.
    ///
    ///
    /// Note that this function returns `Err` if there is not enough data to parse the whole
    /// header, but it can return `Ok` even if there is not enough data in the given buffer to hold
    /// the whole of the payload that the header indicates should be present (however _if_ there is
    /// not enough data to hold the payload, then [`payload()`](#method.payload) will return
    /// `None`).
    pub fn from_bytes(buf: &'buf [u8]) -> Result<AdtsHeader<'_>, AdtsHeaderError> {
        assert!(!buf.is_empty());
        let header_len = 7;
        Self::check_len(header_len, buf.len())?;
        let header = AdtsHeader { buf };
        if header.sync_word() != 0xfff {
            return Err(AdtsHeaderError::BadSyncWord(header.sync_word()));
        }
        let crc_len = 2;
        if header.protection() == ProtectionIndicator::CrcPresent {
            Self::check_len(header_len + crc_len, buf.len())?;
        }
        if header.frame_length() < header.header_length() {
            return Err(AdtsHeaderError::BadFrameLength {
                actual: header.frame_length() as usize,
                minimum: header.header_length() as usize,
            });
        }
        Ok(header)
    }

    fn check_len(expected: usize, actual: usize) -> Result<(), AdtsHeaderError> {
        if actual < expected {
            Err(AdtsHeaderError::NotEnoughData { expected, actual })
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
        u16::from(self.buf[3] & 0b11) << 11
            | u16::from(self.buf[4]) << 3
            | u16::from(self.buf[5]) >> 5
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
        u16::from(self.buf[5] & 0b00000011) << 6 | u16::from(self.buf[6]) >> 2
    }

    /// Gives the 16-bit cyclic redundancy check value stored in this frame header, or `None` if
    /// the header does not supply a CRC.
    ///
    /// NB the implementation doesn't currently check that the CRC is correct
    pub fn crc(&self) -> Option<u16> {
        match self.protection() {
            ProtectionIndicator::CrcAbsent => None,
            ProtectionIndicator::CrcPresent => {
                Some(u16::from(self.buf[7]) << 8 | u16::from(self.buf[8]))
            }
        }
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
    pub fn payload(&self) -> Result<&'buf [u8], PayloadError> {
        let len = self.frame_length() as usize;
        if self.buf.len() < len {
            Err(PayloadError {
                expected: len,
                actual: self.buf.len(),
            })
        } else {
            Ok(&self.buf[self.header_length() as usize..len])
        }
    }
}
impl<'buf> fmt::Debug for AdtsHeader<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("AdtsHeader")
            .field("mpeg_version", &self.mpeg_version())
            .field("protection", &self.protection())
            .field("audio_object_type", &self.audio_object_type())
            .field("sampling_frequency", &self.sampling_frequency())
            .field("private_bit", &self.private_bit())
            .field("channel_configuration", &self.channel_configuration())
            .field("originality", &self.originality())
            .field("home", &self.home())
            .field(
                "copyright_identification_bit",
                &self.copyright_identification_bit(),
            )
            .field(
                "copyright_identification_start",
                &self.copyright_identification_start(),
            )
            .field("frame_length", &self.frame_length())
            .field("adts_buffer_fullness", &self.adts_buffer_fullness())
            .field("crc", &self.crc())
            .field(
                "number_of_raw_data_blocks_in_frame",
                &self.number_of_raw_data_blocks_in_frame(),
            )
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum CopyrightIdErr {
    TooFewBits,
    TooManyBits,
}

#[derive(Debug, PartialEq)]
pub struct CopyrightIdentification {
    pub copyright_identifier: u8,
    pub copyright_number: u64,
}

#[derive(PartialEq)]
enum AdtsState {
    Start,
    Incomplete,
    Error,
}

#[derive(Debug, PartialEq)]
pub enum AdtsParseError {
    BadSyncWord,
    BadFrameLength,
}

/// Trait to be implemented by types that wish to consume the ADTS data produced by [`AdtsParser`](struct.AdtsParser.html).
///
/// # Example
///
/// ```rust
/// use adts_reader::*;
///
/// struct MyConsumer { }
/// impl AdtsConsumer for MyConsumer {
///     fn new_config(&mut self, mpeg_version: MpegVersion, protection: ProtectionIndicator, aot: AudioObjectType, freq: SamplingFrequency, private_bit: u8, channels: ChannelConfiguration, originality: Originality, home: u8) {
///         println!("Configuration {:?} {:?} {:?}", aot, freq, channels);
///     }
///     fn payload(&mut self, buffer_fullness: u16, number_of_blocks: u8, buf: &[u8]) {
///         println!(" - frame of {} bytes", buf.len());
///     }
///     fn error(&mut self, err: AdtsParseError) {
///         println!(" - oops: {:?}", err);
///     }
/// }
///
/// let consumer = MyConsumer { };
/// let parser = AdtsParser::new(consumer);
/// ```
pub trait AdtsConsumer {
    /// Called when a new configuration is found within the ADTS bitstream
    ///
    /// An ADTS bitstream should have the same configuration throughout, so this would usually just
    /// be called once at the beginning of the stream.  The audio configuration header values do
    /// however appear in every frame (so that the bitstream format can support seeking, not that
    /// this implementation helps there) and so it would be possible for a malformed bitstream to
    /// signal a configuration change part way through.
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
    );

    /// called with the ADTS frame payload, and frame-specific header values
    fn payload(&mut self, buffer_fullness: u16, number_of_blocks: u8, buf: &[u8]);

    /// called if AdtsParser encounters an error in the ADTS bitstream.
    fn error(&mut self, err: AdtsParseError);
}

/// Find ADTS frames within provided buffers of data, announcing audio configuration as it is
/// discovered (normally just once at the start, but possibly changing during the stream if the
/// stream is malformed).
///
/// Does not currently try to handle re-synchronise with the ADTS bitstream on encountering bad
/// data.
pub struct AdtsParser<C>
where
    C: AdtsConsumer,
{
    pub consumer: C,
    current_config: [u8; 3],
    state: AdtsState,
    incomplete_frame: Vec<u8>,
    desired_data_len: Option<usize>,
}
impl<C> AdtsParser<C>
where
    C: AdtsConsumer,
{
    pub fn new(consumer: C) -> AdtsParser<C> {
        AdtsParser {
            consumer,
            current_config: [0; 3],
            state: AdtsState::Start,
            incomplete_frame: vec![],
            desired_data_len: None,
        }
    }

    fn is_new_config(&self, header_data: &[u8]) -> bool {
        self.current_config != header_data[0..3]
    }

    fn remember(&mut self, remaining_data: &[u8], desired_data_len: usize) {
        self.state = AdtsState::Incomplete;
        self.incomplete_frame.clear();
        self.incomplete_frame.extend_from_slice(remaining_data);
        self.desired_data_len = Some(desired_data_len);
    }

    /// Initialize or re-initialize parser state.  Call this function before processing a group of
    /// ADTS frames to ensure that any error state due to processing an earlier group of ADTS
    /// frames is cleared.
    pub fn start(&mut self) {
        if self.state == AdtsState::Incomplete {
            self.incomplete_frame.clear();
            self.desired_data_len = None;
            eprintln!("ADTS: incomplete data buffer dropped by call to start()");
        }
        self.state = AdtsState::Start;
    }

    /// Extracts information about each ADTS frame in the given buffer, which is passed to the
    /// `AdtsConsumer` implementation supplied at construction time.
    ///
    /// If the given buffer ends part-way through an ADTS frame, the remaining unconsumed data
    /// will be buffered inside this AdtsParser instance, and the rest of the ADTS frame may be
    /// passed in another buffer in the next call to this method.
    pub fn push(&mut self, adts_buf: &[u8]) {
        let mut buf = adts_buf;
        match self.state {
            AdtsState::Error => return, // TODO: resync to recover from bitstream errors
            AdtsState::Incomplete => {
                // on last call to push(), the end of the adts_buf held the start of an ADTS
                // frame, and we copied that data into incomplete_buffer, so now lets try to add
                // enough initial bytes from the adts_buf given to this call to get a complete
                // frame
                loop {
                    let bytes_needed_to_complete_frame = self.desired_data_len.unwrap() - self.incomplete_frame.len();
                    if buf.len() < bytes_needed_to_complete_frame {
                        self.incomplete_frame.extend_from_slice(buf);
                        return;
                    }
                    self.incomplete_frame
                        .extend_from_slice(&buf[..bytes_needed_to_complete_frame]);
                    buf = &buf[bytes_needed_to_complete_frame..];
                    let mut still_more = false; // TODO: this is horrible
                    match AdtsHeader::from_bytes(&self.incomplete_frame[..]) {
                        Ok(header) => {
                            if (header.frame_length() as usize) > self.incomplete_frame.len() {
                                self.desired_data_len = Some(header.frame_length() as usize);
                                still_more = true;
                            } else {
                                if self.is_new_config(&self.incomplete_frame[..]) {
                                    Self::push_config(
                                        &mut self.current_config,
                                        &mut self.consumer,
                                        &header,
                                        &self.incomplete_frame[..],
                                    );
                                }
                                Self::push_payload(&mut self.consumer, header);
                                self.state = AdtsState::Start;
                            }
                        }
                        Err(e) => {
                            self.state = AdtsState::Error;
                            match e {
                                AdtsHeaderError::BadSyncWord { .. } => {
                                    self.consumer.error(AdtsParseError::BadSyncWord);
                                    return;
                                }
                                AdtsHeaderError::BadFrameLength { .. } => {
                                    self.consumer.error(AdtsParseError::BadFrameLength);
                                    return;
                                }
                                AdtsHeaderError::NotEnoughData { expected, .. } => {
                                    self.desired_data_len = Some(expected);
                                    still_more = true;
                                }
                            }
                        }
                    }
                    if !still_more {
                        break;
                    }
                }
            }
            AdtsState::Start => (),
        };
        let mut pos = 0;
        while pos < buf.len() {
            let remaining_data = &buf[pos..];
            let h = match AdtsHeader::from_bytes(remaining_data) {
                Ok(header) => header,
                Err(e) => {
                    self.state = AdtsState::Error;
                    match e {
                        AdtsHeaderError::BadSyncWord { .. } => {
                            self.consumer.error(AdtsParseError::BadSyncWord)
                        }
                        AdtsHeaderError::BadFrameLength { .. } => {
                            self.consumer.error(AdtsParseError::BadFrameLength);
                            return;
                        }
                        AdtsHeaderError::NotEnoughData { expected, .. } => {
                            self.remember(remaining_data, expected);
                            return;
                        }
                    }
                    return;
                }
            };
            let new_pos = pos + h.frame_length() as usize;
            if new_pos > buf.len() {
                self.remember(remaining_data, h.frame_length() as usize);
                return;
            }
            if self.is_new_config(remaining_data) {
                Self::push_config(
                    &mut self.current_config,
                    &mut self.consumer,
                    &h,
                    remaining_data,
                );
            }
            Self::push_payload(&mut self.consumer, h);
            self.state = AdtsState::Start;
            pos = new_pos;
        }
    }

    fn push_config(
        current_config: &mut [u8; 3],
        consumer: &mut C,
        h: &AdtsHeader<'_>,
        frame_buffer: &[u8],
    ) {
        current_config.copy_from_slice(&frame_buffer[0..3]);
        consumer.new_config(
            h.mpeg_version(),
            h.protection(),
            h.audio_object_type(),
            h.sampling_frequency(),
            h.private_bit(),
            h.channel_configuration(),
            h.originality(),
            h.home(),
        );
    }

    fn push_payload(consumer: &mut C, h: AdtsHeader<'_>) {
        match h.payload() {
            Ok(payload) => {
                consumer.payload(
                    h.adts_buffer_fullness(),
                    h.number_of_raw_data_blocks_in_frame(),
                    payload,
                );
            }
            Err(PayloadError { expected, actual }) => {
                // since we checked we had enough data for the whole frame above, this must be
                // a bug,
                panic!(
                    "Unexpected payload size mismatch: expected {}, actual size {}",
                    expected, actual
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bitstream_io::{BitWriter, BE};
    use std::io;
    use super::*;

    fn make_test_data<F>(builder: F) -> Vec<u8>
    where
        F: Fn(BitWriter<'_, BE>) -> Result<(), io::Error>,
    {
        let mut data: Vec<u8> = Vec::new();
        builder(BitWriter::<BE>::new(&mut data)).unwrap();
        data
    }

    fn write_frame(w: &mut BitWriter<'_, BE>) -> Result<(), io::Error> {
        w.write(12, 0xfff)?; // sync_word
        w.write(1, 0)?; // mpeg_version
        w.write(2, 0)?; // layer
        w.write(1, 1)?; // protection_absent
        w.write(2, 0)?; // object_type
        w.write(4, 0b0011)?; // sampling_frequency_index
        w.write(1, 1)?; // private_bit
        w.write(3, 2)?; // channel_configuration
        w.write(1, 1)?; // original_copy
        w.write(1, 0)?; // home
        w.write(1, 0)?; // copyright_identification_bit
        w.write(1, 1)?; // copyright_identification_start
        w.write(13, 8)?; // frame_length
        w.write(11, 123)?; // adts_buffer_fullness
        w.write(2, 0)?; // number_of_raw_data_blocks_in_frame
        w.write(8, 0b10000001) // 1 byte of payload data
    }

    #[test]
    fn no_crc() {
        let header_data = make_test_data(|mut w| write_frame(&mut w));
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
        assert_eq!(
            header.copyright_identification_start(),
            CopyrightIdentificationStart::Start
        );
        assert_eq!(header.frame_length(), 8);
        assert_eq!(header.payload_length(), Some(8 - 7));
        assert_eq!(header.adts_buffer_fullness(), 123);
        assert_eq!(header.number_of_raw_data_blocks_in_frame(), 1);
        assert_eq!(header.payload(), Ok(&[0b10000001][..]));
    }

    struct MockConsumer {
        seq: usize,
        payload_seq: usize,
        payload_size: Option<usize>,
    }
    impl MockConsumer {
        pub fn new() -> MockConsumer {
            MockConsumer {
                seq: 0,
                payload_seq: 0,
                payload_size: None
            }
        }
        pub fn assert_seq(&mut self, expected: usize) {
            assert_eq!(expected, self.seq);
            self.seq += 1;
        }
    }
    impl AdtsConsumer for MockConsumer {
        // TODO: assertions are terribly brittle
        fn new_config(
            &mut self,
            mpeg_version: MpegVersion,
            _protection: ProtectionIndicator,
            _aot: AudioObjectType,
            _freq: SamplingFrequency,
            _private_bit: u8,
            _channels: ChannelConfiguration,
            _originality: Originality,
            _home: u8,
        ) {
            self.assert_seq(0);
            assert_eq!(mpeg_version, MpegVersion::Mpeg4);
        }
        fn payload(&mut self, _buffer_fullness: u16, _number_of_blocks: u8, buf: &[u8]) {
            self.payload_seq += 1;
            let new_payload_seq = self.payload_seq;
            self.assert_seq(new_payload_seq);
            self.payload_size = Some(buf.len());
        }
        fn error(&mut self, err: AdtsParseError) {
            panic!("no errors expected in bitstream: {:?}", err);
        }
    }

    #[test]
    fn parser() {
        let header_data = make_test_data(|mut w| {
            write_frame(&mut w)?;
            write_frame(&mut w)
        });
        for split in 0..header_data.len() {
            let mut parser = AdtsParser::new(MockConsumer::new());
            let (head, tail) = header_data.split_at(split);
            parser.push(head);
            parser.push(tail);
            assert_eq!(2, parser.consumer.payload_seq);
            assert_eq!(Some(1), parser.consumer.payload_size);
        }
    }

    #[test]
    fn too_short() {
        let header_data = make_test_data(|mut w| write_frame(&mut w));
        let mut parser = AdtsParser::new(MockConsumer::new());
        parser.push(&header_data[..5]);
        parser.push(&header_data[5..7]);
    }
}
