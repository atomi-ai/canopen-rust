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

    #[allow(dead_code)]
    pub(crate) fn from_code(code: u32) -> Option<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_debug() {
        assert_eq!(format!("{:?}", ErrorCode::ByteLengthExceedsLimit), "Byte length exceeds limit");
        assert_eq!(format!("{:?}", ErrorCode::InvalidStandardId { cob_id: 0x123 }), "Invalid Standard ID: 291");
        assert_eq!(format!("{:?}", ErrorCode::FrameCreationFailed { data: vec![1, 2, 3] }), "Frame creation failed, data: [1, 2, 3]");
        assert_eq!(format!("{:?}", ErrorCode::StringToValueFailed { data_type: DataType::Integer16, str: "test".to_string() }), "String conversion failed, data_type = Integer16, str = '\"test\"'");
        assert_eq!(format!("{:?}", ErrorCode::LegacyError { str: "legacy error".to_string() }), "Legacy error described in string: \"legacy error\"");
        assert_eq!(format!("{:?}", ErrorCode::ProcesedSectionFailed { section_name: "section".to_string(), more_info: "info".to_string() }), "Processed section failed, section_name: \"section\", more info: \"info\"");
        assert_eq!(format!("{:?}", ErrorCode::AbortCodeWrapper { abort_code: AbortCode::GeneralError, more_info: "additional info".to_string() }), "Got Canopen abort code: GeneralError, and more information: \"additional info\"");
        assert_eq!(format!("{:?}", ErrorCode::NoCobIdInFrame), "No cob id");
        assert_eq!(format!("{:?}", ErrorCode::NoCobIdInRpdo { cob_id: 0x123 }), "No cob id (123) in Rpdo");
        assert_eq!(format!("{:?}", ErrorCode::NoPdoObjectInIndex { index: 5 }), "No index(5) in pdo object");
        assert_eq!(format!("{:?}", ErrorCode::VariableNotFound { index: 0x1000, sub_index: 0x01 }), "Not variable on (1000, 1");
    }

    #[test]
    fn test_abort_code() {
        // ToggleBitNotAlternated
        assert_eq!(AbortCode::ToggleBitNotAlternated.code(), 0x0503_0000);
        assert_eq!(AbortCode::ToggleBitNotAlternated.description(), "Toggle bit not alternated");

        // SdoProtocolTimedOut
        assert_eq!(AbortCode::SdoProtocolTimedOut.code(), 0x0504_0000);
        assert_eq!(AbortCode::SdoProtocolTimedOut.description(), "SDO protocol timed out");

        // CommandSpecifierNotValidOrUnknown
        assert_eq!(AbortCode::CommandSpecifierNotValidOrUnknown.code(), 0x0504_0001);
        assert_eq!(AbortCode::CommandSpecifierNotValidOrUnknown.description(), "Client/server command specifier not valid or unknown");

        // InvalidBlockSize
        assert_eq!(AbortCode::InvalidBlockSize.code(), 0x0504_0002);
        assert_eq!(AbortCode::InvalidBlockSize.description(), "Invalid block size (block mode only)");

        // InvalidSequenceNumber
        assert_eq!(AbortCode::InvalidSequenceNumber.code(), 0x0504_0003);
        assert_eq!(AbortCode::InvalidSequenceNumber.description(), "Invalid sequence number (block mode only)");

        // CRCError
        assert_eq!(AbortCode::CRCError.code(), 0x0504_0004);
        assert_eq!(AbortCode::CRCError.description(), "CRC error (block mode only)");

        // OutOfMemory
        assert_eq!(AbortCode::OutOfMemory.code(), 0x0504_0005);
        assert_eq!(AbortCode::OutOfMemory.description(), "Out of memory");

        // UnsupportedAccessToObject
        assert_eq!(AbortCode::UnsupportedAccessToObject.code(), 0x0601_0000);
        assert_eq!(AbortCode::UnsupportedAccessToObject.description(), "Unsupported access to an object");

        // AttemptToReadWriteOnlyObject
        assert_eq!(AbortCode::AttemptToReadWriteOnlyObject.code(), 0x0601_0001);
        assert_eq!(AbortCode::AttemptToReadWriteOnlyObject.description(), "Attempt to read a write only object");

        // AttemptToWriteReadOnlyObject
        assert_eq!(AbortCode::AttemptToWriteReadOnlyObject.code(), 0x0601_0002);
        assert_eq!(AbortCode::AttemptToWriteReadOnlyObject.description(), "Attempt to write a read only object");

        // ObjectDoesNotExistInObjectDictionary
        assert_eq!(AbortCode::ObjectDoesNotExistInObjectDictionary.code(), 0x0602_0000);
        assert_eq!(AbortCode::ObjectDoesNotExistInObjectDictionary.description(), "Object does not exist in the object dictionary");

        // ObjectCannotBeMappedToPDO
        assert_eq!(AbortCode::ObjectCannotBeMappedToPDO.code(), 0x0604_0041);
        assert_eq!(AbortCode::ObjectCannotBeMappedToPDO.description(), "Object cannot be mapped to the PDO");

        // ExceedPDOSize
        assert_eq!(AbortCode::ExceedPDOSize.code(), 0x0604_0042);
        assert_eq!(AbortCode::ExceedPDOSize.description(), "The number and length of the objects to be mapped would exceed PDO length");

        // GeneralParameterIncompatibility
        assert_eq!(AbortCode::GeneralParameterIncompatibility.code(), 0x0604_0043);
        assert_eq!(AbortCode::GeneralParameterIncompatibility.description(), "General parameter incompatibility reason");

        // GeneralInternalIncompatibility
        assert_eq!(AbortCode::GeneralInternalIncompatibility.code(), 0x0604_0047);
        assert_eq!(AbortCode::GeneralInternalIncompatibility.description(), "General internal incompatibility in the device");

        // HardwareError
        assert_eq!(AbortCode::HardwareError.code(), 0x0606_0000);
        assert_eq!(AbortCode::HardwareError.description(), "Access failed due to a hardware error");

        // DataTypeMismatchLengthMismatch
        assert_eq!(AbortCode::DataTypeMismatchLengthMismatch.code(), 0x0607_0010);
        assert_eq!(AbortCode::DataTypeMismatchLengthMismatch.description(), "Data type does not match; length of service parameter does not match");

        // DataTypeMismatchLengthTooHigh
        assert_eq!(AbortCode::DataTypeMismatchLengthTooHigh.code(), 0x0607_0012);
        assert_eq!(AbortCode::DataTypeMismatchLengthTooHigh.description(), "Data type does not match; length of service parameter too high");

        // DataTypeMismatchLengthTooLow
        assert_eq!(AbortCode::DataTypeMismatchLengthTooLow.code(), 0x0607_0013);
        assert_eq!(AbortCode::DataTypeMismatchLengthTooLow.description(), "Data type does not match; length of service parameter too low");

        // SubIndexDoesNotExist
        assert_eq!(AbortCode::SubIndexDoesNotExist.code(), 0x0609_0011);
        assert_eq!(AbortCode::SubIndexDoesNotExist.description(), "Sub-index does not exist");

        // ValueRangeExceeded
        assert_eq!(AbortCode::ValueRangeExceeded.code(), 0x0609_0030);
        assert_eq!(AbortCode::ValueRangeExceeded.description(), "Value range of parameter exceeded (only for write access)");

        // ValueWrittenTooHigh
        assert_eq!(AbortCode::ValueWrittenTooHigh.code(), 0x0609_0031);
        assert_eq!(AbortCode::ValueWrittenTooHigh.description(), "Value of parameter written too high");

        // ValueWrittenTooLow
        assert_eq!(AbortCode::ValueWrittenTooLow.code(), 0x0609_0032);
        assert_eq!(AbortCode::ValueWrittenTooLow.description(), "Value of parameter written too low");

        // MaxValueLessThanMinValue
        assert_eq!(AbortCode::MaxValueLessThanMinValue.code(), 0x0609_0036);
        assert_eq!(AbortCode::MaxValueLessThanMinValue.description(), "Maximum value is less than minimum value");

        // DataTransferOrStoreFailed
        assert_eq!(AbortCode::DataTransferOrStoreFailed.code(), 0x0800_0020);
        assert_eq!(AbortCode::DataTransferOrStoreFailed.description(), "Data cannot be transferred or stored to the application");

        // DataTransferOrStoreFailedDueToLocalControl
        assert_eq!(AbortCode::DataTransferOrStoreFailedDueToLocalControl.code(), 0x0800_0021);
        assert_eq!(AbortCode::DataTransferOrStoreFailedDueToLocalControl.description(), "Data cannot be transferred or stored to the application because of local control");

        // DataTransferOrStoreFailedDueToDeviceState
        assert_eq!(AbortCode::DataTransferOrStoreFailedDueToDeviceState.code(), 0x0800_0022);
        assert_eq!(AbortCode::DataTransferOrStoreFailedDueToDeviceState.description(), "Data cannot be transferred or stored to the application because of the present device state");

        // ObjectDictionaryGenerationFailedOrNotPresent
        assert_eq!(AbortCode::ObjectDictionaryGenerationFailedOrNotPresent.code(), 0x0800_0023);
        assert_eq!(AbortCode::ObjectDictionaryGenerationFailedOrNotPresent.description(), "Object dictionary dynamic generation fails or no object dictionary is present (e.g. object dictionary is generated from file and generation fails because of a file error)");

        // GeneralError
        assert_eq!(AbortCode::GeneralError.code(), 0x0800_0000);
        assert_eq!(AbortCode::GeneralError.description(), "General error");

        // Other
        assert_eq!(AbortCode::Other.code(), 0x0000_0000);
        assert_eq!(AbortCode::Other.description(), "Other");
    }

    #[test]
    fn test_from_code() {
        assert_eq!(AbortCode::from_code(0x0503_0000), Some(AbortCode::ToggleBitNotAlternated));
        assert_eq!(AbortCode::from_code(0x0504_0000), Some(AbortCode::SdoProtocolTimedOut));
        assert_eq!(AbortCode::from_code(0x0504_0001), Some(AbortCode::CommandSpecifierNotValidOrUnknown));
        assert_eq!(AbortCode::from_code(0x0504_0002), Some(AbortCode::InvalidBlockSize));
        assert_eq!(AbortCode::from_code(0x0504_0003), Some(AbortCode::InvalidSequenceNumber));
        assert_eq!(AbortCode::from_code(0x0504_0004), Some(AbortCode::CRCError));
        assert_eq!(AbortCode::from_code(0x0504_0005), Some(AbortCode::OutOfMemory));
        assert_eq!(AbortCode::from_code(0x0601_0000), Some(AbortCode::UnsupportedAccessToObject));
        assert_eq!(AbortCode::from_code(0x0601_0001), Some(AbortCode::AttemptToReadWriteOnlyObject));
        assert_eq!(AbortCode::from_code(0x0601_0002), Some(AbortCode::AttemptToWriteReadOnlyObject));
        assert_eq!(AbortCode::from_code(0x0602_0000), Some(AbortCode::ObjectDoesNotExistInObjectDictionary));
        assert_eq!(AbortCode::from_code(0x0604_0041), Some(AbortCode::ObjectCannotBeMappedToPDO));
        assert_eq!(AbortCode::from_code(0x0604_0042), Some(AbortCode::ExceedPDOSize));
        assert_eq!(AbortCode::from_code(0x0604_0043), Some(AbortCode::GeneralParameterIncompatibility));
        assert_eq!(AbortCode::from_code(0x0604_0047), Some(AbortCode::GeneralInternalIncompatibility));
        assert_eq!(AbortCode::from_code(0x0606_0000), Some(AbortCode::HardwareError));
        assert_eq!(AbortCode::from_code(0x0607_0010), Some(AbortCode::DataTypeMismatchLengthMismatch));
        assert_eq!(AbortCode::from_code(0x0607_0012), Some(AbortCode::DataTypeMismatchLengthTooHigh));
        assert_eq!(AbortCode::from_code(0x0607_0013), Some(AbortCode::DataTypeMismatchLengthTooLow));
        assert_eq!(AbortCode::from_code(0x0609_0011), Some(AbortCode::SubIndexDoesNotExist));
        assert_eq!(AbortCode::from_code(0x0609_0030), Some(AbortCode::ValueRangeExceeded));
        assert_eq!(AbortCode::from_code(0x0609_0031), Some(AbortCode::ValueWrittenTooHigh));
        assert_eq!(AbortCode::from_code(0x0609_0032), Some(AbortCode::ValueWrittenTooLow));
        assert_eq!(AbortCode::from_code(0x0609_0036), Some(AbortCode::MaxValueLessThanMinValue));
        assert_eq!(AbortCode::from_code(0x0800_0000), Some(AbortCode::GeneralError));
        assert_eq!(AbortCode::from_code(0x0800_0020), Some(AbortCode::DataTransferOrStoreFailed));
        assert_eq!(AbortCode::from_code(0x0800_0021), Some(AbortCode::DataTransferOrStoreFailedDueToLocalControl));
        assert_eq!(AbortCode::from_code(0x0800_0022), Some(AbortCode::DataTransferOrStoreFailedDueToDeviceState));
        assert_eq!(AbortCode::from_code(0x0800_0023), Some(AbortCode::ObjectDictionaryGenerationFailedOrNotPresent));

        assert_eq!(AbortCode::from_code(0xFFFFFFFF), None);
    }
}
