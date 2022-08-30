use core::{convert::TryFrom, fmt, str::FromStr};
use irq_safety::MutexIrqSafe;
use pl011_qemu::PL011;
use pl011_qemu::UART1;
use pl011_qemu::UART2;
use pl011_qemu::UART3;
use pl011_qemu::UART4;
use embedded_hal::serial::Read;
use embedded_hal::serial::Write;

/// COM serial ports enumeration.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum SerialPortAddress {
    COM1,
    COM2,
    COM3,
    COM4,
}

impl SerialPortAddress {
    /// Returns a reference to the static instance of this serial port.
    fn to_static_port(&self) -> &'static MutexIrqSafe<TriState<SerialPort>> {
        match self {
            SerialPortAddress::COM1 => &COM1_SERIAL_PORT,
            SerialPortAddress::COM2 => &COM2_SERIAL_PORT,
            SerialPortAddress::COM3 => &COM3_SERIAL_PORT,
            SerialPortAddress::COM4 => &COM4_SERIAL_PORT,
        }
    }
}

impl TryFrom<&str> for SerialPortAddress {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            v if v.eq_ignore_ascii_case("COM1") => Ok(Self::COM1),
            v if v.eq_ignore_ascii_case("COM2") => Ok(Self::COM2),
            v if v.eq_ignore_ascii_case("COM3") => Ok(Self::COM3),
            v if v.eq_ignore_ascii_case("COM4") => Ok(Self::COM4),
            _ => Err(()),
        }
    }
}

impl FromStr for SerialPortAddress {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<u16> for SerialPortAddress {
    type Error = ();
    fn try_from(port: u16) -> Result<Self, Self::Error> {
        match port {
            p if p == Self::COM1 as u16 => Ok(Self::COM1),
            p if p == Self::COM2 as u16 => Ok(Self::COM2),
            p if p == Self::COM3 as u16 => Ok(Self::COM3),
            p if p == Self::COM4 as u16 => Ok(Self::COM4),
            _ => Err(()),
        }
    }
}

/// This type is used to ensure that an object of type `T` is only initialized once,
/// but still allows for a caller to take ownership of the object `T`. 
enum TriState<T> {
    Uninited,
    Inited(T),
    Taken,
}

impl<T> TriState<T> {
    fn take(&mut self) -> Option<T> {
        if let Self::Inited(_) = self {
            if let Self::Inited(v) = core::mem::replace(self, Self::Taken) {
                return Some(v);
            }
        }
        None
    }
}

// Serial ports cannot be reliably probed (discovered dynamically), thus,
// we ensure they are exposed safely as singletons through the below static instances.
static COM1_SERIAL_PORT: MutexIrqSafe<TriState<SerialPort>> = MutexIrqSafe::new(TriState::Uninited);
static COM2_SERIAL_PORT: MutexIrqSafe<TriState<SerialPort>> = MutexIrqSafe::new(TriState::Uninited);
static COM3_SERIAL_PORT: MutexIrqSafe<TriState<SerialPort>> = MutexIrqSafe::new(TriState::Uninited);
static COM4_SERIAL_PORT: MutexIrqSafe<TriState<SerialPort>> = MutexIrqSafe::new(TriState::Uninited);


/// Takes ownership of the [`SerialPort`] specified by the given [`SerialPortAddress`].
///
/// This function initializes the given serial port if it has not yet been initialized.
/// If the serial port has already been initialized and taken by another crate,
/// this returns `None`.
///
/// The returned [`SerialPort`] will be restored to this crate upon being dropped.
pub fn take_serial_port(
    addr: SerialPortAddress
) -> Option<SerialPort> {
    let sp = addr.to_static_port();
    let mut locked = sp.lock();
    if let TriState::Uninited = &*locked {
        *locked = TriState::Inited(match addr {
            SerialPortAddress::COM1 => SerialPort::new(addr, Uart::Uart1(UART1::take().unwrap())),
            SerialPortAddress::COM2 => SerialPort::new(addr, Uart::Uart2(UART2::take().unwrap())),
            SerialPortAddress::COM3 => SerialPort::new(addr, Uart::Uart3(UART3::take().unwrap())),
            SerialPortAddress::COM4 => SerialPort::new(addr, Uart::Uart4(UART4::take().unwrap())),
        });
    }
    locked.take()
}

pub(crate) enum Uart {
    Uart1(UART1),
    Uart2(UART2),
    Uart3(UART3),
    Uart4(UART4),
}

/// A serial port and its various data and control registers.
pub enum SerialPort {
    Uart1(SerialPortAddress, PL011<UART1>),
    Uart2(SerialPortAddress, PL011<UART2>),
    Uart3(SerialPortAddress, PL011<UART3>),
    Uart4(SerialPortAddress, PL011<UART4>),
    Dropped,
}

impl Drop for SerialPort {
    fn drop(&mut self) {
        let mut sp_locked = match self {
            Self::Uart1(addr, _) => addr,
            Self::Uart2(addr, _) => addr,
            Self::Uart3(addr, _) => addr,
            Self::Uart4(addr, _) => addr,
            _ => unreachable!()
        }.to_static_port().lock();
        if let TriState::Taken = &*sp_locked {
            let dummy = SerialPort::Dropped;
            let dropped = core::mem::replace(self, dummy);
            *sp_locked = TriState::Inited(dropped);
        }
    }
}

impl SerialPort {
    /// Creates and returns a new serial port structure, 
    /// and initializes that port using standard configuration parameters.
    pub(crate) fn new(addr: SerialPortAddress, uart: Uart) -> Self {
        match uart {
            Uart::Uart1(uart1) => Self::Uart1(addr, PL011::new(uart1)),
            Uart::Uart2(uart2) => Self::Uart2(addr, PL011::new(uart2)),
            Uart::Uart3(uart3) => Self::Uart3(addr, PL011::new(uart3)),
            Uart::Uart4(uart4) => Self::Uart4(addr, PL011::new(uart4)),
        }
    }

    /// Enable or disable interrupts on this serial port for various events.
    ///
    /// Panics on aarch64.
    pub fn enable_interrupt(&mut self, _event: SerialPortInterruptEvent, _enable: bool) {
        panic!("enable_interrupt: aarch64 builds don't support them yet");
    }

    /// Write the given string to the serial port, blocking until data can be transmitted.
    ///
    /// # Special characters
    /// Because this function writes strings, it will transmit a carriage return `'\r'`
    /// after transmitting a line feed (new line) `'\n'` to ensure a proper new line.
    pub fn out_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.out_byte(byte);
            if byte == b'\n' {
                self.out_byte(b'\r');
            } else if byte == b'\r' {
                self.out_byte(b'\n');
            }
        }
    }

    /// Write the given byte to the serial port, blocking until data can be transmitted.
    ///
    /// This writes the byte directly with no special cases, e.g., new lines.
    pub fn out_byte(&mut self, byte: u8) {
        self.out_bytes(&[byte]);
    }

    /// Write the given bytes to the serial port, blocking until data can be transmitted.
    ///
    /// This writes the bytes directly with no special cases, e.g., new lines.
    pub fn out_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            match self {
                Self::Uart1(_, pl011) => pl011.write(*byte),
                Self::Uart2(_, pl011) => pl011.write(*byte),
                Self::Uart3(_, pl011) => pl011.write(*byte),
                Self::Uart4(_, pl011) => pl011.write(*byte),
                _ => unreachable!()
            }.unwrap();
        };
    }

    /// Read one byte from the serial port, blocking until data is available.
    pub fn in_byte(&mut self) -> u8 {
        while !self.data_available() { }
        match self {
            Self::Uart1(_, pl011) => pl011.read(),
            Self::Uart2(_, pl011) => pl011.read(),
            Self::Uart3(_, pl011) => pl011.read(),
            Self::Uart4(_, pl011) => pl011.read(),
            _ => unreachable!()
        }.unwrap()
    }

    /// Reads multiple bytes from the serial port into the given `buffer`, non-blocking.
    ///
    /// The buffer will be filled with as many bytes as are available in the serial port.
    /// Once data is no longer available to be read, the read operation will stop. 
    ///
    /// If no data is immediately available on the serial port, this will read nothing and return `0`.
    ///
    /// Returns the number of bytes read into the given `buffer`.
    pub fn in_bytes(&mut self, buffer: &mut [u8]) -> usize {
        let mut bytes_read = 0;
        for byte in buffer {
            if !self.data_available() {
                break;
            }
            *byte = self.in_byte();
            bytes_read += 1;
        }
        bytes_read
    }

    /// Returns `true` if the serial port is ready to transmit a byte.
    #[inline(always)]
    pub fn ready_to_transmit(&self) -> bool {
        match self {
            Self::Uart1(_, pl011) => pl011.is_writeable(),
            Self::Uart2(_, pl011) => pl011.is_writeable(),
            Self::Uart3(_, pl011) => pl011.is_writeable(),
            Self::Uart4(_, pl011) => pl011.is_writeable(),
            _ => unreachable!()
        }
    }

    /// Returns `true` if the serial port has data available to read.
    #[inline(always)]
    pub fn data_available(&self) -> bool {
        match self {
            Self::Uart1(_, pl011) => pl011.has_incoming_data(),
            Self::Uart2(_, pl011) => pl011.has_incoming_data(),
            Self::Uart3(_, pl011) => pl011.has_incoming_data(),
            Self::Uart4(_, pl011) => pl011.has_incoming_data(),
            _ => unreachable!()
        }
    }

    pub fn base_port_address(&self) -> u16 {
        panic!("unsupported on aarch64")
    }

}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.out_str(s); 
        Ok(())
    }
}

/// The types of events that can trigger an interrupt on a serial port.
#[derive(Debug)]
#[repr(u8)]
pub enum SerialPortInterruptEvent {
    DataReceived     = 1 << 0,
    TransmitterEmpty = 1 << 1,
    ErrorOrBreak     = 1 << 2,
    StatusChange     = 1 << 3,
}
