use crate::prelude::*;

use core::str::FromStr;
use embedded_can::{Frame, Id};

pub trait ParseRadix: FromStr {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

macro_rules! impl_parse_radix_for {
    ($t:ty) => {
        impl ParseRadix for $t {
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, <Self as FromStr>::Err> {
                <$t>::from_str_radix(s, radix)
            }
        }
    };
}

// 使用宏为每种类型实现 ParseRadix trait
impl_parse_radix_for!(i8);
impl_parse_radix_for!(i16);
impl_parse_radix_for!(i32);
impl_parse_radix_for!(i64);
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

pub fn is_sub(s: &str) -> Option<(&str, &str)> {
    if s.len() > 7 && s[4..7].eq_ignore_ascii_case("sub") && s[0..4].chars().all(is_hex_char) {
        Some((&s[0..4], &s[7..]))
    } else {
        None
    }
}

pub fn is_name(s: &str) -> bool {
    s.ends_with("Name") && s[0..4].chars().all(is_hex_char)
}
