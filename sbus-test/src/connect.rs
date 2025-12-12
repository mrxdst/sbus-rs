use std::{error::Error, io::Write, sync::Arc, time::Duration};

use clap::Parser;
use comfy_table::{presets, CellAlignment, ColumnConstraint, Table, Width};
use rustyline::{completion::Completer, history::MemHistory, Editor, Helper, Highlighter, Hinter, Validator};
use sbus::{ieee_to_sbus_float, sbus_float_to_ieee, SBusUDPClient};
use tokio::{join, net::UdpSocket, select, sync::Mutex, time::Instant};

use crate::{
    args::*,
    util::{timeout_or_cancel, AbortReason, PrettyDisplay},
};

use super::args::{Cli, ExportArgs, ReadArgs};

pub async fn run(args: Cli) -> Result<(), Box<dyn Error>> {
    let host_port = format!("{}:{}", args.host, args.port);

    let mut client = ClientImpl::new(args.timeout, host_port);

    client.command_loop().await?;

    Ok(())
}

struct ClientImpl {
    timeout: Duration,
    host_port: String,
    client: Arc<Mutex<Option<Arc<SBusUDPClient>>>>,
    last_table: Option<Table>,
    station: u8,
    offset: i32,
}

impl ClientImpl {
    pub fn new(timeout: Duration, host_port: String) -> Self {
        Self {
            timeout,
            host_port,
            client: Arc::new(Mutex::new(None)),
            last_table: None,
            station: 0,
            offset: 0,
        }
    }

    async fn command_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let client = self.connect_if_needed().await?;

        println!("Attempting to discover station number");
        let start = Instant::now();

        let station = timeout_or_cancel(self.timeout, client.read_sbus_station_number()).await;
        match station {
            Ok(Ok(station)) => self.station = station,
            Ok(Err(err)) => println!("Unable to find station number: {err}"),
            Err(AbortReason::Cancel) => return Ok(()),
            Err(AbortReason::Timeout) => println!("Unable to find station number: Timeout"),
        }

        println!("station = {}", self.station);
        println!();
        println!("Discover: {}ms", (Instant::now() - start).as_millis());
        println!();

        let config = rustyline::Config::builder().build();
        let helper = InteractiveHelper {};

        let mut rl = Editor::<InteractiveHelper, MemHistory>::with_history(config, MemHistory::new())?;
        rl.set_helper(Some(helper));
        let rl = Arc::new(Mutex::new(rl));

        loop {
            let _rl = rl.clone();
            let readline = tokio::spawn(async move { _rl.lock().await.readline("sbus-test> ") }).await?;

            match readline {
                Ok(line) => {
                    _ = rl.lock().await.add_history_entry(line.as_str());

                    println!();

                    let result = self.handle_command(line).await;

                    if let Ok(true) = result {
                        return Ok(());
                    }

                    if let Err(err) = result {
                        println!("{err}");
                    }

                    println!();
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, line: String) -> Result<bool, Box<dyn Error>> {
        let words = shellwords::split(&format!("sbus-test> {}", line))?;

        let cmd = Interactive::try_parse_from(words)?;

        let start = Instant::now();

        let result = match &cmd.command {
            InteractiveCommands::Info => self.info().await,
            InteractiveCommands::Read(args) => self.read(args).await,
            InteractiveCommands::Write(args) => self.write(args).await,
            InteractiveCommands::Export(args) => self.export_csv(args).await,
            InteractiveCommands::Scan(args) => self.scan(args).await,
            InteractiveCommands::Station => self.read_station().await,
            InteractiveCommands::Set(args) => match args.command {
                SetCommands::Station { station } => {
                    self.station = station;
                    println!("station = {station}");
                    return Ok(false);
                }
                SetCommands::Offset { offset } => {
                    self.offset = offset;
                    println!("offset = {offset}");
                    return Ok(false);
                }
                SetCommands::Timeout { timeout } => {
                    self.timeout = timeout;
                    println!("timeout = {}ms", timeout.as_millis());
                    return Ok(false);
                }
            },
            InteractiveCommands::Exit => return Ok(true),
        };

        let dur = Instant::now() - start;

        println!();
        println!("{}: {}ms", cmd.command, dur.as_millis());

        result.map(|_| false)
    }

    async fn info(&mut self) -> Result<(), Box<dyn Error>> {
        let client = self.connect_if_needed().await?;

        let (version, rtc) = join!(
            timeout_or_cancel(self.timeout, client.read_firmware_version(self.station)),
            timeout_or_cancel(self.timeout, client.read_real_time_clock(self.station)),
        );
        let version = version??;
        let rtc = rtc??;

        let rtc = format!(
            "{:02}/{:02}/{:02} {:02}:{:02}:{:02} (Week: {}, Weekday: {})",
            rtc.year, rtc.month, rtc.day, rtc.hour, rtc.minute, rtc.second, rtc.week, rtc.week_day
        );

        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        table.add_row(["Firmware version", &version]);
        table.add_row(["Real-time clock", &rtc]);

        println!("{table}");

        self.last_table = Some(table);

        Ok(())
    }

    async fn scan(&self, args: &ScanArgs) -> Result<(), Box<dyn Error>> {
        let client = self.connect_if_needed().await?;

        let do_scan = || async {
            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(["Station", "Version"]);
            let station_col = ColumnConstraint::Absolute(Width::Fixed(10));
            let result_col = ColumnConstraint::Absolute(Width::Fixed(25));
            table.column_mut(0).unwrap().set_constraint(station_col);
            table.column_mut(1).unwrap().set_constraint(result_col);

            println!("{table}");

            for station in args.min..=args.max {
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);

                let version: String = match timeout_or_cancel(self.timeout, client.read_firmware_version(station)).await {
                    Err(reason) => reason.to_string(),
                    Ok(Ok(version)) => version,
                    Ok(Err(err)) => return Err(err),
                };

                table.add_row([station.to_string(), version]);
                table.column_mut(0).unwrap().set_constraint(station_col);
                table.column_mut(1).unwrap().set_constraint(result_col);

                println!("{table}");
            }

            Ok(())
        };

        select! {
            _ = do_scan() => {}
            _ = tokio::signal::ctrl_c() => {}
        };

        Ok(())
    }

    async fn read_station(&mut self) -> Result<(), Box<dyn Error>> {
        let client = self.connect_if_needed().await?;

        let station = timeout_or_cancel(self.timeout, client.read_sbus_station_number()).await??;

        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        table.add_row(["S-Bus station number", &station.to_string()]);

        println!("{table}");

        self.last_table = Some(table);

        Ok(())
    }

    async fn read(&mut self, args: &ReadArgs) -> Result<(), Box<dyn Error>> {
        let address: u16 = (args.address as i32 + self.offset).try_into().map_err(|_| "Address out of range")?;

        let client = self.connect_if_needed().await?;

        enum ResultType {
            Flags(Vec<bool>),
            Registers(Vec<i32>),
        }

        let result = match args.kind {
            ReadKind::Counters => {
                ResultType::Registers(timeout_or_cancel(self.timeout, client.read_counters(self.station, address, args.length)).await??)
            }
            ReadKind::Flags => ResultType::Flags(timeout_or_cancel(self.timeout, client.read_flags(self.station, address, args.length)).await??),
            ReadKind::Inputs => ResultType::Flags(timeout_or_cancel(self.timeout, client.read_inputs(self.station, address, args.length)).await??),
            ReadKind::Outputs => ResultType::Flags(timeout_or_cancel(self.timeout, client.read_outputs(self.station, address, args.length)).await??),
            ReadKind::Registers => {
                ResultType::Registers(timeout_or_cancel(self.timeout, client.read_registers(self.station, address, args.length)).await??)
            }
            ReadKind::Timers => {
                ResultType::Registers(timeout_or_cancel(self.timeout, client.read_timers(self.station, address, args.length)).await??)
            }
        };

        match result {
            ResultType::Flags(values) => self.print_flags(address, &values),
            ResultType::Registers(values) => self.print_registers(address, &values, args.kind == ReadKind::Registers),
        }

        Ok(())
    }

    async fn write(&self, args: &WriteArgs) -> Result<(), Box<dyn Error>> {
        let address: u16 = (args.address as i32 + self.offset).try_into().map_err(|_| "Address out of range")?;

        match args.kind {
            WriteKind::Flags | WriteKind::Outputs => {
                let mut values: Vec<bool> = vec![];

                for value in args.values.iter() {
                    let value: bool = value.to_lowercase().parse()?;
                    values.push(value);
                }

                let client = self.connect_if_needed().await?;

                match args.kind {
                    WriteKind::Flags => {
                        timeout_or_cancel(self.timeout, client.write_flags(self.station, address, &values)).await??;
                    }
                    WriteKind::Outputs => {
                        timeout_or_cancel(self.timeout, client.write_flags(self.station, address, &values)).await??;
                    }
                    _ => panic!("Never"),
                }

                println!("Wrote {} value(s)", args.values.len());
            }
            WriteKind::Counters | WriteKind::Registers | WriteKind::Timers => {
                let mut values: Vec<i32> = vec![];

                for value in args.values.iter() {
                    match args.datatype {
                        WriteDatatype::Integer => values.push(value.parse::<i32>()?),
                        WriteDatatype::Float => values.push(ieee_to_sbus_float(value.parse::<f64>()?)),
                        WriteDatatype::Hex => values.push(i32::from_str_radix(value, 16)?),
                        WriteDatatype::Bin => values.push(i32::from_str_radix(value, 2)?),
                    }
                }

                let client = self.connect_if_needed().await?;

                match args.kind {
                    WriteKind::Counters => {
                        timeout_or_cancel(self.timeout, client.write_counters(self.station, address, &values)).await??;
                    }
                    WriteKind::Registers => {
                        timeout_or_cancel(self.timeout, client.write_registers(self.station, address, &values)).await??;
                    }
                    WriteKind::Timers => {
                        timeout_or_cancel(self.timeout, client.write_timers(self.station, address, &values)).await??;
                    }
                    _ => panic!("Never"),
                }

                println!("Wrote {} value(s)", args.values.len());
            }
        }

        Ok(())
    }

    fn print_flags(&mut self, address: u16, values: &[bool]) {
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table.set_header(["Address", "Value"]);

        table.column_mut(0).unwrap().set_cell_alignment(CellAlignment::Right);

        for (offset, value) in values.iter().enumerate() {
            let index: i32 = address as i32 + offset as i32 - self.offset;
            table.add_row([index.to_string(), value.to_string().to_uppercase()]);
        }

        println!("{table}");

        self.last_table = Some(table);
    }

    fn print_registers(&mut self, address: u16, values: &[i32], show_float: bool) {
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        let mut header = Vec::with_capacity(17);
        header.push("Address");
        header.push("Integer");
        if show_float {
            header.push("Float");
        }
        header.push("Hex");
        header.push("Bin");

        let column_count = header.len();
        table.set_header(header);

        table.column_iter_mut().for_each(|c| c.set_cell_alignment(CellAlignment::Right));
        table
            .column_iter_mut()
            .skip(column_count - 2)
            .for_each(|c| c.set_cell_alignment(CellAlignment::Left));

        for (offset, value) in values.iter().enumerate() {
            let index: i32 = address as i32 + offset as i32 - self.offset;

            let mut row: Vec<String> = Vec::with_capacity(column_count);

            row.push(index.to_string()); // Address
            row.push(format!("{}", *value)); // Integer

            if show_float {
                row.push(sbus_float_to_ieee(*value).pretty()); // Float
            }

            row.push(format!("{:04X} {:04X}", *value >> 16 & 0xFFFF, *value & 0xFFFF)); // Hex
            row.push(format!(
                "{:04b} {:04b} {:04b} {:04b} {:04b} {:04b} {:04b} {:04b}",
                *value >> 28 & 0xF,
                *value >> 24 & 0xF,
                *value >> 20 & 0xF,
                *value >> 16 & 0xF,
                *value >> 12 & 0xF,
                *value >> 8 & 0xF,
                *value >> 4 & 0xF,
                *value & 0xF
            )); // Bin

            table.add_row(row);
        }

        println!("{table}");

        self.last_table = Some(table);
    }

    async fn export_csv(&self, args: &ExportArgs) -> Result<(), Box<dyn Error>> {
        let table = match &self.last_table {
            Some(table) => table,
            None => {
                println!("Nothing to export");
                return Ok(());
            }
        };

        let mut writer = csv::Writer::from_path(&args.filename)?;

        let header = table.header().unwrap().cell_iter().map(|c| c.content()).collect::<Vec<String>>();
        writer.write_record(header)?;

        for row in table.row_iter() {
            let record = row.cell_iter().map(|c| c.content()).collect::<Vec<String>>();
            writer.write_record(record)?;
        }
        writer.flush()?;

        println!("Exported");

        Ok(())
    }

    async fn connect_if_needed(&self) -> Result<Arc<SBusUDPClient>, Box<dyn Error>> {
        if let Some(client) = self.client.lock().await.as_ref() {
            return Ok(client.clone());
        }

        print!("Binding...");
        std::io::stdout().flush()?;

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(&self.host_port).await?;

        println!(" Bound");
        println!();

        let (client, handle) = SBusUDPClient::new(socket);

        let client = Arc::new(client);

        _ = self.client.lock().await.insert(client.clone());

        let client_ = self.client.clone();

        tokio::spawn(async move {
            let result = handle.await.unwrap_or(Ok(()));
            _ = client_.lock().await.take();
            println!();
            println!();
            match result {
                Ok(_) => println!("Socket closed"),
                Err(err) => println!("{err}"),
            }
            println!();
        });

        Ok(client)
    }
}

#[derive(Helper, Hinter, Validator, Highlighter)]
struct InteractiveHelper {}
const COMPLETIONS: [&str; 20] = [
    "info",
    "scan ",
    "station",
    "read counters ",
    "read flags ",
    "read inputs ",
    "read outputs ",
    "read registers ",
    "read timers ",
    "write counters ",
    "write flags ",
    "write outputs ",
    "write registers ",
    "write timers ",
    "set offset ",
    "set timeout ",
    "set station ",
    "export ",
    "help",
    "exit",
];

impl Completer for InteractiveHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut matches = vec![];

        for cmd in COMPLETIONS {
            if cmd.starts_with(line) {
                matches.push(String::from(&cmd[pos..]));
            }
        }

        let _ = (line, pos, ctx);
        Ok((pos, matches))
    }
}
