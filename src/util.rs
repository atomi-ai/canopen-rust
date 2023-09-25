use crate::prelude::*;

use core::str::FromStr;
use embedded_can::{Frame, Id};

pub trait ParseRadix: FromStr {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

macro_rules! impl_parse_radix_signed {
    ($signed:ty, $unsigned:ty, $limit:expr, $upscale:ty, $wrap_around:expr) => {
        impl ParseRadix for $signed {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                let val = <$unsigned>::from_str_radix(s, radix)?;
                if val <= $limit {
                    Ok(val as $signed)
                } else {
                    Ok((val as $upscale - $wrap_around) as $signed)
                }
            }
        }
    };
}

impl_parse_radix_signed!(i8, u8, 0x7F, i16, 0x100);
impl_parse_radix_signed!(i16, u16, 0x7FFF, i32, 0x10000);
impl_parse_radix_signed!(i32, u32, 0x7FFFFFFF, i64, 0x100000000);
impl_parse_radix_signed!(i64, u64, 0x7FFFFFFFFFFFFFFF, i128, 0x10000000000000000);

macro_rules! impl_parse_radix_for {
    ($t:ty) => {
        impl ParseRadix for $t {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, <Self as FromStr>::Err> {
                <$t>::from_str_radix(s, radix)
            }
        }
    };
}

impl_parse_radix_for!(u8);
impl_parse_radix_for!(u16);
impl_parse_radix_for!(u32);
impl_parse_radix_for!(u64);

pub fn parse_number<T: ParseRadix + Default>(s: &str) -> T {
    if s.starts_with("0x") || s.starts_with("0X") {
        T::from_str_radix(&s[2..], 16).unwrap_or_default()
    } else {
        s.parse().unwrap_or_default()
    }
}

pub fn result_to_option<T, Err>(res: Result<T, Err>) -> Option<T> {
    match res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

pub fn to_value_with_node_id(node_id: u16, expression: &str) -> String {
    // Replace $NODEID with the actual node_id
    let modified_expression = expression.replace("$NODEID", &node_id.to_string());

    // Evaluate simple arithmetic expressions
    let value_sum: i64 = modified_expression
        .split('+')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .sum();

    // Convert the evaluated sum to a string
    value_sum.to_string()
}

pub fn get_standard_can_id_from_frame<F: Frame>(frame: &F) -> Option<u16> {
    if let Id::Standard(sid) = frame.id() {
        return Some(sid.as_raw());
    }
    None
}

fn is_hex_char(c: char) -> bool {
    ('0'..='9').contains(&c) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}

pub fn is_top(s: &str) -> bool {
    s.len() == 4 && s.chars().all(is_hex_char)
}

pub fn is_sub(s: &str) -> Option<(u16, u8)> {
    if s.len() > 7 && s[4..7].eq_ignore_ascii_case("sub") && s[0..4].chars().all(is_hex_char) {
        let (index_str, sub_str) = (&s[0..4], &s[7..]);
        match (u16::from_str_radix(index_str, 16), u8::from_str(sub_str)) {
            (Ok(index), Ok(sub)) => Some((index, sub)),
            _ => None,
        }
    } else {
        None
    }
}

pub fn is_name(s: &str) -> Option<u16> {
    s.ends_with("Name")
        .then(|| s[0..4].chars().all(is_hex_char))
        .and_then(|valid| valid.then(|| u16::from_str_radix(&s[0..4], 16).ok()))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::to_value_with_node_id;
    use crate::util::parse_number;

    #[test]
    fn test_to_value_with_node_id() {
        assert_eq!(to_value_with_node_id(2, "$NODEID + 100"), "102");
        assert_eq!(to_value_with_node_id(2, "100+$NODEID"), "102");
        assert_eq!(to_value_with_node_id(2, "100"), "100");
        assert_eq!(to_value_with_node_id(2, "$NODEID+100+200"), "302");
        assert_eq!(to_value_with_node_id(2, "$NODEID + 100 + 200"), "302");
        assert_eq!(to_value_with_node_id(1234, "$NODEID + 100 + 200"), "1534");
        assert_eq!(to_value_with_node_id(2, "No arithmetic here"), "0");
    }

    #[test]
    fn test_parse_number_i8() {
        assert_eq!(parse_number::<i8>("0xFF"), -1);
        assert_eq!(parse_number::<i8>("0x7F"), 127);
        assert_eq!(parse_number::<i8>("-128"), -128);
        assert_eq!(parse_number::<i8>("0"), 0);
        assert_eq!(parse_number::<i8>("0xAB"), -85); // 0xAB in two's complement for i8 is -85
        assert_eq!(parse_number::<i8>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_parse_number_u8() {
        assert_eq!(parse_number::<u8>("0xFF"), 255);
        assert_eq!(parse_number::<u8>("0"), 0);
        assert_eq!(parse_number::<u8>("255"), 255);
        assert_eq!(parse_number::<u8>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_parse_number_i32() {
        assert_eq!(parse_number::<i32>("0x7FFFFFFF"), 2_147_483_647);
        assert_eq!(parse_number::<i32>("-2147483648"), -2_147_483_648);
        assert_eq!(parse_number::<i32>("0"), 0);
        assert_eq!(parse_number::<i32>("abc"), 0); // Invalid input returns default
    }

    #[test]
    fn test_parse_number_u32() {
        assert_eq!(parse_number::<u32>("0xFFFFFFFF"), 4_294_967_295);
        assert_eq!(parse_number::<u32>("0"), 0);
        assert_eq!(parse_number::<u32>("4294967295"), 4_294_967_295);
        assert_eq!(parse_number::<u32>("abc"), 0); // Invalid input returns default
    }
}
