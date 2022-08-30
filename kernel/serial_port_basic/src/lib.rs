//! Support for basic serial port access, including initialization, transmit, and receive.
//!
//! This is a near-standalone crate with very minimal dependencies and a basic feature set
//! intended for use during early Theseus boot up and initialization.
//! For a more featureful serial port driver, use the `serial_port` crate.
//!
//! # Notes
//! Some serial port drivers use special cases for transmitting some byte values,
//! specifically `0x08` and `0x7F`, which are ASCII "backspace" and "delete", respectively.
//! They do so by writing them as three distinct values (with proper busy waiting in between):
//! 1. `0x08`
//! 2. `0x20` (an ascii space character)
//! 3. `0x08` again. 
//!
//! This isn't necessarily a bad idea, as it "clears out" whatever character was there before,
//! presumably to prevent rendering/display issues for a deleted character. 
//! But, this isn't required, and I personally believe it should be handled by a higher layer,
//! such as a shell or TTY program. 
//! We don't do anything like that here, in case a user of this crate wants to send binary data
//! across the serial port, rather than "smartly-interpreted" ASCII characters.
//!
//! # Resources
//! * <https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming>
//! * <https://tldp.org/HOWTO/Modem-HOWTO-4.html>
//! * <https://wiki.osdev.org/Serial_Ports>
//! * <https://www.sci.muni.cz/docs/pc/serport.txt>

#![no_std]

extern crate spin;
extern crate irq_safety;

#[cfg(target_arch = "x86_64")]
extern crate port_io;
#[cfg(target_arch = "aarch64")]
extern crate pl011_qemu;
#[cfg(target_arch = "aarch64")]
extern crate embedded_hal;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

