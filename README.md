# [dpoll](https://git-inf.skiffenergy.com/xiongdajun/dpoll)

Modbus/IEC104 Master Simulator. It allows to read and write in Modbus slave registers and IEC104.

Thanks to [mbpoll](https://github.com/epsilonrt/mbpoll)

# Install/Update

```bash
wget release.skiffenergy.com/pkg/dpoll.gz &&
gunzip -c dpoll.gz > /usr/sbin/dpoll &&
chmod +x /usr/sbin/dpoll &&
rm ./dpoll.gz
```

# Usage

```
Modbus Master Simulator

Usage: dpoll [OPTIONS] <DEVICE|HOST|NAME> [WRITEVALUES]...

Arguments:
  <DEVICE|HOST|NAME>
          DEVICE: Serial port when using ModBus RTU protocol.
          HOST: Host name or dotted IP address when using ModBus/TCP
          NAME: Name of the device in the configuration file
  [WRITEVALUES]...
          List of values to be written.

Options:
  -m, --mode <MODE>            mode (tcp, rtu, rtu-in-tcp, iec104) iec104 is not supported yet [default: tcp] [possible values: tcp, rtu, rtu-in-tcp, iec104]
  -r, --reference <REFERENCE>  Start reference (supported dec/hex/bin three formats) [default: 0]
  -a, --slave <SLAVE>          Slave address (1-255 for rtu, 0-255 for tcp) for reading, [default: 1]
  -c, --count <COUNT>          Number of values to read (1-125) [default: 1]
  -t, --type <TYPE>            which data type should be read [default: 3]
  -L                           Little endian word order for 32-bit integer and float [default = Big endian]
  -1                           Poll only once only, otherwise every poll rate interval
  -l <POLL_RATE>               Poll rate in ms, ( > 10) [default: 1000]
  -o, --timeout <TIMEOUT>      Time-out in seconds (0.01 - 10.00) [default: 1.00]
  -p, --port <PORT>            TCP port number (502 is default) [default: 502]
  -b <BAUDRATE>                Baudrate (1200-921600) [default: 9600]
  -d <DATABITS>                Databits (7 or 8, 8 for RTU) [default: 8]
  -s <STOPBITS>                Stopbits (1 or 2) [default: 1]
  -P <PARITY>                  Parity (none, even, odd) [default: none]
  -v, --verbose...             Increase logging verbosity
  -q, --quiet...               Decrease logging verbosity
      --conf <CONF>            The path to the configuration file [default: /home/work/deploy/device/conf/device_list.json]
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

# Features

- 支持透传 ( `rtu-in-tcp` )
- 支持从 `device_list.json` 读取设备配置, 无需手动输入 `IP` 、 `端口` 、`串口信息` 等
- `host` 输入格式支持 `ip:port` 或 `ip`
- 更多的输出格式，支持 `bin16 bin32 hex16 hex32 i32abcd i32badc i32cdab i32dcba u32abcd u32badc u32cdab u32dcba f32abcd f32badc f32cdab f32dcba`
- 彩色提示/输出

# Break Changes

- `-t 3`: 3 代表 0x03 功能码，不再是 `input register`，而是 `holding register`
- `-t 4`: 4 代表 0x04 功能码，不再是 `holding register`，而是 `input register`

# Deprecated

- `-0`
- `-B`: `-L` 代替 (`Little endian word order for 32-bit integer and float`)

# Example

```bash
dpoll /dev/ttyS0 -t 3:f32 -r 0 -c 10
dpoll /dev/ttyS0 -t 3:bin16 -r 0 -c 10 -L
dpoll 192.168.111.111:502 -t 3:i16 -r 0 1 0x11 0b11
dpoll 192.168.111.111:502 -t 3:i32 -r 0 -c 10
dpoll bms_0 -t 4:hex16 -r 0x00 -c 10 -vv
dpoll pcs_0 -t 4:bin32 -r 0b11 -c 10 -vvv
dpoll em2_0 -t 4:hex32 -r 0 -c 10 -vvvv
```

# TODO

- [x] `-vvvv` 显示协议接收字节信息
- [ ] 支持 IEC104
- [ ] 支持 DLT645
- [ ] 支持 `i64/u64/hex64/bin64/f64`
