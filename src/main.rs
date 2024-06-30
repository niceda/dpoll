use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use dpoll::{sync::rtuintcp, Args, Device, DeviceList, DeviceType, Formats, Functions, Mode};
use lazy_static::lazy_static;
use std::{
    fs::File,
    io::BufReader,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio_modbus::prelude::*;

lazy_static! {
    static ref TRANSMIT_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    static ref RECEIVE_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    static ref ERROR_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    check_args(&mut args)?;

    if args.verbose.log_level().is_some() {
        print_args(&args);
    }

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let argsc = args.clone();
    ctrlc::set_handler(move || {
        if !argsc.once && argsc.writevalues.is_none() {
            let tc = TRANSMIT_COUNT.load(std::sync::atomic::Ordering::Relaxed);
            let rc = RECEIVE_COUNT.load(std::sync::atomic::Ordering::Relaxed);
            let ec = ERROR_COUNT.load(std::sync::atomic::Ordering::Relaxed);
            println!(
                "--- {} poll statistics --- \n
{} frames transmitted, {} received, {} errors, {:.1}% frame loss\n",
                argsc.device,
                tc,
                rc,
                ec,
                (tc - rc) as f32 / tc as f32 * 100.0
            );
        }
        println!("everything was closed.\nHave a nice day !");
        std::process::exit(0);
    })?;

    match args.device_type() {
        DeviceType::Device => match args.mode.unwrap() {
            Mode::Rtu => rtu_client(args)?,
            Mode::Tcp => unreachable!(),
            Mode::RtuInTcp => unreachable!(),
            Mode::IEC104 => todo!(),
        },
        _ => match args.mode.unwrap() {
            Mode::Rtu => rtu_client(args)?,
            Mode::Tcp => tcp_client(args)?,
            Mode::RtuInTcp => rtu_in_tcp_client(args)?,
            Mode::IEC104 => todo!(),
        },
    }

    Ok(())
}

fn run<T: SyncWriter + SyncReader>(mut ctx: T, args: Args) -> Result<()> {
    loop {
        let writevalues = args.writevalues.clone();
        let function = args.r#type.clone().unwrap().function;
        let format = args.r#type.clone().unwrap().format;
        let slave = args.slave.clone();
        let reference = args.reference.clone();
        let count = args.count.unwrap();
        let mut nregs = count;

        if format == Formats::I32
            || format == Formats::I32abcd
            || format == Formats::I32badc
            || format == Formats::I32cdab
            || format == Formats::I32dcba
            || format == Formats::U32
            || format == Formats::U32abcd
            || format == Formats::U32badc
            || format == Formats::U32cdab
            || format == Formats::U32dcba
            || format == Formats::F32
            || format == Formats::F32abcd
            || format == Formats::F32badc
            || format == Formats::F32cdab
            || format == Formats::F32dcba
            || format == Formats::Hex32
            || format == Formats::Bin32
        {
            nregs *= 2;
        }

        // write
        if writevalues.is_some() {
            TRANSMIT_COUNT.fetch_add(1, Ordering::Relaxed);
            let writevalues = writevalues.unwrap();
            let rs;
            match format {
                Formats::Bin16
                    if function == Functions::Coil || function == Functions::DiscreteInput =>
                {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<bool>().unwrap())
                        .collect::<Vec<bool>>();
                    if wd.len() == 1 {
                        rs = ctx.write_single_coil(reference[0], wd[0]);
                    } else {
                        rs = ctx.write_multiple_coils(reference[0], &wd);
                    }
                }
                Formats::U16 | Formats::Bin16 | Formats::Hex16 => {
                    let wd = writevalues
                        .iter()
                        .map(|v| {
                            if v.parse::<u16>().is_ok() {
                                v.parse::<u16>().unwrap()
                            } else if v.starts_with("0x")
                                && u16::from_str_radix(v.trim_start_matches("0x"), 16).is_ok()
                            {
                                u16::from_str_radix(v.trim_start_matches("0x"), 16).unwrap()
                            } else {
                                u16::from_str_radix(v.trim_start_matches("0b"), 2).unwrap()
                            }
                        })
                        .collect::<Vec<u16>>();
                    if wd.len() == 1 {
                        rs = ctx.write_single_register(reference[0], wd[0]);
                    } else {
                        rs = ctx.write_multiple_registers(reference[0], &wd);
                    }
                }
                Formats::I16 => {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<i16>().unwrap())
                        .collect::<Vec<i16>>();
                    if wd.len() == 1 {
                        rs = ctx.write_single_register(reference[0], wd[0] as u16);
                    } else {
                        rs = ctx.write_multiple_registers(
                            reference[0],
                            &wd.iter().map(|v| *v as u16).collect::<Vec<u16>>(),
                        );
                    }
                }
                Formats::I32 => {
                    let wd = writevalues.iter().map(|v| v.parse::<i32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            if args.little_endian {
                                acc.push(u16::from_be_bytes([data[2], data[3]]));
                                acc.push(u16::from_be_bytes([data[0], data[1]]));
                            } else {
                                acc.push(u16::from_be_bytes([data[0], data[1]]));
                                acc.push(u16::from_be_bytes([data[2], data[3]]));
                            }
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::I32abcd => {
                    let wd = writevalues.iter().map(|v| v.parse::<i32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::I32cdab => {
                    let wd = writevalues.iter().map(|v| v.parse::<i32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::I32badc => {
                    let wd = writevalues.iter().map(|v| v.parse::<i32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[1], data[0]]));
                            acc.push(u16::from_be_bytes([data[3], data[2]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::I32dcba => {
                    let wd = writevalues.iter().map(|v| v.parse::<i32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[3], data[2]]));
                            acc.push(u16::from_be_bytes([data[1], data[0]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::U32 | Formats::Hex32 | Formats::Bin32 => {
                    // check_args already checked Formats::I32
                    let wd = writevalues
                        .iter()
                        .map(|v| {
                            if v.parse::<u32>().is_ok() {
                                v.parse::<u32>().unwrap()
                            } else if v.starts_with("0x")
                                && u32::from_str_radix(v.trim_start_matches("0x"), 16).is_ok()
                            {
                                u32::from_str_radix(v.trim_start_matches("0x"), 16).unwrap()
                            } else {
                                u32::from_str_radix(v.trim_start_matches("0b"), 2).unwrap()
                            }
                        })
                        .collect::<Vec<u32>>();

                    let wd = wd.iter().fold(Vec::new(), |mut acc, v| {
                        let data = v.to_be_bytes();
                        if args.little_endian {
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                        } else {
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                        }
                        acc
                    });
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::U32abcd => {
                    let wd = writevalues.iter().map(|v| v.parse::<u32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::U32cdab => {
                    let wd = writevalues.iter().map(|v| v.parse::<u32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::U32badc => {
                    let wd = writevalues.iter().map(|v| v.parse::<u32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[1], data[0]]));
                            acc.push(u16::from_be_bytes([data[3], data[2]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::U32dcba => {
                    let wd = writevalues.iter().map(|v| v.parse::<u32>().unwrap()).fold(
                        Vec::new(),
                        |mut acc, v| {
                            let data = v.to_be_bytes();
                            acc.push(u16::from_be_bytes([data[3], data[2]]));
                            acc.push(u16::from_be_bytes([data[1], data[0]]));
                            acc
                        },
                    );
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::F32 => {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<f32>().unwrap())
                        .map(|v| v.to_bits())
                        .collect::<Vec<u32>>();

                    let wd = wd.iter().fold(Vec::new(), |mut acc, v| {
                        let data = v.to_be_bytes();
                        if args.little_endian {
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                        } else {
                            acc.push(u16::from_be_bytes([data[0], data[1]]));
                            acc.push(u16::from_be_bytes([data[2], data[3]]));
                        }
                        acc
                    });
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::F32abcd => {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<f32>().unwrap())
                        .map(|v| v.to_bits())
                        .collect::<Vec<u32>>();

                    let wd = wd.iter().fold(Vec::new(), |mut acc, v| {
                        let data = v.to_be_bytes();
                        acc.push(u16::from_be_bytes([data[0], data[1]]));
                        acc.push(u16::from_be_bytes([data[2], data[3]]));
                        acc
                    });
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::F32cdab => {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<f32>().unwrap())
                        .map(|v| v.to_bits())
                        .collect::<Vec<u32>>();

                    let wd = wd.iter().fold(Vec::new(), |mut acc, v| {
                        let data = v.to_be_bytes();
                        acc.push(u16::from_be_bytes([data[2], data[3]]));
                        acc.push(u16::from_be_bytes([data[0], data[1]]));
                        acc
                    });
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::F32badc => {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<f32>().unwrap())
                        .map(|v| v.to_bits())
                        .collect::<Vec<u32>>();

                    let wd = wd.iter().fold(Vec::new(), |mut acc, v| {
                        let data = v.to_be_bytes();
                        acc.push(u16::from_be_bytes([data[1], data[0]]));
                        acc.push(u16::from_be_bytes([data[3], data[2]]));
                        acc
                    });
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::F32dcba => {
                    let wd = writevalues
                        .iter()
                        .map(|v| v.parse::<f32>().unwrap())
                        .map(|v| v.to_bits())
                        .collect::<Vec<u32>>();

                    let wd = wd.iter().fold(Vec::new(), |mut acc, v| {
                        let data = v.to_be_bytes();
                        acc.push(u16::from_be_bytes([data[3], data[2]]));
                        acc.push(u16::from_be_bytes([data[1], data[0]]));
                        acc
                    });
                    rs = ctx.write_multiple_registers(reference[0], &wd);
                }
                Formats::String => {
                    todo!()
                }
                Formats::Unknown => unreachable!(),
            }
            if rs.is_ok() {
                RECEIVE_COUNT.fetch_add(1, Ordering::Relaxed);
                println!("Write {} references.", count);
            } else {
                println!("Write {:?} failed: {:?}", function, rs.err().unwrap());
            }
        } else {
            // read
            for slave in slave {
                ctx.set_slave(Slave(slave));
                TRANSMIT_COUNT.fetch_add(1, Ordering::Relaxed);
                print!("-- Polling slave {}...", slave);
                if !args.once {
                    println!(" Ctrl-C to stop");
                } else {
                    println!();
                }
                for &r in reference.iter() {
                    match function {
                        Functions::Coil => {
                            let rs = ctx.read_coils(r, nregs).map(|v| {
                                v.iter()
                                    .map(|v| if !(*v) { 0 } else { 1 })
                                    .collect::<Vec<u16>>()
                            });
                            print_read_value(r, count, &format, &function, args.little_endian, rs);
                        }
                        Functions::DiscreteInput => {
                            let rs = ctx.read_discrete_inputs(r, nregs).map(|v| {
                                v.iter()
                                    .map(|v| if !(*v) { 0 } else { 1 })
                                    .collect::<Vec<u16>>()
                            });
                            print_read_value(r, count, &format, &function, args.little_endian, rs);
                        }
                        Functions::HoldingRegister => {
                            let rs = ctx.read_holding_registers(r, nregs);
                            print_read_value(r, count, &format, &function, args.little_endian, rs);
                        }
                        Functions::InputRegister => {
                            let rs = ctx.read_input_registers(r, nregs);
                            print_read_value(r, count, &format, &function, args.little_endian, rs);
                        }
                        Functions::Unknown => {
                            todo!()
                        }
                    }
                }
                if !args.once {
                    std::thread::sleep(Duration::from_millis(args.poll_rate.unwrap()));
                }
            }
        }

        if args.once {
            break;
        }
    }

    Ok(())
}

fn tcp_client(args: Args) -> Result<()> {
    let socket_addr = SocketAddr::new(
        IpAddr::V4(args.device.parse::<Ipv4Addr>().unwrap()),
        args.port.unwrap(),
    );

    loop {
        match sync::tcp::connect_with_timeout(socket_addr, args.timeout) {
            Ok(ctx) => {
                run(ctx, args)?;
                break;
            }
            Err(e) => {
                if args.once {
                    return Err(anyhow::anyhow!("Connect error: {:?}", e));
                }
                println!("Connect error: {:?}", e);
            }
        }
    }
    Ok(())
}

fn rtu_client(args: Args) -> Result<()> {
    let path = args.device.clone();
    let slave = Slave(args.slave[0]);
    let builder = tokio_serial::new(path, args.baudrate.unwrap())
        .data_bits(match args.databits.unwrap() {
            7 => tokio_serial::DataBits::Seven,
            8 => tokio_serial::DataBits::Eight,
            _ => tokio_serial::DataBits::Eight,
        })
        .parity(match args.parity.clone().unwrap().as_str() {
            "none" => tokio_serial::Parity::None,
            "even" => tokio_serial::Parity::Even,
            "odd" => tokio_serial::Parity::Odd,
            _ => tokio_serial::Parity::None,
        })
        .stop_bits(match args.stopbits.unwrap() {
            1 => tokio_serial::StopBits::One,
            2 => tokio_serial::StopBits::Two,
            _ => tokio_serial::StopBits::One,
        })
        // .flow_control(tokio_serial::FlowControl::None)
        .timeout(args.timeout.unwrap());

    loop {
        match sync::rtu::connect_slave_with_timeout(&builder, slave, args.timeout) {
            Ok(ctx) => {
                run(ctx, args)?;
                break;
            }
            Err(e) => {
                if args.once {
                    return Err(anyhow::anyhow!("Connect error: {:?}", e));
                }
                println!("Connect error: {:?}", e);
            }
        }
    }

    Ok(())
}

fn rtu_in_tcp_client(args: Args) -> Result<()> {
    let socket_addr = SocketAddr::new(
        IpAddr::V4(args.device.parse::<Ipv4Addr>().unwrap()),
        args.port.unwrap(),
    );

    let timeout = args.timeout;
    let slave = Slave(args.slave[0]);

    let sync_ctx = rtuintcp::connect_slave_with_timeout(socket_addr, slave, timeout)?;
    run(sync_ctx, args)?;
    Ok(())
}

#[allow(dead_code)]
fn iec104_client(_args: Args) -> Result<()> {
    todo!()
}

fn print_read_value(
    mut addr: u16,
    count: u16,
    format: &Formats,
    function: &Functions,
    little_endian: bool,
    rs: Result<Vec<u16>, std::io::Error>,
) {
    if rs.is_ok() {
        RECEIVE_COUNT.fetch_add(1, Ordering::Relaxed);
        let data = rs.unwrap();
        for c in 0..count as usize {
            // print!("{}", format!("[{}({:#04X})]: \t", addr, addr).green());
            print!("[{}({:#04X})]: \t", addr, addr);
            match format {
                Formats::U16 => {
                    if (data[c] & 0x8000) != 0 {
                        println!("{} ({})", data[c], data[c] as i16);
                    } else {
                        println!("{}", data[c]);
                    }
                    addr += 1;
                }
                Formats::I16 => {
                    println!("{}", data[c] as i16);
                    addr += 1;
                }
                Formats::I32 => {
                    let v = extract_data(&data, 2 * c, little_endian);
                    println!("{}", v as i32);
                    addr += 2;
                }
                Formats::I32abcd => {
                    let v = extract_data(&data, 2 * c, false);
                    println!("{}", v as i32);
                    addr += 2;
                }
                Formats::I32cdab => {
                    let v = extract_data(&data, 2 * c, true);
                    println!("{}", v as i32);
                    addr += 2;
                }
                Formats::I32badc => {
                    let v = extract_data_32(&data, 2 * c, 1, 0, 3, 2);
                    println!("{}", v as i32);
                    addr += 2;
                }
                Formats::I32dcba => {
                    let v = extract_data_32(&data, 2 * c, 3, 2, 1, 0);
                    println!("{}", v as i32);
                    addr += 2;
                }
                Formats::U32 => {
                    let v = extract_data(&data, 2 * c, little_endian);
                    if v & 0x80000000 != 0 {
                        println!("{} ({})", v, v as i32);
                    } else {
                        println!("{}", v);
                    }
                    addr += 2;
                }
                Formats::U32abcd => {
                    let v = extract_data(&data, 2 * c, false);
                    if v & 0x80000000 != 0 {
                        println!("{} ({})", v, v as i32);
                    } else {
                        println!("{}", v);
                    }
                    addr += 2;
                }
                Formats::U32cdab => {
                    let v = extract_data(&data, 2 * c, true);
                    if v & 0x80000000 != 0 {
                        println!("{} ({})", v, v as i32);
                    } else {
                        println!("{}", v);
                    }
                    addr += 2;
                }
                Formats::U32badc => {
                    let v = extract_data_32(&data, 2 * c, 1, 0, 3, 2);
                    if v & 0x80000000 != 0 {
                        println!("{} ({})", v, v as i32);
                    } else {
                        println!("{}", v);
                    }
                    addr += 2;
                }
                Formats::U32dcba => {
                    let v = extract_data_32(&data, 2 * c, 3, 2, 1, 0);
                    if v & 0x80000000 != 0 {
                        println!("{} ({})", v, v as i32);
                    } else {
                        println!("{}", v);
                    }
                    addr += 2;
                }
                Formats::F32 => {
                    let v = extract_data(&data, 2 * c, little_endian);
                    println!("{}", f32::from_bits(v));
                    addr += 2;
                }
                Formats::F32abcd => {
                    let v = extract_data(&data, 2 * c, false);
                    println!("{}", f32::from_bits(v));
                    addr += 2;
                }
                Formats::F32cdab => {
                    let v = extract_data(&data, 2 * c, true);
                    println!("{}", f32::from_bits(v));
                    addr += 2;
                }
                Formats::F32badc => {
                    let v = extract_data_32(&data, 2 * c, 1, 0, 3, 2);
                    println!("{}", f32::from_bits(v));
                    addr += 2;
                }
                Formats::F32dcba => {
                    let v = extract_data_32(&data, 2 * c, 3, 2, 1, 0);
                    println!("{}", f32::from_bits(v));
                    addr += 2;
                }
                Formats::Hex16 => {
                    println!("{:#04X}", data[c]);
                    addr += 1;
                }
                Formats::Hex32 => {
                    let v = extract_data(&data, 2 * c, little_endian);
                    println!("{:#010X}", v);
                    addr += 2;
                }
                Formats::Bin16
                    if *function == Functions::Coil || *function == Functions::DiscreteInput =>
                {
                    println!("{:b}", data[c]);
                    addr += 1;
                }
                Formats::Bin16 => {
                    println!("{:016b}", data[c]);
                    addr += 1;
                }
                Formats::Bin32 => {
                    let v = extract_data(&data, 2 * c, little_endian);
                    println!("{:032b}", v);
                    addr += 2;
                }

                Formats::String => {
                    // addr += 1;
                    todo!()
                }
                Formats::Unknown => unreachable!(),
            }
        }
    } else {
        ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
        println!("Read {:?} failed: {:?}", function, rs.err().unwrap());
    }
}

fn check_args(args: &mut Args) -> Result<()> {
    if args.writevalues.is_some() {
        if args.slave.len() > 1 {
            Err(anyhow::anyhow!("Only one slave can write"))?;
        }

        match args.r#type.clone().unwrap().function {
            Functions::DiscreteInput | Functions::InputRegister => {
                Err(anyhow::anyhow!("Unable to write read-only element"))?;
            }
            Functions::Coil => {
                for v in args.writevalues.clone().unwrap() {
                    if v.parse::<bool>().is_err() {
                        Err(anyhow::anyhow!("Write value {} must be bool", v))?;
                    }
                }
            }
            Functions::HoldingRegister => match args.r#type.clone().unwrap().format {
                Formats::Bin16
                    if args.r#type.as_ref().unwrap().function == Functions::Coil
                        || args.r#type.as_ref().unwrap().function == Functions::DiscreteInput =>
                {
                    for v in args.writevalues.clone().unwrap() {
                        if v.parse::<bool>().is_err() {
                            Err(anyhow::anyhow!("Write value {} must be bool", v))?;
                        }
                    }
                }
                Formats::U16 | Formats::Hex16 | Formats::Bin16 => {
                    for v in args.writevalues.clone().unwrap() {
                        if v.parse::<u16>().is_err()
                            && (v.starts_with("0x")
                                && u16::from_str_radix(v.trim_start_matches("0x"), 16).is_err())
                            && (v.starts_with("0b")
                                && u16::from_str_radix(v.trim_start_matches("0b"), 2).is_err())
                        {
                            Err(anyhow::anyhow!("Write value {} must be u16/hex16/bin16", v))?;
                        }
                    }
                }
                Formats::I16 => {
                    for v in args.writevalues.clone().unwrap() {
                        if v.parse::<i16>().is_err() {
                            Err(anyhow::anyhow!("Write value {} must be int16", v))?;
                        }
                    }
                }
                Formats::U32
                | Formats::U32abcd
                | Formats::U32cdab
                | Formats::U32badc
                | Formats::U32dcba
                | Formats::Hex32
                | Formats::Bin32 => {
                    for v in args.writevalues.clone().unwrap() {
                        if v.parse::<u32>().is_err()
                            && (v.starts_with("0x")
                                && u32::from_str_radix(v.trim_start_matches("0x"), 16).is_err())
                            && (v.starts_with("0b")
                                && u32::from_str_radix(v.trim_start_matches("0b"), 2).is_err())
                        {
                            Err(anyhow::anyhow!("Write value {} must be u32/hex32/bin32", v))?;
                        }
                    }
                }
                Formats::I32
                | Formats::I32abcd
                | Formats::I32cdab
                | Formats::I32badc
                | Formats::I32dcba => {
                    for v in args.writevalues.clone().unwrap() {
                        if v.parse::<i32>().is_err() {
                            Err(anyhow::anyhow!("Write value {} must be int32", v))?;
                        }
                    }
                }
                Formats::F32
                | Formats::F32abcd
                | Formats::F32cdab
                | Formats::F32badc
                | Formats::F32dcba => {
                    for v in args.writevalues.clone().unwrap() {
                        if v.parse::<f32>().is_err() {
                            Err(anyhow::anyhow!("Write value {} must be float", v))?;
                        }
                    }
                }
                Formats::String => {
                    Err(anyhow::anyhow!("You can use string format only for output"))?
                }
                Formats::Unknown => {
                    Err(anyhow::anyhow!("Unsupported format"))?;
                }
            },
            Functions::Unknown => {
                Err(anyhow::anyhow!("Unsupported function"))?;
            }
        }

        args.once = true;
        args.count = Some(args.writevalues.clone().unwrap().len() as u16);
    }

    match args.device_type() {
        DeviceType::Device => {
            args.mode = Some(Mode::Rtu);
        }
        DeviceType::Host => {
            if args.mode.unwrap() == Mode::Rtu {
                args.mode = Some(Mode::Tcp);
            }
            if args.device.parse::<SocketAddr>().is_ok() {
                let sd = args.device.parse::<SocketAddr>().unwrap();
                args.device = sd.ip().to_string();
                args.port = Some(sd.port());
            }
        }
        DeviceType::Name => {
            let conf = File::open(args.clone().conf.unwrap())?;
            let reader = BufReader::new(conf);
            let device_list: DeviceList = serde_json::from_reader(reader)?;

            let d = device_list
                .devices
                .iter()
                .filter(|d| d.signature.name == args.device)
                .collect::<Vec<&Device>>();

            if d.is_empty() {
                Err(anyhow::anyhow!("No device found"))?;
            }
            if d.len() > 1 {
                Err(anyhow::anyhow!("Multiple devices found: {:?}", d))?;
            }

            let device = d[0];
            if device.remote.protocol.to_lowercase() == "modbus" {
                if device.remote.mode.to_lowercase() == "rtu" {
                    args.mode = Some(Mode::Rtu);
                    args.device = device.remote.device.clone().unwrap();
                    if device.remote.slave_id.is_some() {
                        args.slave.clear();
                        args.slave.push(device.remote.slave_id.unwrap());
                    }
                    if device.remote.baud.is_some() {
                        args.baudrate = Some(device.remote.baud.unwrap());
                    }
                    if device.remote.data_bit.is_some() {
                        args.databits = Some(device.remote.data_bit.unwrap());
                    }
                    if device.remote.stop_bit.is_some() {
                        args.stopbits = Some(device.remote.stop_bit.unwrap());
                    }
                    if device.remote.parity.is_some() {
                        args.parity = Some(device.remote.parity.clone().unwrap());
                    }
                    if device.remote.timeout_ms.is_some() {
                        args.timeout = Some(Duration::from_secs_f32(
                            device.remote.timeout_ms.unwrap() as f32 / 1000.0,
                        ));
                    }
                } else if device.remote.mode.to_lowercase() == "tcp" {
                    args.mode = Some(Mode::Tcp);
                    args.device = device.remote.host.clone().unwrap();
                    if device.remote.slave_id.is_some() {
                        args.slave.clear();
                        args.slave.push(device.remote.slave_id.unwrap());
                    }
                    if device.remote.port.is_some() {
                        args.port =
                            Some(device.remote.port.clone().unwrap().parse::<u16>().unwrap());
                    }
                    if device.remote.timeout_ms.is_some() {
                        args.timeout = Some(Duration::from_secs_f32(
                            device.remote.timeout_ms.unwrap() as f32 / 1000.0,
                        ));
                    }
                } else if device.remote.mode.to_lowercase() == "rtu_in_tcp" {
                    args.mode = Some(Mode::RtuInTcp);
                    args.device = device.remote.host.clone().unwrap();
                    if device.remote.slave_id.is_some() {
                        args.slave.clear();
                        args.slave.push(device.remote.slave_id.unwrap());
                    }
                    if device.remote.port.is_some() {
                        args.port =
                            Some(device.remote.port.clone().unwrap().parse::<u16>().unwrap());
                    }
                    if device.remote.timeout_ms.is_some() {
                        args.timeout = Some(Duration::from_secs_f32(
                            device.remote.timeout_ms.unwrap() as f32 / 1000.0,
                        ));
                    }
                } else {
                    Err(anyhow::anyhow!("Unsupported mode:{}", device.remote.mode))?;
                }
            } else {
                if device.remote.protocol.to_lowercase() == "iec104" {
                    Err(anyhow::anyhow!("iec104 protocol isn't supported yet"))?;
                }
                Err(anyhow::anyhow!(
                    "Unsupported protocol:{}",
                    device.remote.protocol
                ))?;
            }
        }
    }

    Ok(())
}

fn print_args(args: &Args) {
    println!("Protocol configuration: ModBus {:?}", args.mode.unwrap());
    println!("Slave configuration...: address = {:?}", args.slave);
    println!(
        "                      : start reference = {:?}, count = {}",
        args.reference,
        args.count.unwrap()
    );
    if args.mode == Some(Mode::Rtu) {
        println!(
            "Communication.........: {}, {:?}-{:1?}-{}-{:?}
                        t/o {:.2} s, poll rate {} ms",
            args.device.to_string().red(),
            args.baudrate.unwrap(),
            args.databits.unwrap(),
            args.parity.clone().unwrap(),
            args.stopbits.unwrap(),
            args.timeout.unwrap().as_secs_f32(),
            args.poll_rate.unwrap()
        );
    } else if args.mode == Some(Mode::Tcp) || args.mode == Some(Mode::RtuInTcp) {
        println!(
            "Communication.........: {}, port {}, t/o {:.2} s, poll rate {} ms",
            args.device.to_string().red(),
            args.port.unwrap().to_string().red(),
            args.timeout.unwrap().as_secs_f32(),
            args.poll_rate.unwrap()
        );
    } else {
        todo!()
    }
    println!(
        "Data type.............: {:?} {:?}\n",
        args.r#type.clone().unwrap().format,
        args.r#type.clone().unwrap().function
    );
}

fn extract_data(data: &[u16], pos: usize, little_endian: bool) -> u32 {
    if little_endian {
        extract_data_32(data, pos, 2, 3, 0, 1)
    } else {
        extract_data_32(data, pos, 0, 1, 2, 3)
    }
}

fn extract_data_32(data: &[u16], pos: usize, a: usize, b: usize, c: usize, d: usize) -> u32 {
    let data = [data[pos].to_be_bytes(), data[pos + 1].to_be_bytes()].concat();
    u32::from_be_bytes([data[a], data[b], data[c], data[d]])
}

#[cfg(test)]
mod tests {}
