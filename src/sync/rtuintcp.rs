use std::{io::Result, net::SocketAddr, time::Duration};

use super::{block_on_with_timeout, Context};

use tokio::net::TcpStream;
use tokio_modbus::{client::rtu, Slave};

/// Connect to no particular Modbus slave device for sending
/// broadcast messages.
pub fn connect(socket_addr: SocketAddr) -> Result<Context> {
    connect_slave(socket_addr, Slave::broadcast())
}

/// Connect to no particular Modbus slave device for sending
/// broadcast messages with a timeout.
pub fn connect_with_timeout(socket_addr: SocketAddr, timeout: Option<Duration>) -> Result<Context> {
    connect_slave_with_timeout(socket_addr, Slave::broadcast(), timeout)
}

/// Connect to any kind of Modbus slave device.
pub fn connect_slave(socket_addr: SocketAddr, slave: Slave) -> Result<Context> {
    connect_slave_with_timeout(socket_addr, slave, None)
}

/// Connect to any kind of Modbus slave device with a timeout.
pub fn connect_slave_with_timeout(
    socket_addr: SocketAddr,
    slave: Slave,
    timeout: Option<Duration>,
) -> Result<Context> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    // SerialStream::open requires a runtime at least on cfg(unix).
    let mock_serial = block_on_with_timeout(&runtime, timeout, async {
        TcpStream::connect(socket_addr).await
    })?;
    let async_ctx = rtu::attach_slave(mock_serial, slave);
    let sync_ctx = Context {
        runtime,
        async_ctx,
        timeout,
    };
    Ok(sync_ctx)
}
