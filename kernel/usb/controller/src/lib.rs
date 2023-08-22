//! USB controller support

#![no_std]

extern crate alloc;

use pci::PciDevice;
use memory::{MappedPages, PhysicalAddress, map_frame_range, create_identity_mapping, PAGE_SIZE, MMIO_FLAGS};
use zerocopy::FromBytes;
use volatile::{Volatile, ReadOnly};
use bilge::prelude::*;
use sleep::{Duration, sleep};
use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;

mod ehci;

pub enum Standard<T> {
    Ehci(T),
}

pub fn init(pci_device: Standard<&PciDevice>) -> Result<(), &'static str> {
    match pci_device {
        Standard::Ehci(dev) => ehci::init(dev),
    }
}

/*

#[derive(Debug, Default, FromBytes)]
pub struct Allocator<const N: usize, T: FromBytes> {
    slots: [T; N],
    occupied: [bool; N],
}

impl<const N: usize, T: FromBytes> Allocator<N, T> {
    pub fn init(&mut self) {
        self.occupied.fill(false);
    }

    pub fn alloc(&mut self, value: T) -> Result<(usize, u32), &'static str> {
        for i in 0..N {
            if !self.occupied[i] {
                self.occupied[i] = true;
                let mut_ref = &mut self.slots[i];
                *mut_ref = value;
                let addr = mut_ref as *const Request as usize;

                return Ok((i, addr as u32))
            }
        }

        Err("Allocator: Out of slots")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index)
    }

    pub fn get_mut(&self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index)
    }
}

*/

#[derive(Debug, Default, FromBytes)]
#[repr(C)]
pub struct DeviceDescriptor {
    pub len: u8,
    pub device_type: u8,
    pub usb_version: u16,
    pub class: u8,
    pub sub_class: u8,
    pub protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub vendor_str: u8,
    pub product_str: u8,
    pub serial_str: u8,
    pub conf_count: u8,
}

#[bitsize(64)]
#[derive(DebugBits, Copy, Clone, FromBits, FromBytes)]
struct Request {
    pub recipient: RequestRecipient,
    pub req_type: RequestType,
    pub direction: Direction,

    pub request_name: RequestName,
    pub value: u16,
    pub index: u16,
    pub len: u16,
}

#[bitsize(1)]
#[derive(Debug, FromBits)]
enum Direction {
    /// Host to function
    Out = 0,
    /// Function to host
    In = 1,
}

#[bitsize(5)]
#[derive(Debug, FromBits)]
enum RequestRecipient {
    Device = 0x0,
    Interface = 0x1,
    Endpoint = 0x2,
    Other = 0x3,
    #[fallback]
    Reserved = 0x4,
}

#[bitsize(2)]
#[derive(Debug, FromBits)]
enum RequestType {
    Standard = 0x0,
    Class = 0x1,
    Vendor = 0x2,
    Reserved = 0x3,
}

#[bitsize(8)]
#[derive(Debug, FromBits)]
enum RequestName {
    GetStatus = 0x00,
    ClearFeature = 0x01,
    SetFeature = 0x03,
    SetAddress = 0x05,
    GetDescriptor = 0x06,
    SetDescriptor = 0x07,
    GetConfiguration = 0x08,
    SetConfiguration = 0x09,
    GetIntf = 0x0a,
    SetIntf = 0x0b,
    SyncFrame = 0x0c,

    #[fallback]
    Reserved = 0xff,
}
