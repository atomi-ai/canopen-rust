use crate::data_type::DataType;
use crate::prelude::*;
use crate::util;

#[derive(Clone, Debug)]
pub struct Value {
    pub data: Vec<u8>,
}

impl Value {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

pub trait ByteConvertible: Sized {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_byte_convertible_for_int {
    ($t:ty, $len:expr) => {
        impl ByteConvertible for $t {
            fn to_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn from_bytes(bytes: &[u8]) -> Self {
                assert_eq!(bytes.len(), $len);
                let array: [u8; $len] = bytes.try_into().expect("Wrong number of bytes");
                <$t>::from_le_bytes(array)
            }
        }
    };
}

impl_byte_convertible_for_int!(i8, 1);
impl_byte_convertible_for_int!(i16, 2);
impl_byte_convertible_for_int!(i32, 4);
impl_byte_convertible_for_int!(i64, 8);
impl_byte_convertible_for_int!(u8, 1);
impl_byte_convertible_for_int!(u16, 2);
impl_byte_convertible_for_int!(u32, 4);
impl_byte_convertible_for_int!(u64, 8);
impl_byte_convertible_for_int!(f32, 4);
impl_byte_convertible_for_int!(f64, 8);

impl ByteConvertible for String {
    fn from_bytes(bytes: &[u8]) -> Self {
        String::from_utf8(bytes.to_vec()).expect("Failed to convert bytes to String")
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Value {
    pub fn from<T: ByteConvertible>(val: T) -> Self {
        let bytes = val.to_bytes();
        Self::new(bytes)
    }

    pub fn to<T: ByteConvertible>(&self) -> T {
        T::from_bytes(self.as_slice())
    }
}

fn string_to_value(data_type: &DataType, data_string: &str) -> Result<Value, String> {
    match data_type {
        DataType::Unknown => Err("Unknown DataType".into()),

        DataType::Boolean => {
            let val: u8 = match data_string.to_lowercase().as_str() {
                "true" | "1" => 1,
                "false" | "0" => 0,
                _ => return Err("Invalid boolean value".into()),
            };
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Integer8 => {
            let val: i8 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Integer16 => {
            let val: i16 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Integer32 => {
            let val: i32 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Integer64 => {
            let val: i64 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Unsigned8 => {
            let val: u8 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Unsigned16 => {
            let val: u16 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Unsigned32 => {
            let val: u32 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Unsigned64 => {
            let val: u64 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Real32 => {
            let val: f32 = data_string.parse().map_err(|_| "Failed to parse f32")?;
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::Real64 => {
            let val: f64 = data_string.parse().map_err(|_| "Failed to parse f64")?;
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::VisibleString | DataType::OctetString | DataType::UnicodeString => Ok(Value {
            data: data_string.as_bytes().to_vec(),
        }),

        DataType::Domain => {
            let val: i32 = data_string
                .parse()
                .map_err(|_| "Failed to parse domain as i32")?;
            Ok(Value {
                data: val.to_bytes(),
            })
        }
    }
}

pub fn get_value(
    properties: &HashMap<String, String>,
    property_name: &str,
    node_id: u16,
    data_type: &DataType,
) -> Option<Value> {
    let mut raw = properties
        .get(&String::from(property_name))
        .unwrap_or(&String::from(""))
        .clone();

    if raw.is_empty() {
        return None;
    }
    if raw.contains("$NODEID") {
        // rewrite the string "123 + $NODEID" => "125" if this is node 2.
        raw = util::to_value_with_node_id(node_id, &raw);
    }

    match string_to_value(data_type, &raw) {
        Ok(val) => Some(val),
        Err(_e) => None,
    }
}
