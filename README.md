# [dpoll](https://git-inf.skiffenergy.com/xiongdajun/dpoll)

Modbus/IEC104 Client Simulator. It allow you to read and write in Modbus slave registers and IEC104.

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
Modbus/IEC104 Client Simulator

Usage: dpoll [OPTIONS] <DEVICE|HOST|NAME> [WRITEVALUES]...

Arguments:
  <DEVICE|HOST|NAME>
          DEVICE: Serial port when using ModBus RTU protocol.
          HOST: Host name or dotted IP address when using ModBus/TCP or IEC104
          NAME: Name of the device in the configuration file

          DEVICE: COM1, COM2 ... on Windows. /dev/ttyS0, /dev/ttyS1 ...  on Linux. /dev/ser1, /dev/ser2 ...    on QNX
          HOST: 192.168.10.13. 192.168.10.13:502. 192.168.10.13:501, After the IP address, you can specify the port number separated by a colon.
          NAME: The name of the device in the configuration file. The configuration file is specified by the -conf option. for example: dpoll em2_0 -r 1 -c 10 -t 4

  [WRITEVALUES]...
          List of values to be written.

          If none specified (default) dpoll reads data.
          If negative numbers are provided, it will precede the list of
          data to be written by two dashes ('--'). for example : dpoll -t4:i16 /dev/ttyUSB0 -- 123 -1568 8974 -12

Options:
  -m, --mode <MODE>
          mode (tcp, rtu, rtu-in-tcp, iec104)

          [default: tcp]
          [possible values: tcp, rtu, rtu-in-tcp, iec104]

  -r, --reference <REFERENCE>
          Start reference (supported dec/hex/bin three formats)

          [default: 0]

  -a, --slave <SLAVE>
          Slave address (1-255 for rtu, 0-255 for tcp) for reading,

          it is possible to give an address list separated by commas or colons, for example : -a 32,33,34,36:40 read [32,33,34,36,37,38,39,40]

          [default: 1]

  -c, --count <COUNT>
          Number of values to read (1-125)

          [default: 1]

  -t, --type <TYPE>
          which data type should be read

          -t 1          Discrete output (coil) data type (binary 0 or 1)
          -t 2          Discrete input data type (binary 0 or 1)
          -t 3          16-bit output(holding) register data type
          -t 3:i16      16-bit integer data type in output(holding) register table
          -t 3:u16      16-bit unsigned integer data type in output(holding) register table
          -t 3:i32      32-bit integer data type in output(holding) register table
          -t 3:i32abcd  32-bit integer data type in output(holding) register table
          -t 3:i32badc  32-bit integer data type in output(holding) register table
          -t 3:i32cdab  32-bit integer data type in output(holding) register table
          -t 3:i32dcba  32-bit integer data type in output(holding) register table
          -t 3:u32      32-bit unsigned integer data type in output(holding) register table
          -t 3:u32abcd  32-bit unsigned integer data type in output(holding) register table
          -t 3:u32badc  32-bit unsigned integer data type in output(holding) register table
          -t 3:u32cdab  32-bit unsigned integer data type in output(holding) register table
          -t 3:u32dcba  32-bit unsigned integer data type in output(holding) register table
          -t 3:hex16    16-bit output(holding) register data type with hex display
          -t 3:hex32    32-bit output(holding) register data type with hex display
          -t 3:bin16    16-bit output(holding) register data type with bin display
          -t 3:bin32    32-bit output(holding) register data type with bin display
          -t 3:f32      32-bit float data type in output(holding) register table
          -t 3:f32abcd  32-bit float data type in output(holding) register table
          -t 3:f32badc  32-bit float data type in output(holding) register table
          -t 3:f32cdab  32-bit float data type in output(holding) register table
          -t 3:f32dcba  32-bit float data type in output(holding) register table
          -t 4          16-bit input register data type (default)
          -t 4:i16      16-bit integer data type in input register table
          -t 4:u16      16-bit unsigned integer data type in input register table
          -t 4:i32      32-bit integer data type in input register table
          -t 4:u32      32-bit unsigned integer data type in input register table
          -t 4:hex16    16-bit input register data type with hex display
          -t 4:hex32    32-bit input register data type with hex display
          -t 4:bin16    16-bit input register data type with bin display
          -t 4:bin32    32-bit input register data type with bin display
          -t 4:f32      32-bit float data type in input register table
          -t 4:f32abcd  32-bit float data type in input register table
          -t 4:f32badc  32-bit float data type in input register table
          -t 4:f32cdab  32-bit float data type in input register table
          -t 4:f32dcba  32-bit float data type in input register table
          -t siq        IEC104 Single Point Info 单点信息
          -t diq        IEC104 Double Point Info 双点信息
          -t nva        IEC104 Measured Value Normal Info 测量值,规一化值
          -t sva        IEC104 Measured Value Scaled Info 测量值,标度化值
          -t r          IEC104 Measured Value Float Info 测量值,短浮点数
          -t bcr        IEC104 Binary Counter Reading Info 累计量

          [default: 3]

  -L
          Little endian word order for 32-bit integer and float [default = Big endian]

  -1
          Poll only once only, otherwise every poll rate interval

  -l <POLL_RATE>
          Poll rate in ms, ( > 10)

          [default: 1000]

  -o, --timeout <TIMEOUT>
          Time-out in seconds (0.01 - 10.00)

          [default: 1.00]

  -p, --port <PORT>
          TCP port number (502 is default)

          [default: 502]

  -b <BAUDRATE>
          Baudrate (1200-921600)

          [default: 9600]

  -d <DATABITS>
          Databits (7 or 8, 8 for RTU)

          [default: 8]

  -s <STOPBITS>
          Stopbits (1 or 2)

          [default: 1]

  -P <PARITY>
          Parity (none, even, odd)

          [default: none]

  -v, --verbose...
          Increase logging verbosity

  -q, --quiet...
          Decrease logging verbosity

      --conf <CONF>
          The path to the configuration file

          [default: /home/work/deploy/device/conf/device_list.json]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

# Features

- 支持 `IEC104`
- 支持透传 ( `rtu-in-tcp` )
- 支持从 `device_list.json` 读取设备配置, 无需手动输入 `IP` 、 `端口` 、`串口信息` 等
- `host` 输入格式支持 `ip:port` 或 `ip`
- 更多的输出格式，支持 `bin16 bin32 hex16 hex32 i32abcd i32badc i32cdab i32dcba u32abcd u32badc u32cdab u32dcba f32abcd f32badc f32cdab f32dcba`
- 彩色提示/输出

# Break Changes

- `-t 1`: 1 代表 0x01 功能码，`Discrete output (COIL/BITS)`
- `-t 2`: 2 代表 0x02 功能码， `Discrete input (INPUT BITS)`
- `-t 3`: 3 代表 0x03 功能码，不再是 `input register`，而是 `holding register`
- `-t 4`: 4 代表 0x04 功能码，不再是 `holding register`，而是 `input register`

# Deprecated

- `-0`
- `-B`: `-L` 代替 (`Little endian word order for 32-bit integer and float`)

# Example

```bash
dpoll -h
dpoll --help
dpoll 192.168.111.111:2404 -m iec104 -t siq -r 0 -r 3 -r 4
dpoll 192.168.111.111:2404 -m iec104 -t diq -r 1000 3
dpoll 192.168.111.111:2404 -m iec104 -t diq -r 1000 -c 10
dpoll /dev/ttyS0 -t 3:f32 -r 0 -c 10
dpoll /dev/ttyS0 -t 3:bin16 -r 0 -c 10 -L
dpoll 192.168.111.111:502 -t 3:i16 -r 0 1 0x11 0b11
dpoll 192.168.111.111:502 -t 3:i32 -r 0 -c 10
dpoll bms_0 -t 4:hex16 -r 0x00 -c 10 -vv
dpoll pcs_0 -t 4:bin32 -r 0b11 -c 10 -vvv
dpoll em2_0 -t 4:hex32 -r 0 -c 10 -vvvv
```

# TODO

- [ ] 支持 DLT645
- [ ] 支持 `i64/u64/hex64/bin64/f64` 输出格式
- [ ] `-vvvv` 显示IEC104协议接收字节信息
- [x] `-vvvv` 显示MODBUS协议接收字节信息
- [x] 支持 IEC104
