//! USB controller support
//!
//! Current support:
//! - EHCI controllers:
//!    * control transfers (requests): full support
//!    * interrupt transfers: full support
//!    * bulk: unused, no API support
//!    * isochronous transfers: unused, no API support

#![no_std]

extern crate alloc;

use memory::{MappedPages, PhysicalAddress, map_frame_range, create_identity_mapping, PAGE_SIZE, MMIO_FLAGS};
use core::{mem::size_of, num::NonZeroU8, str::from_utf8, any::TypeId, ops::{Deref, DerefMut}};
use alloc::{string::String, vec::Vec};
use volatile::{Volatile, ReadOnly};
use pci::{PciDevice, PciLocation};
use sync_irq::{RwLock, Mutex};
use sleep::{Duration, sleep};
use zerocopy::FromBytes;
use bilge::prelude::*;

mod interfaces;

pub(crate) mod controllers;
pub mod descriptors;
pub mod allocators;
pub mod request;

use descriptors::DescriptorType;
use allocators::{CommonUsbAlloc, AllocSlot, UsbPointer, invalid_ptr_slot};
use request::Request;
use controllers::Controller;
pub use controllers::PciInterface;

static CONTROLLERS: RwLock<Vec<(PciLocation, Mutex<Controller>)>> = RwLock::new(Vec::new());

pub fn init(pci_device: PciInterface) -> Result<(), &'static str> {
    let (location, mut controller) = match pci_device {
        PciInterface::Ehci(dev) => (dev.location, Controller::Ehci(controllers::ehci::EhciController::init(dev)?)),
    };

    controller.probe_ports()?;

    let mut controllers = CONTROLLERS.write();
    controllers.push((location, Mutex::new(controller)));

    Ok(())
}

pub type InterfaceId = usize;
pub type DeviceAddress = u8;
pub type MaxPacketSize = u16;

#[bitsize(16)]
#[derive(DebugBits, Copy, Clone, FromBits, FromBytes)]
pub struct DeviceStatus {
    self_powered: bool,
    remote_wakeup: bool,
    reserved: u14,
}

/// Specific feature ID; must be appropriate to the recipient
pub type FeatureId = u16;

pub type InterfaceIndex = u8;

#[bitsize(8)]
#[derive(DebugBits, Copy, Clone, FromBits, FromBytes)]
pub struct EndpointAddress {
    ep_number: u4,
    reserved: u3,
    // Ignored for control endpoints
    direction: Direction,
}

pub type StringIndex = u8;

pub type DescriptorIndex = u8;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Device,
    Interface(InterfaceIndex),
    Endpoint(EndpointAddress),
}

impl Target {
    fn index(self) -> u16 {
        match self {
            Self::Device => 0u16,
            Self::Interface(index) => index as u16,
            Self::Endpoint(addr) => addr.ep_number().value() as u16,
        }
    }
}

#[bitsize(64)]
#[derive(DebugBits, Copy, Clone, FromBits, FromBytes)]
struct RawRequest {
    pub recipient: RawRequestRecipient,
    pub req_type: RequestType,
    pub direction: Direction,

    pub request_name: u8,
    pub value: u16,
    pub index: u16,
    pub len: u16,
}

#[bitsize(1)]
#[derive(Debug, FromBits)]
pub enum Direction {
    /// Host to function
    Out = 0,
    /// Function to host
    In = 1,
}

#[bitsize(5)]
#[derive(Debug, FromBits)]
enum RawRequestRecipient {
    Device = 0x0,
    Interface = 0x1,
    Endpoint = 0x2,
    Other = 0x3,
    #[fallback]
    Reserved = 0x4,
}

impl From<Target> for RawRequestRecipient {
    fn from(target: Target) -> RawRequestRecipient {
        match target {
            Target::Device => RawRequestRecipient::Device,
            Target::Interface(_) => RawRequestRecipient::Interface,
            Target::Endpoint(_) => RawRequestRecipient::Endpoint,
        }
    }
}

#[bitsize(2)]
#[derive(Debug, FromBits)]
enum RequestType {
    Standard = 0x0,
    Class = 0x1,
    Vendor = 0x2,
    Reserved = 0x3,
}

mod std_req {
    pub const GET_STATUS: u8 = 0x00;
    pub const CLEAR_FEATURE: u8 = 0x01;
    pub const SET_FEATURE: u8 = 0x03;
    pub const SET_ADDRESS: u8 = 0x05;
    pub const GET_DESCRIPTOR: u8 = 0x06;
    pub const SET_DESCRIPTOR: u8 = 0x07;
    pub const GET_CONFIGURATION: u8 = 0x08;
    pub const SET_CONFIGURATION: u8 = 0x09;
    pub const GET_INTERFACE_ALT_SETTING: u8 = 0x0a;
    pub const SET_INTERFACE_ALT_SETTING: u8 = 0x0b;
    pub const _SYNC_FRAME: u8 = 0x0c;
}

mod hid_req {
    pub const GET_REPORT: u8 = 0x01;
    pub const SET_REPORT: u8 = 0x09;
    pub const GET_PROTOCOL: u8 = 0x03;
    pub const SET_PROTOCOL: u8 = 0x0B;
    pub const _GET_IDLE: u8 = 0x02;
    pub const _SET_IDLE: u8 = 0x0A;
}
