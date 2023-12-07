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

    pub(crate) fn data(&self) -> &Vec<u8> {
        &self.data
    }
    pub(crate) fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    fn as_slice(&self) -> &[u8] {
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

fn evaluate_expression_with_node_id(node_id: u8, expression: &str) -> String {
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

pub(crate) fn get_formatted_value_from_properties(
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
    use alloc::string::{String, ToString};
    use alloc::{format, vec};
    use crate::data_type::DataType;
    use super::{ByteConvertible, evaluate_expression_with_node_id, make_error, string_to_value, Value};

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

    #[test]
    fn test_value_creation_and_access() {
        let data = vec![1, 2, 3, 4];
        let value = Value::new(data.clone());

        assert_eq!(value.data(), &data);
        assert_eq!(value.as_slice(), data.as_slice());
    }

    #[test]
    fn test_byte_convertible_for_integers() {
        let int_values = [42i32, -1i32, 0i32, i32::MAX, i32::MIN];
        for &val in int_values.iter() {
            let value = Value::from(val);
            assert_eq!(value.to::<i32>(), val);
        }

        let uint_values = [42u32, 0u32, u32::MAX];
        for &val in uint_values.iter() {
            let value = Value::from(val);
            assert_eq!(value.to::<u32>(), val);
        }
    }

    #[test]
    fn test_byte_convertible_for_floats() {
        let float_values = [42.0f32, -1.0f32, 0.0f32, f32::MAX, f32::MIN];
        for &val in float_values.iter() {
            let value = Value::from(val);
            assert_eq!(value.to::<f32>(), val);
        }
    }

    #[test]
    fn test_byte_convertible_for_string() {
        let string_values = ["hello", "world", ""];
        for &val in string_values.iter() {
            let value = Value::from(val.to_string());
            assert_eq!(value.to::<String>(), val);
        }
    }
    #[test]
    fn test_value_partial_eq() {
        let value1 = Value::new(vec![1, 2, 3, 4]);
        let value2 = Value::new(vec![1, 2, 3, 4]);
        let value3 = Value::new(vec![4, 3, 2, 1]);

        assert_eq!(value1, value2);
        assert_ne!(value1, value3);
    }

    #[test]
    fn test_byte_convertible_default_value() {
        let wrong_length_bytes = vec![1, 2, 3];
        assert_eq!(i32::from_bytes(wrong_length_bytes.as_slice()), 0i32);

        let invalid_utf8_bytes = vec![0xFF, 0xFF, 0xFF];
        assert_eq!(String::from_bytes(invalid_utf8_bytes.as_slice()), "".to_string());
    }

    // Tests for ByteConvertible trait
    #[test]
    fn test_from_bytes_for_int_with_wrong_length_returns_zero() {
        assert_eq!(u8::from_bytes(&[0x01, 0x02]), 0);
        assert_eq!(i16::from_bytes(&[0x01]), 0);
        assert_eq!(u32::from_bytes(&[0x01, 0x02, 0x03]), 0);
    }

    #[test]
    fn test_from_bytes_for_string_with_wrong_length_returns_empty_string() {
        assert_eq!(String::from_bytes(&[0xFF, 0xFF]), "");
    }

    // Tests for Value struct
    #[test]
    fn test_clone_for_value() {
        let original = Value::new(vec![1, 2, 3]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_debug_for_value() {
        let value = Value::new(vec![1, 2, 3]);
        let debug_output = format!("{:?}", value);
        assert!(debug_output.contains("Value"));
    }

    #[test]
    fn test_string_to_value_for_boolean() {
        assert_eq!(string_to_value(&DataType::Boolean, "true"), Ok(Value::new(vec![1])));
        assert_eq!(string_to_value(&DataType::Boolean, "false"), Ok(Value::new(vec![0])));
        assert_eq!(string_to_value(&DataType::Boolean, "invalid"), Err(make_error(DataType::Boolean, "invalid")));
    }

    #[test]
    fn test_string_to_value_for_integer8() {
        assert_eq!(string_to_value(&DataType::Integer8, "127"), Ok(Value::new(127i8.to_bytes())));
        assert_eq!(string_to_value(&DataType::Integer8, "-128"), Ok(Value::new((-128i8).to_bytes())));
        assert_eq!(string_to_value(&DataType::Integer8, "invalid"), Ok(Value::new(vec![0])));
    }

    #[test]
    fn test_string_to_value_for_visible_string() {
        let test_string = "Hello";
        assert_eq!(string_to_value(&DataType::VisibleString, test_string), Ok(Value::new(test_string.as_bytes().to_vec())));
    }

    #[test]
    fn test_string_to_value_for_domain() {
        assert_eq!(string_to_value(&DataType::Domain, "123"), Ok(Value::new(123i32.to_bytes())));
        assert_eq!(string_to_value(&DataType::Domain, "invalid"), Err(make_error(DataType::Domain, "invalid")));
    }
}