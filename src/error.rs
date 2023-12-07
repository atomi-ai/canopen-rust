use core::fmt::Formatter;
use crate::data_type::DataType;
use crate::prelude::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorCode {
    ByteLengthExceedsLimit,
    InvalidStandardId { cob_id: u16 },
    FrameCreationFailed { data: Vec<u8> },
    NoCobIdInFrame,
    NoCobIdInRpdo { cob_id: u16 },
    StringToValueFailed { data_type: DataType, str: String },
    ProcesedSectionFailed { section_name: String, more_info: String },
    AbortCodeWrapper { abort_code: AbortCode, more_info: String },
    NoPdoObjectInIndex { index: usize },
    VariableNotFound {index: u16, sub_index: u8},
    LegacyError { str: String },
}

impl Debug for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::ByteLengthExceedsLimit => write!(f, "Byte length exceeds limit"),
            ErrorCode::InvalidStandardId { cob_id } => write!(f, "Invalid Standard ID: {}", cob_id),
            ErrorCode::FrameCreationFailed { data } => write!(f, "Frame creation failed, data: {:x?}", data),
            ErrorCode::StringToValueFailed { data_type, str } =>
                write!(f, "String conversion failed, data_type = {:?}, str = '{:?}'", data_type, str),
            ErrorCode::LegacyError { str } => write!(f, "Legacy error described in string: {:?}", str),
            ErrorCode::ProcesedSectionFailed { section_name, more_info } =>
                write!(f, "Processed section failed, section_name: {:?}, more info: {:?}",
                section_name, more_info),
            ErrorCode::AbortCodeWrapper { abort_code, more_info } => write!(f,
                "Got Canopen abort code: {:x?}, and more information: {:?}", abort_code, more_info),
            ErrorCode::NoCobIdInFrame => write!(f, "No cob id"),
            ErrorCode::NoCobIdInRpdo { cob_id } => write!(f, "No cob id ({:x?}) in Rpdo", cob_id),
            ErrorCode::NoPdoObjectInIndex { index } => write!(f, "No index({}) in pdo object", index),
            ErrorCode::VariableNotFound { index, sub_index } => write!(f, "Not variable on ({:x?}, {:x?}", index, sub_index),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AbortCode {
    ToggleBitNotAlternated,
    SdoProtocolTimedOut,
    CommandSpecifierNotValidOrUnknown,
    InvalidBlockSize,
    InvalidSequenceNumber,
    CRCError,
    OutOfMemory,
    UnsupportedAccessToObject,
    AttemptToReadWriteOnlyObject,
    AttemptToWriteReadOnlyObject,
    ObjectDoesNotExistInObjectDictionary,
    ObjectCannotBeMappedToPDO,
    ExceedPDOSize,
    GeneralParameterIncompatibility,
    GeneralInternalIncompatibility,
    HardwareError,
    DataTypeMismatchLengthMismatch,
    DataTypeMismatchLengthTooHigh,
    DataTypeMismatchLengthTooLow,
    SubIndexDoesNotExist,
    ValueRangeExceeded,
    ValueWrittenTooHigh,
    ValueWrittenTooLow,
    MaxValueLessThanMinValue,
    GeneralError,
    DataTransferOrStoreFailed,
    DataTransferOrStoreFailedDueToLocalControl,
    DataTransferOrStoreFailedDueToDeviceState,
    ObjectDictionaryGenerationFailedOrNotPresent,

    Other,
}

impl AbortCode {
    pub fn code(&self) -> u32 {
        match *self {
            AbortCode::ToggleBitNotAlternated => 0x0503_0000,
            AbortCode::SdoProtocolTimedOut => 0x0504_0000,
            AbortCode::CommandSpecifierNotValidOrUnknown => 0x0504_0001,
            AbortCode::InvalidBlockSize => 0x0504_0002,
            AbortCode::InvalidSequenceNumber => 0x0504_0003,
            AbortCode::CRCError => 0x0504_0004,
            AbortCode::OutOfMemory => 0x0504_0005,
            AbortCode::UnsupportedAccessToObject => 0x0601_0000,
            AbortCode::AttemptToReadWriteOnlyObject => 0x0601_0001,
            AbortCode::AttemptToWriteReadOnlyObject => 0x0601_0002,
            AbortCode::ObjectDoesNotExistInObjectDictionary => 0x0602_0000,
            AbortCode::ObjectCannotBeMappedToPDO => 0x0604_0041,
            AbortCode::ExceedPDOSize => 0x0604_0042,
            AbortCode::GeneralParameterIncompatibility => 0x0604_0043,
            AbortCode::GeneralInternalIncompatibility => 0x0604_0047,
            AbortCode::HardwareError => 0x0606_0000,
            AbortCode::DataTypeMismatchLengthMismatch => 0x0607_0010,
            AbortCode::DataTypeMismatchLengthTooHigh => 0x0607_0012,
            AbortCode::DataTypeMismatchLengthTooLow => 0x0607_0013,
            AbortCode::SubIndexDoesNotExist => 0x0609_0011,
            AbortCode::ValueRangeExceeded => 0x0609_0030,
            AbortCode::ValueWrittenTooHigh => 0x0609_0031,
            AbortCode::ValueWrittenTooLow => 0x0609_0032,
            AbortCode::MaxValueLessThanMinValue => 0x0609_0036,
            AbortCode::GeneralError => 0x0800_0000,
            AbortCode::DataTransferOrStoreFailed => 0x0800_0020,
            AbortCode::DataTransferOrStoreFailedDueToLocalControl => 0x0800_0021,
            AbortCode::DataTransferOrStoreFailedDueToDeviceState => 0x0800_0022,
            AbortCode::ObjectDictionaryGenerationFailedOrNotPresent => 0x0800_0023,

            // Only used in the project
            AbortCode::Other => 0x0000_0000,
        }
    }

    pub fn description(&self) -> &'static str {
        match *self {
            AbortCode::ToggleBitNotAlternated => "Toggle bit not alternated",
            AbortCode::SdoProtocolTimedOut => "SDO protocol timed out",
            AbortCode::CommandSpecifierNotValidOrUnknown => "Client/server command specifier not valid or unknown",
            AbortCode::InvalidBlockSize => "Invalid block size (block mode only)",
            AbortCode::InvalidSequenceNumber => "Invalid sequence number (block mode only)",
            AbortCode::CRCError => "CRC error (block mode only)",
            AbortCode::OutOfMemory => "Out of memory",
            AbortCode::UnsupportedAccessToObject => "Unsupported access to an object",
            AbortCode::AttemptToReadWriteOnlyObject => "Attempt to read a write only object",
            AbortCode::AttemptToWriteReadOnlyObject => "Attempt to write a read only object",
            AbortCode::ObjectDoesNotExistInObjectDictionary => "Object does not exist in the object dictionary",
            AbortCode::ObjectCannotBeMappedToPDO => "Object cannot be mapped to the PDO",
            AbortCode::ExceedPDOSize => "The number and length of the objects to be mapped would exceed PDO length",
            AbortCode::GeneralParameterIncompatibility => "General parameter incompatibility reason",
            AbortCode::GeneralInternalIncompatibility => "General internal incompatibility in the device",
            AbortCode::HardwareError => "Access failed due to a hardware error",
            AbortCode::DataTypeMismatchLengthMismatch => "Data type does not match; length of service parameter does not match",
            AbortCode::DataTypeMismatchLengthTooHigh => "Data type does not match; length of service parameter too high",
            AbortCode::DataTypeMismatchLengthTooLow => "Data type does not match; length of service parameter too low",
            AbortCode::SubIndexDoesNotExist => "Sub-index does not exist",
            AbortCode::ValueRangeExceeded => "Value range of parameter exceeded (only for write access)",
            AbortCode::ValueWrittenTooHigh => "Value of parameter written too high",
            AbortCode::ValueWrittenTooLow => "Value of parameter written too low",
            AbortCode::MaxValueLessThanMinValue => "Maximum value is less than minimum value",
            AbortCode::GeneralError => "General error",
            AbortCode::DataTransferOrStoreFailed => "Data cannot be transferred or stored to the application",
            AbortCode::DataTransferOrStoreFailedDueToLocalControl => "Data cannot be transferred or stored to the application because of local control",
            AbortCode::DataTransferOrStoreFailedDueToDeviceState => "Data cannot be transferred or stored to the application because of the present device state",
            AbortCode::ObjectDictionaryGenerationFailedOrNotPresent => "Object dictionary dynamic generation fails or no object dictionary is present (e.g. object dictionary is generated from file and generation fails because of a file error)",

            AbortCode::Other => "Other",
        }
    }

    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0x0503_0000 => Some(AbortCode::ToggleBitNotAlternated),
            0x0504_0000 => Some(AbortCode::SdoProtocolTimedOut),
            0x0504_0001 => Some(AbortCode::CommandSpecifierNotValidOrUnknown),
            0x0504_0002 => Some(AbortCode::InvalidBlockSize),
            0x0504_0003 => Some(AbortCode::InvalidSequenceNumber),
            0x0504_0004 => Some(AbortCode::CRCError),
            0x0504_0005 => Some(AbortCode::OutOfMemory),
            0x0601_0000 => Some(AbortCode::UnsupportedAccessToObject),
            0x0601_0001 => Some(AbortCode::AttemptToReadWriteOnlyObject),
            0x0601_0002 => Some(AbortCode::AttemptToWriteReadOnlyObject),
            0x0602_0000 => Some(AbortCode::ObjectDoesNotExistInObjectDictionary),
            0x0604_0041 => Some(AbortCode::ObjectCannotBeMappedToPDO),
            0x0604_0042 => Some(AbortCode::ExceedPDOSize),
            0x0604_0043 => Some(AbortCode::GeneralParameterIncompatibility),
            0x0604_0047 => Some(AbortCode::GeneralInternalIncompatibility),
            0x0606_0000 => Some(AbortCode::HardwareError),
            0x0607_0010 => Some(AbortCode::DataTypeMismatchLengthMismatch),
            0x0607_0012 => Some(AbortCode::DataTypeMismatchLengthTooHigh),
            0x0607_0013 => Some(AbortCode::DataTypeMismatchLengthTooLow),
            0x0609_0011 => Some(AbortCode::SubIndexDoesNotExist),
            0x0609_0030 => Some(AbortCode::ValueRangeExceeded),
            0x0609_0031 => Some(AbortCode::ValueWrittenTooHigh),
            0x0609_0032 => Some(AbortCode::ValueWrittenTooLow),
            0x0609_0036 => Some(AbortCode::MaxValueLessThanMinValue),
            0x0800_0000 => Some(AbortCode::GeneralError),
            0x0800_0020 => Some(AbortCode::DataTransferOrStoreFailed),
            0x0800_0021 => Some(AbortCode::DataTransferOrStoreFailedDueToLocalControl),
            0x0800_0022 => Some(AbortCode::DataTransferOrStoreFailedDueToDeviceState),
            0x0800_0023 => Some(AbortCode::ObjectDictionaryGenerationFailedOrNotPresent),
            _ => None,
        }
    }
}
