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
}
