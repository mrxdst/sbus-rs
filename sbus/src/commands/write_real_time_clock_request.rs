use crate::{encoding::*, RealTimeClock};

#[derive(PartialEq, Debug)]
pub struct WriteRealTimeClockRequest {
    pub rtc: RealTimeClock,
}

impl Encodable for WriteRealTimeClockRequest {
    fn encode(&self, encoder: &mut Encoder) -> EncodeResult {
        encoder.write_type(&self.rtc)?;
        return Ok(());
    }
}

impl Decodable<Self> for WriteRealTimeClockRequest {
    fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
        return Ok(Self {
            rtc: decoder.read_type::<RealTimeClock>()?,
        });
    }
}
