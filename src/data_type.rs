use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    Unknown = 0x0,
    Boolean = 0x1,
    Integer8 = 0x2,
    Integer16 = 0x3,
    Integer32 = 0x4,
    Unsigned8 = 0x5,
    Unsigned16 = 0x6,
    Unsigned32 = 0x7,
    Real32 = 0x8,
    VisibleString = 0x9,
    OctetString = 0xA,
    UnicodeString = 0xB,
    Domain = 0xF,
    Real64 = 0x11,
    Integer64 = 0x15,
    Unsigned64 = 0x1B,
}

impl DataType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x0 => DataType::Unknown,
            0x1 => DataType::Boolean,
            0x2 => DataType::Integer8,
            0x3 => DataType::Integer16,
            0x4 => DataType::Integer32,
            0x5 => DataType::Unsigned8,
            0x6 => DataType::Unsigned16,
            0x7 => DataType::Unsigned32,
            0x8 => DataType::Real32,
            0x9 => DataType::VisibleString,
            0xA => DataType::OctetString,
            0xB => DataType::UnicodeString,
            0xF => DataType::Domain,
            0x11 => DataType::Real64,
            0x15 => DataType::Integer64,
            0x1B => DataType::Unsigned64,
            _ => DataType::Unknown,
        }
    }

    // Return size of a type.
    // Size 0 means it is variant.
    pub fn size(&self) -> usize {
        match self {
            DataType::Unknown => 0,       // Size 0 for unknown data type
            DataType::Boolean => 1,       // 1 byte
            DataType::Integer8 => 1,      // 1 byte
            DataType::Integer16 => 2,     // 2 bytes
            DataType::Integer32 => 4,     // 4 bytes
            DataType::Unsigned8 => 1,     // 1 byte
            DataType::Unsigned16 => 2,    // 2 bytes
            DataType::Unsigned32 => 4,    // 4 bytes
            DataType::Real32 => 4,        // 4 bytes
            DataType::VisibleString => 0, // 1 byte per character, but variable length
            DataType::OctetString => 0,   // 1 byte per character, but variable length
            DataType::UnicodeString => 0, // 2 bytes per character, but variable length
            DataType::Domain => 4,        // Variable length, assuming 1 for simplicity
            DataType::Real64 => 8,        // 8 bytes
            DataType::Integer64 => 8,     // 8 bytes
            DataType::Unsigned64 => 8,    // 8 bytes
        }
    }

    pub fn default_value(&self) -> Vec<u8> {
        match *self {
            DataType::Unknown | DataType::Boolean => vec![0x0],
            DataType::Integer8 | DataType::Unsigned8 => vec![0x0],
            DataType::Integer16 | DataType::Unsigned16 => vec![0x0, 0x0],
            DataType::Integer32 | DataType::Unsigned32 | DataType::Real32 => {
                vec![0x0, 0x0, 0x0, 0x0]
            }
            DataType::VisibleString | DataType::OctetString | DataType::UnicodeString => vec![],
            DataType::Domain => vec![], // TODO(zephyr): understand and implement domain type.
            DataType::Real64 | DataType::Integer64 | DataType::Unsigned64 => {
                vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
            }
        }
    }
}
