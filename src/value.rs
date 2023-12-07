use crate::data_type::DataType;
use crate::prelude::*;
use crate::{error, util};
use crate::error::ErrorCode;

#[derive(Clone, Debug)]
pub struct Value {
    data: Vec<u8>,
}

impl Value {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
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
                if bytes.len() == $len {
                    if let Ok(arr) = bytes.try_into() {
                        return <$t>::from_le_bytes(arr);
                    }
                }
                0 as $t
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
        String::from_utf8(bytes.to_vec()).unwrap_or_default()
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

fn make_error(data_type: DataType, data_string: &str) -> ErrorCode {
    ErrorCode::StringToValueFailed {
        data_type,
        str: data_string.to_string(),
    }
}

fn string_to_value(data_type: &DataType, data_string: &str) -> Result<Value, ErrorCode> {
    match data_type {
        DataType::Unknown => Err(make_error(*data_type, data_string)),

        DataType::Boolean => {
            let val: u8 = match data_string.to_lowercase().as_str() {
                "true" | "1" => 1,
                "false" | "0" => 0,
                _ => return Err(make_error(*data_type, data_string)),
            };
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Integer8 => {
            let val: i8 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Integer16 => {
            let val: i16 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Integer32 => {
            let val: i32 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Integer64 => {
            let val: i64 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Unsigned8 => {
            let val: u8 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Unsigned16 => {
            let val: u16 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Unsigned32 => {
            let val: u32 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Unsigned64 => {
            let val: u64 = util::parse_number(data_string);
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Real32 => {
            let val: f32 = data_string.parse().map_err(
                |_| make_error(*data_type, data_string))?;
            Ok(Value::new(val.to_bytes()))
        }

        DataType::Real64 => {
            let val: f64 = data_string.parse().map_err(
                |_| make_error(*data_type, data_string))?;
            Ok(Value::new(val.to_bytes()))
        }

        DataType::VisibleString | DataType::OctetString | DataType::UnicodeString => Ok(Value {
            data: data_string.as_bytes().to_vec(),
        }),

        DataType::Domain => {
            let val: i32 = data_string
                .parse()
                .map_err(|_| make_error(*data_type, data_string))?;
            Ok(Value::new(val.to_bytes()))
        }
    }
}

pub fn evaluate_expression_with_node_id(node_id: u8, expression: &str) -> String {
    // Replace $NODEID with the actual node_id
    let modified_expression = expression.replace("$NODEID", &node_id.to_string());

    // Evaluate simple arithmetic expressions
    modified_expression
        .split('+')
        .map(|s| s.trim())
        .filter_map(|s| if s.starts_with("0x") || s.starts_with("0X") {
            i64::from_str_radix(&s[2..], 16).ok()
        } else {
            s.parse::<i64>().ok()
        })
        .sum::<i64>()
        .to_string()
}

pub fn get_formatted_value_from_properties(
    properties: &HashMap<String, String>,
    property_name: &str,
    node_id: u8,
    data_type: &DataType,
) -> Option<Value> {
    let raw = match properties.get(property_name) {
        Some(value) if !value.is_empty() => value,
        _ => return None,
    };

    let modified_raw = if raw.contains("$NODEID") {
        evaluate_expression_with_node_id(node_id, raw)
    } else {
        raw.clone()
    };

    match string_to_value(data_type, &modified_raw) {
        Ok(val) => Some(val),
        Err(e) => {
            error!("Error converting string to value: {:?}", e);
            None
        },
    }
}

#[cfg(test)]
mod value_tests {
    use super::evaluate_expression_with_node_id;

    #[test]
    fn test_to_value_with_node_id() {
        assert_eq!(evaluate_expression_with_node_id(2, "$NODEID + 100"), "102");
        assert_eq!(evaluate_expression_with_node_id(2, "100+$NODEID"), "102");
        assert_eq!(evaluate_expression_with_node_id(2, "100"), "100");
        assert_eq!(evaluate_expression_with_node_id(2, "$NODEID+100+200"), "302");
        assert_eq!(evaluate_expression_with_node_id(2, "$NODEID + 100 + 200"), "302");
        assert_eq!(evaluate_expression_with_node_id(34, "$NODEID + 100 + 200"), "334");
        assert_eq!(evaluate_expression_with_node_id(2, "No arithmetic here"), "0");
        assert_eq!(evaluate_expression_with_node_id(2, "$NODEID+0x600"), "1538");
    }
}