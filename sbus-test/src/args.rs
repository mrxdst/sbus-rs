use std::{fmt::Display, num::ParseIntError, path::PathBuf, time::Duration};

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Hostname or ip address
    pub host: String,

    /// UDP port number
    #[arg(default_value = "5050")]
    pub port: u16,

    /// Network timeout in ms
    #[arg(short, long, default_value = "500", value_parser = parse_duration)]
    pub timeout: Duration,
}

#[derive(Parser, Debug)]
#[command()]
pub struct Interactive {
    #[command(subcommand)]
    pub command: InteractiveCommands,
}

#[derive(Subcommand, Debug)]
pub enum InteractiveCommands {
    /// Read info from device
    Info,

    /// Read values from device
    Read(ReadArgs),

    /// Write values to device
    Write(WriteArgs),

    /// Export the previously printed table
    Export(ExportArgs),

    /// Scan for devices
    Scan(ScanArgs),

    /// Read the station nr
    Station,

    /// Set configuration
    Set(SetArgs),

    /// Exit the program
    Exit,
}

impl Display for InteractiveCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractiveCommands::Info => write!(f, "Info"),
            InteractiveCommands::Read(_) => write!(f, "Read"),
            InteractiveCommands::Write(_) => write!(f, "Write"),
            InteractiveCommands::Export(_) => write!(f, "Export"),
            InteractiveCommands::Scan(_) => write!(f, "Scan"),
            InteractiveCommands::Station => write!(f, "Station"),
            InteractiveCommands::Set(_) => write!(f, "Set"),
            InteractiveCommands::Exit => write!(f, "Exit"),
        }
    }
}

#[derive(Args, Debug)]
pub struct ReadArgs {
    /// Type to read
    #[arg(value_enum)]
    pub kind: ReadKind,

    /// Address to start reading from
    pub address: u16,

    /// Number of addresses to read
    #[arg(default_value = "1")]
    pub length: u8,
}

#[derive(Debug, PartialEq, Clone, Copy, ValueEnum)]
#[value()]
pub enum ReadKind {
    Counters,
    Flags,
    Inputs,
    Outputs,
    Registers,
    Timers,
}

#[derive(Args, Debug)]
#[command(allow_negative_numbers = true)]
pub struct WriteArgs {
    /// Type to write
    #[arg(value_enum)]
    pub kind: WriteKind,

    /// Address to start writing to
    pub address: u16,

    /// Values to write
    #[arg(required = true)]
    pub values: Vec<String>,

    /// Datatype of the values
    #[arg(long = "type", value_enum, default_value = "Integer")]
    pub datatype: WriteDatatype,
}

#[derive(Debug, PartialEq, Clone, Copy, ValueEnum)]
#[value()]
pub enum WriteKind {
    Counters,
    Flags,
    Outputs,
    Registers,
    Timers,
}

#[derive(Debug, PartialEq, Clone, Copy, ValueEnum)]
#[value(rename_all = "PascalCase")]
pub enum WriteDatatype {
    Integer,
    Float,
    Hex,
    Bin,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// The file to write to
    pub filename: PathBuf,
}

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Minimum station
    #[arg(default_value = "0")]
    pub min: u8,

    /// Maximum station
    #[arg(default_value = "255")]
    pub max: u8,
}

#[derive(Args, Debug)]
pub struct SetArgs {
    #[command(subcommand)]
    pub command: SetCommands,
}

#[derive(Subcommand, Debug)]
pub enum SetCommands {
    /// Set the station
    Station { station: u8 },

    /// Set timeout
    Timeout {
        #[arg(value_parser = parse_duration)]
        timeout: Duration,
    },

    /// Set address offset
    #[command(allow_negative_numbers = true)]
    Offset { offset: i32 },
}

fn parse_duration(input: &str) -> Result<Duration, ParseIntError> {
    let ms = input.parse()?;
    Ok(Duration::from_millis(ms))
}
