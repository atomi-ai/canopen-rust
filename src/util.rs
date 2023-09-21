use regex::Regex;
use socketcan::{CanFrame, EmbeddedFrame, Id};

pub trait ParseRadix: std::str::FromStr {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

macro_rules! impl_parse_radix_for {
    ($t:ty) => {
        impl ParseRadix for $t {
            fn from_str_radix(
                s: &str,
                radix: u32,
            ) -> Result<Self, <Self as std::str::FromStr>::Err> {
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
    let regex = Regex::new(r"\$NODEID").unwrap();
    let modified_expression = regex
        .replace_all(expression, &node_id.to_string())
        .to_string();

    // Evaluate simple arithmetic expressions
    let value_sum: i64 = modified_expression
        .split('+')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .sum();

    // Convert the evaluated sum to a string
    value_sum.to_string()
}

pub fn get_standard_can_id_from_frame(frame: &CanFrame) -> Option<u16> {
    if let Id::Standard(sid) = frame.id() {
        return Some(sid.as_raw());
    }
    None
}
