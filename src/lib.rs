use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use clap_verbosity_flag::Verbosity;
use serde::{Deserialize, Serialize};

pub mod sync;

pub enum DeviceType {
    Device,
    Host,
    Name,
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about="Modbus Master Simulator", long_about = None)]
pub struct Args {
    /// DEVICE: Serial port when using ModBus RTU protocol.
    /// HOST: Host name or dotted IP address when using ModBus/TCP
    /// NAME: Name of the device in the configuration file
    ///
    /// DEVICE: COM1, COM2 ... on Windows. /dev/ttyS0, /dev/ttyS1 ...  on Linux. /dev/ser1, /dev/ser2 ...    on QNX
    /// HOST: 192.168.10.13. 192.168.10.13:502. 192.168.10.13:501, After the IP address, you can specify the port number separated by a colon.
    /// NAME: The name of the device in the configuration file. The configuration file is specified by the -conf option. for example: dpoll em2_0 -r 1 -c 10 -t 4
    #[clap(name = "DEVICE|HOST|NAME", verbatim_doc_comment)]
    #[arg(next_line_help = true)]
    pub device: String,

    /// List of values to be written.
    ///
    /// If none specified (default) dpoll reads data.
    /// If negative numbers are provided, it will precede the list of
    /// data to be written by two dashes ('--'). for example : dpoll -t4:i16 /dev/ttyUSB0 -- 123 -1568 8974 -12
    #[arg(group = "input", verbatim_doc_comment)]
    pub writevalues: Option<Vec<String>>,

    /// mode (tcp, rtu, rtu-in-tcp, iec104) iec104 is not supported yet
    #[clap(short, long, default_value = "tcp")]
    pub mode: Option<Mode>,

    /// Start reference (supported dec/hex/bin three formats)
    #[clap(short, long, default_value = "0")]
    #[arg(value_parser = parse_reference)]
    pub reference: Vec<u16>,

    /// Slave address (1-255 for rtu, 0-255 for tcp) for reading,
    ///
    /// it is possible to give an address list separated by commas or colons, for example : -a 32,33,34,36:40 read [32,33,34,36,37,38,39,40]
    #[clap(short = 'a', long, default_value = "1")]
    pub slave: Vec<u8>,

    /// Number of values to read (1-125)
    #[clap(short, long, default_value = "1")]
    #[arg(group = "input")]
    pub count: Option<u16>,

    /// which data type should be read
    ///
    /// -t 0          Discrete output (coil) data type (binary 0 or 1)
    /// -t 1          Discrete input data type (binary 0 or 1)
    /// -t 3          16-bit output(holding) register data type
    /// -t 3:i16      16-bit integer data type in output(holding) register table
    /// -t 3:u16      16-bit unsigned integer data type in output(holding) register table
    /// -t 3:i32      32-bit integer data type in output(holding) register table
    /// -t 3:u32      32-bit unsigned integer data type in output(holding) register table
    /// -t 3:hex16    16-bit output(holding) register data type with hex display
    /// -t 3:hex32    32-bit output(holding) register data type with hex display
    /// -t 3:bin16    16-bit output(holding) register data type with bin display
    /// -t 3:bin32    32-bit output(holding) register data type with bin display
    /// -t 3:f32      32-bit float data type in output(holding) register table
    /// -t 4          16-bit input register data type (default)
    /// -t 4:i16      16-bit integer data type in input register table
    /// -t 4:u16      16-bit unsigned integer data type in input register table
    /// -t 4:i32      32-bit integer data type in input register table
    /// -t 4:u32      32-bit unsigned integer data type in input register table
    /// -t 4:hex16    16-bit input register data type with hex display
    /// -t 4:hex32    32-bit input register data type with hex display
    /// -t 4:bin16    16-bit input register data type with bin display
    /// -t 4:bin32    32-bit input register data type with bin display
    /// -t 4:f32      32-bit float data type in input register table
    #[clap(short, long, default_value = "3", verbatim_doc_comment)]
    #[arg(value_parser = parse_type)]
    pub r#type: Option<Type>,

    /// Little endian word order for 32-bit integer and float [default = Big endian]
    #[clap(short = 'L')]
    pub little_endian: bool,

    /// Poll only once only, otherwise every poll rate interval
    #[clap(short = '1')]
    pub once: bool,

    /// Poll rate in ms, ( > 10)
    #[clap(short = 'l', default_value = "1000")]
    pub poll_rate: Option<u64>,

    /// Time-out in seconds (0.01 - 10.00)
    #[clap(short = 'o', long, default_value = "1.00")]
    #[arg(value_parser = parse_timeout)]
    pub timeout: Option<Duration>,

    /// TCP port number (502 is default)
    #[clap(short, long, default_value = "502")]
    pub port: Option<u16>,

    /// Baudrate (1200-921600)
    #[clap(short, default_value = "9600")]
    pub baudrate: Option<u32>,

    /// Databits (7 or 8, 8 for RTU)
    #[clap(short, default_value = "8")]
    pub databits: Option<u8>,

    /// Stopbits (1 or 2)
    #[clap(short, default_value = "1")]
    pub stopbits: Option<u8>,

    /// Parity (none, even, odd)
    #[clap(short = 'P', default_value = "none")]
    pub parity: Option<String>,

    /// Verbose mode.  Causes dpoll to print debugging messages about
    #[command(flatten)]
    pub verbose: Verbosity,

    /// The path to the configuration file
    #[clap(long, default_value = "/home/work/deploy/device/conf/device_list.json")]
    pub conf: Option<String>,
    // // TODO
    // /// Read the description of the type, the current status, and other information specific to a remote device (RTU only)
    // #[clap(short = 'u')]
    // pub b_is_report_slave_id: bool,

    // DEPRECATED
    // -0            First reference is 0 (PDU addressing) instead 1

    // TODO
    // -R            RS-485 mode (/RTS on (0) after sending)
    // -F            RS-485 mode (/RTS on (0) when sending)
}

impl Args {
    pub fn device_type(&mut self) -> DeviceType {
        let d = self.device.to_lowercase();
        if (d.contains("com") || d.contains("tty")) || d.contains("ser") {
            DeviceType::Device
        } else if d.parse::<Ipv4Addr>().is_ok() || d.parse::<SocketAddr>().is_ok() {
            DeviceType::Host
        } else {
            DeviceType::Name
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Functions {
    Coil,
    DiscreteInput,
    InputRegister,
    HoldingRegister,
    Unknown = -1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Formats {
    U16,
    I16,
    I32,
    U32,
    F32,
    Hex16,
    Hex32,
    Bin16,
    Bin32,
    String,
    Unknown = -1,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub function: Functions,
    pub format: Formats,
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let function;
        let format;
        if s.contains(':') {
            let mut iter = s.split(':');
            let function_str = iter.next().unwrap();
            let format_str = iter.next().unwrap();
            function = match function_str {
                "0" => Functions::Coil,
                "1" => Functions::DiscreteInput,
                "3" => Functions::HoldingRegister,
                "4" => Functions::InputRegister,
                _ => Err(anyhow::anyhow!("Unsupported function"))?,
            };
            format = match format_str {
                "u16" => Formats::U16,
                "i16" => Formats::I16,
                "hex16" => Formats::Hex16,
                "hex32" => Formats::Hex32,
                "u32" => Formats::U32,
                "i32" => Formats::I32,
                "f32" => Formats::F32,
                "bin16" => Formats::Bin16,
                "bin32" => Formats::Bin32,
                "string" => Err(anyhow::anyhow!("Unsupported format"))?,
                _ => Err(anyhow::anyhow!("Unsupported format"))?,
            };

            if (function == Functions::Coil || function == Functions::DiscreteInput)
                && format != Formats::Bin16
            {
                Err(anyhow::anyhow!("Coil and DiscreteInput only supported Bin"))?;
            }
        } else {
            function = match s {
                "0" => {
                    format = Formats::Bin16;
                    Functions::Coil
                }
                "1" => {
                    format = Formats::Bin16;
                    Functions::DiscreteInput
                }
                "3" => {
                    format = Formats::U16;
                    Functions::HoldingRegister
                }
                "4" => {
                    format = Formats::U16;
                    Functions::InputRegister
                }
                _ => {
                    format = Formats::U16;
                    Err(anyhow::anyhow!("Unsupported function"))?
                }
            };
        }

        Ok(Type { function, format })
    }
}

fn parse_type(s: &str) -> Result<Type> {
    s.parse()
}

fn parse_timeout(s: &str) -> Result<Duration> {
    let f = s.parse::<f32>()?;
    Ok(Duration::from_secs_f32(f))
}

fn parse_reference(s: &str) -> Result<u16> {
    if s.parse::<u16>().is_ok() {
        s.parse::<u16>().map_err(|e| e.into())
    } else if s.starts_with("0x") && u16::from_str_radix(s.trim_start_matches("0x"), 16).is_ok() {
        Ok(u16::from_str_radix(s.trim_start_matches("0x"), 16)?)
    } else if s.starts_with("0b") && u16::from_str_radix(s.trim_start_matches("0b"), 2).is_ok() {
        Ok(u16::from_str_radix(s.trim_start_matches("0b"), 2)?)
    } else {
        Err(anyhow::anyhow!("only supported dec/hex/bin formats"))?
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Signature {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Remote {
    pub protocol: String,
    pub mode: String,
    pub host: Option<String>,
    pub port: Option<String>,
    pub parity: Option<String>,
    pub device: Option<String>,
    pub baud: Option<u32>,
    pub slave_id: Option<u8>,
    pub data_bit: Option<u8>,
    pub stop_bit: Option<u8>,
    pub timeout_ms: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Device {
    pub signature: Signature,
    pub remote: Remote,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeviceList {
    #[serde(rename = "device")]
    pub devices: Vec<Device>,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Tcp,
    Rtu,
    // 透传
    RtuInTcp,
    IEC104,
}

#[cfg(test)]
mod tests {}
