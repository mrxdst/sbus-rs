use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};

use tokio::{
    net::UdpSocket,
    sync::{oneshot, Mutex},
    task::{AbortHandle, JoinHandle},
};

use crate::{acknowledge::Acknowledge, command_id::CommandId, commands::*, consts::*, encoding::*, message::*, request::Request, RealTimeClock};

/// Errors returned by the [`SBusUDPClient`].
#[derive(Debug, Clone)]
pub enum SBusError {
    /// Represent an IO error.
    IO(Arc<tokio::io::Error>),
    /// Some arguments provided to the function are out of range.
    /// Commonly the combination of address + length is outside the allowed range.
    /// The request was never sent to the server.
    ArgumentsOutOfRange(String),
    /// Indicates that the response received from the server is not a valid response.
    InvalidResponse(String),
}

impl Display for SBusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SBusError::IO(err) => write!(f, "{err}"),
            SBusError::ArgumentsOutOfRange(err) => write!(f, "Argument out of range: {err}"),
            SBusError::InvalidResponse(err) => write!(f, "Invalid response: {err}"),
        }
    }
}

impl Error for SBusError {}

impl From<tokio::io::Error> for SBusError {
    fn from(value: tokio::io::Error) -> Self {
        Self::IO(value.into())
    }
}

impl From<DecodeError> for SBusError {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::MissingData => Self::InvalidResponse("The server sent invalid data".into()),
            DecodeError::InvalidData(text) => Self::InvalidResponse(text),
        }
    }
}

impl From<EncodeError> for SBusError {
    fn from(value: EncodeError) -> Self {
        match value {
            EncodeError::Overflow => Self::ArgumentsOutOfRange("Error encoding message".into()),
        }
    }
}

type ResponseResult = Result<Message, SBusError>;
type ResponseMap = Arc<Mutex<HashMap<u16, oneshot::Sender<ResponseResult>>>>;

pub struct SBusUDPClient {
    socket: Arc<UdpSocket>,
    sequence_number: AtomicU16,
    response_map: ResponseMap,
    abort_handle: AbortHandle,
}

impl SBusUDPClient {
    pub fn new(socket: UdpSocket) -> (Self, JoinHandle<Result<(), SBusError>>) {
        let socket = Arc::new(socket);
        let response_map = Arc::new(Mutex::new(HashMap::new()));

        let join_handle = tokio::spawn(Self::receive_response(socket.clone(), response_map.clone()));

        let client = Self {
            socket,
            sequence_number: AtomicU16::default(),
            response_map,
            abort_handle: join_handle.abort_handle(),
        };

        (client, join_handle)
    }

    pub async fn read_real_time_clock(&self, station: u8) -> Result<RealTimeClock, SBusError> {
        let res_body = self
            .send_request(station, CommandId::ReadRealTimeClock, vec![], TelegramAttribute::Response)
            .await?;
        let res = ReadRealTimeClockResponse::decode_from_bytes(&res_body)?;
        Ok(res.rtc)
    }

    pub async fn read_display_register(&self, station: u8) -> Result<u32, SBusError> {
        let res_body = self
            .send_request(station, CommandId::ReadDisplayRegister, vec![], TelegramAttribute::Response)
            .await?;
        let res = ReadDisplayRegisterResponse::decode_from_bytes(&res_body)?;
        Ok(res.register)
    }

    pub async fn read_firmware_version(&self, station: u8) -> Result<String, SBusError> {
        let res_body = self
            .send_request(station, CommandId::ReadFirmwareVersion, vec![], TelegramAttribute::Response)
            .await?;
        let res = ReadFirmwareVersionResponse::decode_from_bytes(&res_body)?;
        Ok(res.version.into())
    }

    pub async fn read_sbus_station_number(&self) -> Result<u8, SBusError> {
        let res_body = self
            .send_request(254, CommandId::ReadSBusStationNumber, vec![], TelegramAttribute::Response)
            .await?;
        let res = ReadSBusStationNumberResponse::decode_from_bytes(&res_body)?;
        Ok(res.station)
    }

    pub async fn read_counters(&self, station: u8, address: u16, length: u8) -> Result<Vec<i32>, SBusError> {
        validate_input(address, length as usize, COUNTERS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::ReadCounters,
                ReadCountersRequest { address, length }.encode_to_bytes()?,
                TelegramAttribute::Response,
            )
            .await?;
        let res = ReadCountersResponse::decode_from_bytes(&res_body)?;
        Ok(res.values.into())
    }

    pub async fn read_flags(&self, station: u8, address: u16, length: u8) -> Result<Vec<bool>, SBusError> {
        validate_input(address, length as usize, FLAGS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::ReadFlags,
                ReadFlagsRequest { address, length }.encode_to_bytes()?,
                TelegramAttribute::Response,
            )
            .await?;
        let res = ReadFlagsResponse::decode_from_bytes(&res_body)?;
        Ok(res.values.into())
    }

    pub async fn read_inputs(&self, station: u8, address: u16, length: u8) -> Result<Vec<bool>, SBusError> {
        validate_input(address, length as usize, INPUTS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::ReadInputs,
                ReadInputsRequest { address, length }.encode_to_bytes()?,
                TelegramAttribute::Response,
            )
            .await?;
        let res = ReadInputsResponse::decode_from_bytes(&res_body)?;
        Ok(res.values.into())
    }

    pub async fn read_outputs(&self, station: u8, address: u16, length: u8) -> Result<Vec<bool>, SBusError> {
        validate_input(address, length as usize, OUTPUTS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::ReadOutputs,
                ReadOutputsRequest { address, length }.encode_to_bytes()?,
                TelegramAttribute::Response,
            )
            .await?;
        let res = ReadOutputsResponse::decode_from_bytes(&res_body)?;
        Ok(res.values.into())
    }

    pub async fn read_registers(&self, station: u8, address: u16, length: u8) -> Result<Vec<i32>, SBusError> {
        validate_input(address, length as usize, REGISTERS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::ReadRegisters,
                ReadRegistersRequest { address, length }.encode_to_bytes()?,
                TelegramAttribute::Response,
            )
            .await?;
        let res = ReadRegistersResponse::decode_from_bytes(&res_body)?;
        Ok(res.values.into())
    }

    pub async fn read_timers(&self, station: u8, address: u16, length: u8) -> Result<Vec<i32>, SBusError> {
        validate_input(address, length as usize, TIMERS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::ReadTimers,
                ReadTimersRequest { address, length }.encode_to_bytes()?,
                TelegramAttribute::Response,
            )
            .await?;
        let res = ReadTimersResponse::decode_from_bytes(&res_body)?;
        Ok(res.values.into())
    }

    pub async fn write_real_time_clock(&self, station: u8, rtc: RealTimeClock) -> Result<bool, SBusError> {
        let res_body = self
            .send_request(
                station,
                CommandId::WriteRealTimeClock,
                WriteRealTimeClockRequest { rtc }.encode_to_bytes()?,
                TelegramAttribute::Acknowledge,
            )
            .await?;
        let res = Acknowledge::decode_from_bytes(&res_body)?;
        Ok(res == Acknowledge::Ack)
    }

    pub async fn write_counters(&self, station: u8, address: u16, values: &[i32]) -> Result<bool, SBusError> {
        validate_input(address, values.len(), COUNTERS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::WriteCounters,
                WriteCountersRequest {
                    address,
                    values: values.into(),
                }
                .encode_to_bytes()?,
                TelegramAttribute::Acknowledge,
            )
            .await?;
        let res = Acknowledge::decode_from_bytes(&res_body)?;
        Ok(res == Acknowledge::Ack)
    }

    pub async fn write_flags(&self, station: u8, address: u16, values: &[bool]) -> Result<bool, SBusError> {
        validate_input(address, values.len(), FLAGS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::WriteFlags,
                WriteFlagsRequest {
                    address,
                    values: values.into(),
                }
                .encode_to_bytes()?,
                TelegramAttribute::Acknowledge,
            )
            .await?;
        let res = Acknowledge::decode_from_bytes(&res_body)?;
        Ok(res == Acknowledge::Ack)
    }

    pub async fn write_outputs(&self, station: u8, address: u16, values: &[bool]) -> Result<bool, SBusError> {
        validate_input(address, values.len(), OUTPUTS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::WriteOutputs,
                WriteOutputsRequest {
                    address,
                    values: values.into(),
                }
                .encode_to_bytes()?,
                TelegramAttribute::Acknowledge,
            )
            .await?;
        let res = Acknowledge::decode_from_bytes(&res_body)?;
        Ok(res == Acknowledge::Ack)
    }

    pub async fn write_registers(&self, station: u8, address: u16, values: &[i32]) -> Result<bool, SBusError> {
        validate_input(address, values.len(), REGISTERS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::WriteRegisters,
                WriteRegistersRequest {
                    address,
                    values: values.into(),
                }
                .encode_to_bytes()?,
                TelegramAttribute::Acknowledge,
            )
            .await?;
        let res = Acknowledge::decode_from_bytes(&res_body)?;
        Ok(res == Acknowledge::Ack)
    }

    pub async fn write_timers(&self, station: u8, address: u16, values: &[i32]) -> Result<bool, SBusError> {
        validate_input(address, values.len(), TIMERS_MAX_REQUEST_LEN)?;
        let res_body = self
            .send_request(
                station,
                CommandId::WriteTimers,
                WriteTimersRequest {
                    address,
                    values: values.into(),
                }
                .encode_to_bytes()?,
                TelegramAttribute::Acknowledge,
            )
            .await?;
        let res = Acknowledge::decode_from_bytes(&res_body)?;
        Ok(res == Acknowledge::Ack)
    }

    async fn send_request(&self, station: u8, command_id: CommandId, body: Vec<u8>, response_type: TelegramAttribute) -> Result<Vec<u8>, SBusError> {
        let sequence_number = self.sequence_number.fetch_add(1, Ordering::Relaxed);

        let req = Request {
            station,
            command_id,
            body: body.into(),
        };

        let req_msg = Message {
            sequence_number,
            telegram_attribute: TelegramAttribute::Request,
            body: req.encode_to_bytes()?,
        };

        let req_bytes = req_msg.encode_to_bytes()?;

        let (sender, receiver) = oneshot::channel::<ResponseResult>();

        {
            let mut map = self.response_map.lock().await;
            map.insert(sequence_number, sender);
        }

        self.socket.send(&req_bytes).await?;

        let res_msg = match receiver.await.unwrap() {
            Ok(msg) => msg,
            Err(error) => return Err(error),
        };

        if res_msg.telegram_attribute != response_type {
            return Err(SBusError::InvalidResponse(format!("Telegram attribute mismatch")));
        }

        Ok(res_msg.body)
    }

    async fn receive_response(socket: Arc<UdpSocket>, response_map: ResponseMap) -> Result<(), SBusError> {
        let mut read_buffer = [0; 256];
        loop {
            async fn recv(socket: &UdpSocket, read_buffer: &mut [u8]) -> Result<Message, SBusError> {
                let byte_length = socket.recv(read_buffer).await?;
                Ok(Message::decode_from_bytes(&read_buffer[0..byte_length])?)
            }

            let msg = match recv(&socket, &mut read_buffer).await {
                Ok(msg) => msg,
                Err(error) => {
                    let mut response_map = response_map.lock().await;
                    for (_, sender) in response_map.drain() {
                        _ = sender.send(Err(error.clone()));
                    }
                    return Err(error);
                }
            };

            let sender = response_map.lock().await.remove(&msg.sequence_number);
            match sender {
                None => return Err(SBusError::InvalidResponse("The server sent an unexpected response".into())),
                Some(sender) => _ = sender.send(Ok(msg)),
            }
        }
    }
}

impl Drop for SBusUDPClient {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

fn validate_input(address: u16, length: usize, max_length: u16) -> Result<(), SBusError> {
    if length == 0 || length > max_length as usize {
        return Err(SBusError::ArgumentsOutOfRange(format!(
            "Length exceeds maximum allowed length {max_length}"
        )));
    }
    u16::checked_add(address, (length - 1) as u16).ok_or(SBusError::ArgumentsOutOfRange(format!("Address + length exceeds device address space")))?;
    Ok(())
}
