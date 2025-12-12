use crate::{encoding::*, RealTimeClock};

#[derive(PartialEq, Debug)]
pub struct ReadRealTimeClockResponse {
    pub rtc: RealTimeClock,
}

impl Encodable for ReadRealTimeClockResponse {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_type(&self.rtc)?;
        Ok(())
    }
}

impl Decodable<Self> for ReadRealTimeClockResponse {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        Ok(Self {
            rtc: decoder.read_type::<RealTimeClock>()?,
        })
    }
}
