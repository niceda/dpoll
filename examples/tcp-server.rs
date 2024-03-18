// SPDX-FileCopyrightText: Copyright (c) 2017-2024 slowtec GmbH <post@slowtec.de>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # TCP server example
//!
//! This example shows how to start a server and implement basic register
//! read/write operations.

use std::{
    collections::HashMap,
    future,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::net::TcpListener;

use tokio_modbus::{
    prelude::*,
    server::tcp::{accept_tcp_connection, Server},
};

struct ExampleService {
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl tokio_modbus::server::Service for ExampleService {
    type Request = Request<'static>;
    type Future = future::Ready<Result<Response, Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadInputRegisters(addr, cnt) => {
                match register_read(&self.input_registers.lock().unwrap(), addr, cnt) {
                    Ok(values) => future::ready(Ok(Response::ReadInputRegisters(values))),
                    Err(err) => future::ready(Err(err)),
                }
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                match register_read(&self.holding_registers.lock().unwrap(), addr, cnt) {
                    Ok(values) => future::ready(Ok(Response::ReadHoldingRegisters(values))),
                    Err(err) => future::ready(Err(err)),
                }
            }
            Request::WriteMultipleRegisters(addr, values) => {
                println!("SERVER: WriteMultipleRegisters: {addr} {values:?}");
                match register_write(&mut self.holding_registers.lock().unwrap(), addr, &values) {
                    Ok(_) => future::ready(Ok(Response::WriteMultipleRegisters(
                        addr,
                        values.len() as u16,
                    ))),
                    Err(err) => future::ready(Err(err)),
                }
            }
            Request::WriteSingleRegister(addr, value) => {
                println!("SERVER: WriteSingleRegister: {addr} {value}");
                match register_write(
                    &mut self.holding_registers.lock().unwrap(),
                    addr,
                    std::slice::from_ref(&value),
                ) {
                    Ok(_) => future::ready(Ok(Response::WriteSingleRegister(addr, value))),
                    Err(err) => future::ready(Err(err)),
                }
            }
            _ => {
                println!("SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}");
                future::ready(Err(Exception::IllegalFunction))
            }
        }
    }
}

impl ExampleService {
    fn new() -> Self {
        // Insert some test data as register values.
        let mut input_registers = HashMap::new();
        input_registers.insert(0, 1234);
        input_registers.insert(1, 5678);

        let f32 = 1.1_f32;
        let data_f32 = f32.to_be_bytes();
        input_registers.insert(2, u16::from_be_bytes(data_f32[0..2].try_into().unwrap()));
        input_registers.insert(3, u16::from_be_bytes(data_f32[2..4].try_into().unwrap()));

        let f32abcd = 1.3_f32;
        let data_f32abcd = f32abcd.to_be_bytes();
        input_registers.insert(
            4,
            u16::from_be_bytes(data_f32abcd[0..2].try_into().unwrap()),
        );
        input_registers.insert(
            5,
            u16::from_be_bytes(data_f32abcd[2..4].try_into().unwrap()),
        );

        let f32badc = 1.3_f32;
        let data_f32badc = f32badc.to_be_bytes();
        input_registers.insert(6, u16::from_be_bytes([data_f32badc[1], data_f32badc[0]]));
        input_registers.insert(7, u16::from_be_bytes([data_f32badc[3], data_f32badc[2]]));

        let f32dcba = 1.4_f32;
        let data_f32dcba = f32dcba.to_be_bytes();
        input_registers.insert(8, u16::from_be_bytes([data_f32dcba[3], data_f32dcba[2]]));
        input_registers.insert(9, u16::from_be_bytes([data_f32dcba[1], data_f32dcba[0]]));

        let f32cdab = 1.5_f32;
        let data_f32cdab = f32cdab.to_be_bytes();
        input_registers.insert(10, u16::from_be_bytes([data_f32cdab[2], data_f32cdab[3]]));
        input_registers.insert(11, u16::from_be_bytes([data_f32cdab[0], data_f32cdab[1]]));

        let mut holding_registers = HashMap::new();
        let u32 = 40_u32;
        let data_u32 = u32.to_be_bytes();
        holding_registers.insert(0, u16::from_be_bytes(data_u32[0..2].try_into().unwrap()));
        holding_registers.insert(1, u16::from_be_bytes(data_u32[2..4].try_into().unwrap()));

        let f32 = 1.1_f32;
        let data_f32 = f32.to_be_bytes();
        holding_registers.insert(2, u16::from_be_bytes(data_f32[0..2].try_into().unwrap()));
        holding_registers.insert(3, u16::from_be_bytes(data_f32[2..4].try_into().unwrap()));

        let f32abcd = 1.2_f32;
        let data_f32abcd = f32abcd.to_be_bytes();
        holding_registers.insert(
            4,
            u16::from_be_bytes(data_f32abcd[0..2].try_into().unwrap()),
        );
        holding_registers.insert(
            5,
            u16::from_be_bytes(data_f32abcd[2..4].try_into().unwrap()),
        );

        let f32badc = 1.3_f32;
        let data_f32badc = f32badc.to_be_bytes();
        holding_registers.insert(6, u16::from_be_bytes([data_f32badc[1], data_f32badc[0]]));
        holding_registers.insert(7, u16::from_be_bytes([data_f32badc[3], data_f32badc[2]]));

        let f32dcba = 1.4_f32;
        let data_f32dcba = f32dcba.to_be_bytes();
        holding_registers.insert(8, u16::from_be_bytes([data_f32dcba[3], data_f32dcba[2]]));
        holding_registers.insert(9, u16::from_be_bytes([data_f32dcba[1], data_f32dcba[0]]));

        let f32cdab = 1.5_f32;
        let data_f32cdab = f32cdab.to_be_bytes();
        holding_registers.insert(10, u16::from_be_bytes([data_f32cdab[2], data_f32cdab[3]]));
        holding_registers.insert(11, u16::from_be_bytes([data_f32cdab[0], data_f32cdab[1]]));

        Self {
            input_registers: Arc::new(Mutex::new(input_registers)),
            holding_registers: Arc::new(Mutex::new(holding_registers)),
        }
    }
}

/// Helper function implementing reading registers from a HashMap.
fn register_read(
    registers: &HashMap<u16, u16>,
    addr: u16,
    cnt: u16,
) -> Result<Vec<u16>, Exception> {
    let mut response_values = vec![0; cnt.into()];
    for i in 0..cnt {
        let reg_addr = addr + i;
        if let Some(r) = registers.get(&reg_addr) {
            response_values[i as usize] = *r;
        } else {
            // TODO: Return a Modbus Exception response `IllegalDataAddress` https://github.com/slowtec/tokio-modbus/issues/165
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(Exception::IllegalDataAddress);
        }
    }

    Ok(response_values)
}

/// Write a holding register. Used by both the write single register
/// and write multiple registers requests.
fn register_write(
    registers: &mut HashMap<u16, u16>,
    addr: u16,
    values: &[u16],
) -> Result<(), Exception> {
    for (i, value) in values.iter().enumerate() {
        let reg_addr = addr + i as u16;
        if let Some(r) = registers.get_mut(&reg_addr) {
            println!("SERVER: Writing holding register {reg_addr} = {value}");
            *r = *value;
        } else {
            // TODO: Return a Modbus Exception response `IllegalDataAddress` https://github.com/slowtec/tokio-modbus/issues/165
            println!("SERVER: Exception::IllegalDataAddress");
            return Err(Exception::IllegalDataAddress);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_addr = "127.0.0.1:5502".parse().unwrap();

    // tokio::select! {
    //     _ = server_context(socket_addr) => unreachable!(),
    //     _ = client_context(socket_addr) => println!("Exiting"),
    // }

    server_context(socket_addr).await?;

    Ok(())
}

async fn server_context(socket_addr: SocketAddr) -> anyhow::Result<()> {
    println!("Starting up server on {socket_addr}");
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    let new_service = |_socket_addr| Ok(Some(ExampleService::new()));
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    let on_process_error = |err| {
        eprintln!("{err}");
    };
    server.serve(&on_connected, on_process_error).await?;
    Ok(())
}

// async fn client_context(socket_addr: SocketAddr) {
//     tokio::join!(
//         async {
//             // Give the server some time for starting up
//             tokio::time::sleep(Duration::from_secs(1)).await;
//
//             println!("CLIENT: Connecting client...");
//             let mut ctx = tcp::connect(socket_addr).await.unwrap();
//
//             println!("CLIENT: Reading 2 input registers...");
//             let response = ctx.read_input_registers(0x00, 2).await.unwrap();
//             println!("CLIENT: The result is '{response:?}'");
//             assert_eq!(response, [1234, 5678]);
//
//             println!("CLIENT: Writing 2 holding registers...");
//             ctx.write_multiple_registers(0x01, &[7777, 8888])
//                 .await
//                 .unwrap();
//
//             // Read back a block including the two registers we wrote.
//             println!("CLIENT: Reading 4 holding registers...");
//             let response = ctx.read_holding_registers(0x00, 4).await.unwrap();
//             println!("CLIENT: The result is '{response:?}'");
//             assert_eq!(response, [10, 7777, 8888, 40]);
//
//             // Now we try to read with an invalid register address.
//             // This should return a Modbus exception response with the code
//             // IllegalDataAddress.
//             println!("CLIENT: Reading nonexisting holding register address... (should return IllegalDataAddress)");
//             let response = ctx.read_holding_registers(0x100, 1).await;
//             println!("CLIENT: The result is '{response:?}'");
//             assert!(response.is_err());
//             // TODO: How can Modbus client identify Modbus exception responses? E.g. here we expect IllegalDataAddress
//             // Question here: https://github.com/slowtec/tokio-modbus/issues/169
//
//             println!("CLIENT: Done.")
//         },
//         tokio::time::sleep(Duration::from_secs(5))
//     );
// }
