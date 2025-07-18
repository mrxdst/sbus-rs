use num_enum::{FromPrimitive, IntoPrimitive};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, FromPrimitive)]
pub enum CommandId {
    ReadCounters = 0x00,
    ReadDisplayRegister = 0x01,
    ReadFlags = 0x02,
    ReadInputs = 0x03,
    ReadRealTimeClock = 0x04,
    ReadOutputs = 0x05,
    ReadRegisters = 0x06,
    ReadTimers = 0x07,
    WriteCounters = 0x0A,
    WriteFlags = 0x0B,
    WriteRealTimeClock = 0x0C,
    WriteOutputs = 0x0D,
    WriteRegisters = 0x0E,
    WriteTimers = 0x0F,
    // ReadWriteMultimedias = 0x13,
    // ReadPCDStatusCPU0 = 0x14,
    // ReadPCDStatusCPU1 = 0x15,
    // ReadPCDStatusCPU2 = 0x16,
    // ReadPCDStatusCPU3 = 0x17,
    // ReadPCDStatusCPU4 = 0x18,
    // ReadPCDStatusCPU5 = 0x19,
    // ReadPCDStatusCPU6 = 0x1A,
    // ReadPCDStatusOwn = 0x1B,
    ReadSBusStationNumber = 0x1D,
    // ReadUserMemory = 0x1E,
    // ReadProgramLine = 0x1F,
    ReadFirmwareVersion = 0x20,
    // ReadText = 0x21,
    // ReadActiveTransition = 0x22,
    // WriteUserMemory = 0x23,
    // WriteProgramLine = 0x24,
    // WriteText = 0x25,
    // RunProcedureCPU0 = 0x28,
    // RunProcedureCPU1 = 0x29,
    // RunProcedureCPU2 = 0x2A,
    // RunProcedureCPU3 = 0x2B,
    // RunProcedureCPU4 = 0x2C,
    // RunProcedureCPU5 = 0x2D,
    // RunProcedureCPU6 = 0x2E,
    // RunProcedureOwnCPU = 0x2F,
    // RunProcedureAllCPUS = 0x30,
    // RestartColdCPU1 = 0x32,
    // RestartColdCPU2 = 0x33,
    // RestartColdCPU3 = 0x34,
    // RestartColdCPU4 = 0x35,
    // RestartColdCPU5 = 0x36,
    // RestartColdCPU6 = 0x37,
    // RestartColdOwnCPU = 0x38,
    // RestartColdAllCPUS = 0x39,
    // StopProcedureCPU0 = 0x3C,
    // StopProcedureCPU1 = 0x3D,
    // StopProcedureCPU2 = 0x3E,
    // StopProcedureCPU3 = 0x3F,
    // StopProcedureCPU4 = 0x40,
    // StopProcedureCPU5 = 0x41,
    // StopProcedureCPU6 = 0x42,
    // StopProcedureOwnCPU = 0x43,
    // StopProcedureAllCPUS = 0x44,
    // ReadArithmeticStatusAndACCU = 0x46,
    // ReadByte = 0x47,
    // ReadHaltFailureRegister = 0x48,
    // ReadIndexRegister = 0x49,
    // ReadInstructionPointer = 0x4A,
    // FindHistory = 0x4B,
    // WriteArithmeticStatusAndACCU = 0x50,
    // WriteByte = 0x51,
    // WriteIndexRegister = 0x52,
    // WriteInstructionPointer = 0x53,
    // ClearAllFORT = 0x5A,
    // ClearFlags = 0x5B,
    // ClearOutputs = 0x5C,
    // ClearRegisters = 0x5D,
    // ClearTimers = 0x5E,
    // RestartWarmCPU1 = 0x64,
    // RestartWarmCPU2 = 0x65,
    // RestartWarmCPU3 = 0x66,
    // RestartWarmCPU4 = 0x67,
    // RestartWarmCPU5 = 0x68,
    // RestartWarmCPU6 = 0x69,
    // RestartWarmOwnCPU = 0x6A,
    // RestartWarmAllCPUS = 0x6B,
    // ChangeBlock = 0x6E,
    // ClearHistoryFailure = 0x6F,
    // DeleteProgramLine = 0x70,
    // GoConditional = 0x71,
    // InsertProgramLine = 0x72,
    // LocalCycles = 0x73,
    // AllCycles = 0x74,
    // MakeText = 0x75,
    // ExecuteSingleInstruction = 0x76,
    // SingleStep = 0x77,
    // XOB17Interrupt = 0x82,
    // XOB18Interrupt = 0x83,
    // XOB19Interrupt = 0x84,
    // ReadHangupTimeout = 0x91,
    // ReadDataBlock = 0x96,
    // WriteDataBlock = 0x97,
    // MakeDataBlock = 0x98,
    // ClearDataBlock = 0x99,
    // ClearText = 0x9A,
    // ReadBlockAddress = 0x9B,
    // ReadBlockSizes = 0x9C,
    // ReadCurrentBlock = 0x9D,
    // ReadCallStack = 0x9E,
    // ReadDBX = 0x9F,
    // ReadUserEEPROMRegister = 0xA1,
    // WriteUserEEPROMRegister = 0xA3,
    // EraseFlash = 0xA5,
    // RestartColdFlag = 0xA6,
    // WriteSystemBuffer = 0xA7,
    // ReadSystemBuffer = 0xA8,
    // ReadWriteBlockData = 0xA9,
    // GetDiagnostic = 0xAA,
    // ReadSystemInformation = 0xAB,
    // ChangesBlocksOnRun = 0xAC,
    // FlashcardTelegram = 0xAD,
    // DownloadFW = 0xAE,
    // WebServerSerialCommunication = 0xAF,
    #[num_enum(catch_all)]
    Unknown(u8),
}
