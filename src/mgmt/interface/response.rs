use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use bytes::*;
use num_traits::FromPrimitive;

use crate::mgmt::interface::controller::Controller;
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;

pub struct ManagementResponse {
    pub event: ManagementEvent,
    pub controller: Controller,
}

impl ManagementResponse {
    pub fn parse<T: Buf>(mut buf: T) -> Result<Self, ManagementError> {
        let evt_code = buf.get_u16_le();
        let controller = Controller(buf.get_u16_le());
        buf.advance(2); // we already know param length

        Ok(ManagementResponse {
            controller,
            event: match evt_code {
                0x0001 | 0x0002 => {
                    let opcode = buf.get_u16_le();
                    let opcode = FromPrimitive::from_u16(opcode)
                        .ok_or(ManagementError::UnknownOpcode { opcode })?;

                    let status = buf.get_u8();
                    let status = FromPrimitive::from_u8(status)
                        .ok_or(ManagementError::UnknownStatus { status })?;

                    if evt_code == 0x0001 {
                        ManagementEvent::CommandComplete {
                            opcode,
                            status,
                            param: buf.to_bytes(),
                        }
                    } else {
                        ManagementEvent::CommandStatus { opcode, status }
                    }
                }
                0x0003 => ManagementEvent::ControllerError { code: buf.get_u8() },
                0x0004 => ManagementEvent::IndexAdded,
                0x0005 => ManagementEvent::IndexRemoved,
                0x0006 => unimplemented!("ManagementEvent::NewSettings"),
                0x0007 => unimplemented!("ManagementEvent::ClassOfDeviceChanged"),
                0x0008 => {
                    let mut buf = buf.to_bytes();
                    let name = OsString::from_vec(buf.split_to(249).to_vec());
                    let short_name = OsString::from_vec(buf.to_vec());

                    ManagementEvent::LocalNameChanged { name, short_name }
                }
                _ => todo!("throw error instead of panicking"),
            },
        })
    }
}
