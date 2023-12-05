use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use embedded_can::Frame;
use embedded_can::nb::Can;
use crate::node::Node;
use crate::util::genf_and_padding;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EmergencyErrorCode {
    PdoNotProcessed,
}

impl EmergencyErrorCode {
    pub fn code(&self) -> u16 {
        match *self {
            EmergencyErrorCode::PdoNotProcessed => 0x8210,
        }
    }

    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            0x8210 => Some(EmergencyErrorCode::PdoNotProcessed),
            _ => None,
        }
    }
}

pub enum ErrorRegister {
    GenericError,
    Current,
    Voltage,
    Temperature,
    CommunicationError,  // Overrun / Error state
    DeviceProfileSpecific,
    Reserved,
    ManufacturerSpecific,
}

impl ErrorRegister {
    pub fn code(&self) -> u8 {
        match *self {
            ErrorRegister::GenericError => 0,
            ErrorRegister::Current => 1,
            ErrorRegister::Voltage => 2,
            ErrorRegister::Temperature => 3,
            ErrorRegister::CommunicationError => 4,
            ErrorRegister::DeviceProfileSpecific => 5,
            ErrorRegister::Reserved => 6,
            ErrorRegister::ManufacturerSpecific => 7,
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(ErrorRegister::GenericError),
            1 => Some(ErrorRegister::Current),
            2 => Some(ErrorRegister::Voltage),
            3 => Some(ErrorRegister::Temperature),
            4 => Some(ErrorRegister::CommunicationError),
            5 => Some(ErrorRegister::DeviceProfileSpecific),
            6 => Some(ErrorRegister::Reserved),
            7 => Some(ErrorRegister::ManufacturerSpecific),
            _ => None,
        }
    }
}

impl<CAN: Can> Node<CAN> where CAN::Frame: Frame + Debug {
    pub fn trigger_emergency(&mut self, eec: EmergencyErrorCode, er: ErrorRegister, data: &[u8]) {
        let eec_arr = eec.code().to_le_bytes();
        let (eecl, eech) = (eec_arr[0], eec_arr[1]);
        let erc = er.code();
        let mut v: Vec<u8> = vec![eecl, eech, erc];
        v.extend_from_slice(data);
        let frame = genf_and_padding(0x080 | self.node_id as u16, v.as_slice());
        self.can_network.transmit(&frame).expect("Errors in transmit packet");

        let tmp_count = self.error_count + 1;
        self.object_directory.set_value(0x1003, 0x0, &[tmp_count], true).expect("TODO: panic message");
        self.object_directory.set_value(0x1003, tmp_count, &[eecl, eech, 0, 0], true).expect("TODO");
        self.object_directory.set_value(0x1001, 0x0, &[erc], true).expect("TODO");
        self.error_count = tmp_count;

        let mut reset_v: Vec<u8> = vec![0, 0, 0];
        reset_v.extend_from_slice(data);
        let reset_frame = genf_and_padding(0x080 | self.node_id as u16, reset_v.as_slice());
        self.can_network.transmit(&reset_frame).expect("Errors in transmit reset packet");
    }
}