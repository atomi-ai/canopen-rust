use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
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

impl Ord for DataType {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u16).cmp(&(*other as u16))
    }
}

impl PartialOrd for DataType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for DataType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u16).hash(state);
    }
}

impl DataType {
    pub(crate) fn from_u32(value: u32) -> Self {
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
    pub(crate) fn size(&self) -> usize {
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

    pub(crate) fn default_value(&self) -> Vec<u8> {
        match *self {
            DataType::Unknown | DataType::Boolean => vec![0x0],
            DataType::Integer8 | DataType::Unsigned8 => vec![0x0],
            DataType::Integer16 | DataType::Unsigned16 => vec![0x0, 0x0],
            DataType::Integer32 | DataType::Unsigned32 | DataType::Real32 => {
                vec![0x0, 0x0, 0x0, 0x0]
            }
            DataType::VisibleString | DataType::OctetString | DataType::UnicodeString => vec![],
            DataType::Domain => vec![0x0],
            DataType::Real64 | DataType::Integer64 | DataType::Unsigned64 => {
                vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_from_u32() {
        assert_eq!(DataType::from_u32(0x0), DataType::Unknown);
        assert_eq!(DataType::from_u32(0x1), DataType::Boolean);
        assert_eq!(DataType::from_u32(0x2), DataType::Integer8);
        assert_eq!(DataType::from_u32(0x3), DataType::Integer16);
        assert_eq!(DataType::from_u32(0x4), DataType::Integer32);
        assert_eq!(DataType::from_u32(0x5), DataType::Unsigned8);
        assert_eq!(DataType::from_u32(0x6), DataType::Unsigned16);
        assert_eq!(DataType::from_u32(0x7), DataType::Unsigned32);
        assert_eq!(DataType::from_u32(0x8), DataType::Real32);
        assert_eq!(DataType::from_u32(0x9), DataType::VisibleString);
        assert_eq!(DataType::from_u32(0xA), DataType::OctetString);
        assert_eq!(DataType::from_u32(0xB), DataType::UnicodeString);
        assert_eq!(DataType::from_u32(0xF), DataType::Domain);
        assert_eq!(DataType::from_u32(0x11), DataType::Real64);
        assert_eq!(DataType::from_u32(0x15), DataType::Integer64);
        assert_eq!(DataType::from_u32(0x1B), DataType::Unsigned64);
        assert_eq!(DataType::from_u32(0xFF), DataType::Unknown);
    }

    #[test]
    fn test_size() {
        assert_eq!(DataType::Unknown.size(), 0);
        assert_eq!(DataType::Boolean.size(), 1);
        assert_eq!(DataType::Integer8.size(), 1);
        assert_eq!(DataType::Integer16.size(), 2);
        assert_eq!(DataType::Integer32.size(), 4);
        assert_eq!(DataType::Unsigned8.size(), 1);
        assert_eq!(DataType::Unsigned16.size(), 2);
        assert_eq!(DataType::Unsigned32.size(), 4);
        assert_eq!(DataType::Real32.size(), 4);
        assert_eq!(DataType::VisibleString.size(), 0);
        assert_eq!(DataType::OctetString.size(), 0);
        assert_eq!(DataType::UnicodeString.size(), 0);
        assert_eq!(DataType::Domain.size(), 4);
        assert_eq!(DataType::Real64.size(), 8);
        assert_eq!(DataType::Integer64.size(), 8);
        assert_eq!(DataType::Unsigned64.size(), 8);
    }

    #[test]
    fn test_default_value() {
        assert_eq!(DataType::Unknown.default_value(), vec![0x0]);
        assert_eq!(DataType::Boolean.default_value(), vec![0x0]);
        assert_eq!(DataType::Integer8.default_value(), vec![0x0]);
        assert_eq!(DataType::Integer16.default_value(), vec![0x0, 0x0]);
        assert_eq!(DataType::Integer32.default_value(), vec![0x0, 0x0, 0x0, 0x0]);
        assert_eq!(DataType::Unsigned8.default_value(), vec![0x0]);
        assert_eq!(DataType::Unsigned16.default_value(), vec![0x0, 0x0]);
        assert_eq!(DataType::Unsigned32.default_value(), vec![0x0, 0x0, 0x0, 0x0]);
        assert_eq!(DataType::Real32.default_value(), vec![0x0, 0x0, 0x0, 0x0]);
        assert_eq!(DataType::VisibleString.default_value(), vec![]);
        assert_eq!(DataType::OctetString.default_value(), vec![]);
        assert_eq!(DataType::UnicodeString.default_value(), vec![]);
        assert_eq!(DataType::Domain.default_value(), vec![0x0]);
        assert_eq!(DataType::Real64.default_value(), vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
        assert_eq!(DataType::Integer64.default_value(), vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
        assert_eq!(DataType::Unsigned64.default_value(), vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]);
    }

    #[test]
    fn test_data_type_ordering() {
        let type1 = DataType::Boolean;
        let type2 = DataType::Integer32;

        assert!(type1 < type2);
        assert!(type2 > type1);

        assert!(type1.partial_cmp(&type2) == Some(Ordering::Less));
        assert!(type2.partial_cmp(&type1) == Some(Ordering::Greater));
    }

    #[test]
    fn test_data_type_hash() {
        let data_type = DataType::Integer8;
        let mut hasher = DefaultHasher::new();
        data_type.hash(&mut hasher);
        let hashed = hasher.finish();
        assert_ne!(hashed, 0);

        let data_type2 = DataType::Integer16;
        let mut hasher2 = DefaultHasher::new();
        data_type2.hash(&mut hasher2);
        let hashed2 = hasher2.finish();
        assert_ne!(hashed, hashed2);
    }
}
