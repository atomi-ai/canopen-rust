use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;

use embedded_can::Frame;
use embedded_can::nb::Can;

use crate::constant::{COB_FUNC_SYNC, EMCY_PDO_NOT_PROCESSED, REG_ERROR, REG_PRE_DEFINED_ERROR};
use crate::error::ErrorCode;
use crate::node::Node;
use crate::util::create_frame_with_padding;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EmergencyErrorCode {
    PdoNotProcessed,
}

impl EmergencyErrorCode {
    pub(crate) fn code(&self) -> u16 {
        match *self {
            EmergencyErrorCode::PdoNotProcessed => EMCY_PDO_NOT_PROCESSED,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_code(code: u16) -> Option<Self> {
        match code {
            EMCY_PDO_NOT_PROCESSED => Some(EmergencyErrorCode::PdoNotProcessed),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorRegister {
    GenericError,
    Current,
    Voltage,
    Temperature,
    CommunicationError,
    // Overrun / Error state
    DeviceProfileSpecific,
    Reserved,
    ManufacturerSpecific,
}

impl ErrorRegister {
    pub(crate) fn code(&self) -> u8 {
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

    #[allow(dead_code)]
    pub(crate) fn from_code(code: u8) -> Option<Self> {
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
    pub(crate) fn trigger_emergency(&mut self, eec: EmergencyErrorCode, er: ErrorRegister, data: &[u8])
                                    -> Result<(), ErrorCode> {
        let eec_arr = eec.code().to_le_bytes();
        let (eecl, eech) = (eec_arr[0], eec_arr[1]);
        let erc = er.code();
        let mut v: Vec<u8> = vec![eecl, eech, erc];
        v.extend_from_slice(data);
        let frame = create_frame_with_padding(COB_FUNC_SYNC | self.node_id as u16, v.as_slice())?;
        self.transmit(&frame);

        let tmp_count = self.error_count + 1;
        self.object_directory.set_value(REG_PRE_DEFINED_ERROR, 0x0, &[tmp_count], true)?;
        self.object_directory.set_value(REG_PRE_DEFINED_ERROR, tmp_count, &[eecl, eech, 0, 0], true)?;
        self.object_directory.set_value(REG_ERROR, 0x0, &[erc], true)?;
        self.error_count = tmp_count;

        let mut reset_v: Vec<u8> = vec![0, 0, 0];
        reset_v.extend_from_slice(data);
        let reset_frame = create_frame_with_padding(COB_FUNC_SYNC | self.node_id as u16, reset_v.as_slice())?;
        self.transmit(&reset_frame);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_emergency_error_code() {
        assert_eq!(EmergencyErrorCode::PdoNotProcessed.code(), EMCY_PDO_NOT_PROCESSED);

        assert_eq!(EmergencyErrorCode::from_code(EMCY_PDO_NOT_PROCESSED), Some(EmergencyErrorCode::PdoNotProcessed));
        assert_eq!(EmergencyErrorCode::from_code(0xFFFF), None);
    }

    #[test]
    fn test_error_register() {
        assert_eq!(ErrorRegister::GenericError.code(), 0);
        assert_eq!(ErrorRegister::Current.code(), 1);
        assert_eq!(ErrorRegister::Voltage.code(), 2);
        assert_eq!(ErrorRegister::Temperature.code(), 3);
        assert_eq!(ErrorRegister::CommunicationError.code(), 4);
        assert_eq!(ErrorRegister::DeviceProfileSpecific.code(), 5);
        assert_eq!(ErrorRegister::Reserved.code(), 6);
        assert_eq!(ErrorRegister::ManufacturerSpecific.code(), 7);

        assert_eq!(ErrorRegister::from_code(0), Some(ErrorRegister::GenericError));
        assert_eq!(ErrorRegister::from_code(1), Some(ErrorRegister::Current));
        assert_eq!(ErrorRegister::from_code(2), Some(ErrorRegister::Voltage));
        assert_eq!(ErrorRegister::from_code(3), Some(ErrorRegister::Temperature));
        assert_eq!(ErrorRegister::from_code(4), Some(ErrorRegister::CommunicationError));
        assert_eq!(ErrorRegister::from_code(5), Some(ErrorRegister::DeviceProfileSpecific));
        assert_eq!(ErrorRegister::from_code(6), Some(ErrorRegister::Reserved));
        assert_eq!(ErrorRegister::from_code(7), Some(ErrorRegister::ManufacturerSpecific));
        assert_eq!(ErrorRegister::from_code(8), None);
    }

    #[test]
    fn test_error_register_debug() {
        let error = ErrorRegister::GenericError;
        assert_eq!(format!("{:?}", error), "GenericError");
    }
    #[test]
    fn test_error_register_copy_clone() {
        let error = ErrorRegister::GenericError;
        let error_copy = error;
        let error_cloned = error.clone();
        assert_eq!(error, error_copy);
        assert_eq!(error, error_cloned);
    }
    #[test]
    fn test_error_register_equality() {
        assert_eq!(ErrorRegister::GenericError, ErrorRegister::GenericError);
        assert_ne!(ErrorRegister::GenericError, ErrorRegister::Current);
    }

    #[test]
    fn test_emergency_error_code_debug() {
        let error = EmergencyErrorCode::PdoNotProcessed;
        assert_eq!(format!("{:?}", error), "PdoNotProcessed");
    }

    #[test]
    fn test_emergency_error_code_copy() {
        let error = EmergencyErrorCode::PdoNotProcessed;
        let error_copy = error;
        assert_eq!(error, error_copy);
    }

    #[test]
    fn test_emergency_error_code_clone() {
        let error = EmergencyErrorCode::PdoNotProcessed;
        let error_clone = error.clone();
        assert_eq!(error, error_clone);
    }

    #[test]
    fn test_emergency_error_code_eq() {
        let error1 = EmergencyErrorCode::PdoNotProcessed;
        let error2 = EmergencyErrorCode::PdoNotProcessed;
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_emergency_error_code_ord() {
        let error1 = EmergencyErrorCode::PdoNotProcessed;
        let error2 = EmergencyErrorCode::PdoNotProcessed;
        assert!(error1 <= error2);
        assert!(error1 >= error2);
    }

    #[test]
    fn test_emergency_error_code_hash() {
        let error = EmergencyErrorCode::PdoNotProcessed;
        let mut hasher = DefaultHasher::new();
        error.hash(&mut hasher);
        let hashed = hasher.finish();
        assert_ne!(hashed, 0);
    }
}
