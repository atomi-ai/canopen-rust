use hashbrown::HashMap;
use ini_core as ini;
use regex::Regex;

use crate::util;
use crate::xprintln;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    UNKNOWN = 0x0,
    BOOLEAN = 0x1,
    INTEGER8 = 0x2,
    INTEGER16 = 0x3,
    INTEGER32 = 0x4,
    UNSIGNED8 = 0x5,
    UNSIGNED16 = 0x6,
    UNSIGNED32 = 0x7,
    REAL32 = 0x8,
    VISIBLE_STRING = 0x9,
    OCTET_STRING = 0xA,
    UNICODE_STRING = 0xB,
    DOMAIN = 0xF,
    REAL64 = 0x11,
    INTEGER64 = 0x15,
    UNSIGNED64 = 0x1B,
}

impl DataType {
    fn from_u32(value: u32) -> Self {
        match value {
            0x0 => DataType::UNKNOWN,
            0x1 => DataType::BOOLEAN,
            0x2 => DataType::INTEGER8,
            0x3 => DataType::INTEGER16,
            0x4 => DataType::INTEGER32,
            0x5 => DataType::UNSIGNED8,
            0x6 => DataType::UNSIGNED16,
            0x7 => DataType::UNSIGNED32,
            0x8 => DataType::REAL32,
            0x9 => DataType::VISIBLE_STRING,
            0xA => DataType::OCTET_STRING,
            0xB => DataType::UNICODE_STRING,
            0xF => DataType::DOMAIN,
            0x11 => DataType::REAL64,
            0x15 => DataType::INTEGER64,
            0x1B => DataType::UNSIGNED64,
            _ => DataType::UNKNOWN,
        }
    }
}

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
                assert!(bytes.len() == $len);
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
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        String::from_utf8(bytes.to_vec()).expect("Failed to convert bytes to String")
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

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub storage_location: String,
    pub data_type: DataType,
    pub default_value: Value,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub pdo_mappable: bool,
    pub access_type: String,
    pub parameter_value: Option<Value>,
    pub index: u16,
    pub subindex: u8,
}

impl Variable {
    pub fn to_packet(&self, cmd: u8) -> Vec<u8> {
        let mut packet = Vec::new();
        let v = &self.default_value;
        let real_cmd = cmd | ((4 - v.len() as u8) << 2);
        packet.push(real_cmd);
        packet.push((self.index & 0xFF) as u8);
        packet.push((self.index >> 8) as u8);
        packet.push(self.subindex);
        packet.extend_from_slice(v.as_slice());

        packet
    }
}

#[derive(Debug)]
struct Array {
    // Array的字段
}

#[derive(Debug)]
struct Record {
    items: HashMap<u8, Variable>,
}

#[derive(Debug)]
enum ObjectType {
    Variable(Variable),
    Array(Array),
    Record(Record),
}

fn string_to_value(data_type: &DataType, data_string: &str) -> Result<Value, String> {
    match data_type {
        DataType::UNKNOWN => Err("Unknown DataType".into()),

        DataType::BOOLEAN => {
            let val: u8 = match data_string.to_lowercase().as_str() {
                "true" | "1" => 1,
                "false" | "0" => 0,
                _ => return Err("Invalid boolean value".into()),
            };
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::INTEGER8 => {
            let val: i8 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::INTEGER16 => {
            let val: i16 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::INTEGER32 => {
            let val: i32 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::INTEGER64 => {
            let val: i64 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::UNSIGNED8 => {
            let val: u8 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::UNSIGNED16 => {
            let val: u16 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::UNSIGNED32 => {
            let val: u32 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::UNSIGNED64 => {
            let val: u64 = util::parse_number(data_string);
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::REAL32 => {
            let val: f32 = data_string.parse().map_err(|_| "Failed to parse f32")?;
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::REAL64 => {
            let val: f64 = data_string.parse().map_err(|_| "Failed to parse f64")?;
            Ok(Value {
                data: val.to_bytes(),
            })
        }

        DataType::VISIBLE_STRING | DataType::OCTET_STRING | DataType::UNICODE_STRING => Ok(Value {
            data: data_string.as_bytes().to_vec(),
        }),

        DataType::DOMAIN => {
            let val: i32 = data_string
                .parse()
                .map_err(|_| "Failed to parse domain as i32")?;
            Ok(Value {
                data: val.to_bytes(),
            })
        }
    }
}

fn get_value(
    properties: &HashMap<String, String>,
    property_name: &str,
    node_id: u16,
    data_type: &DataType,
) -> Option<Value> {
    let mut raw = properties
        .get(property_name)
        .unwrap_or(&"".to_string())
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
        Err(e) => {
            // Handle the error
            eprintln!("xfguo Error: {}", e);
            None
        }
    }
}

fn build_variable(
    properties: &HashMap<String, String>,
    node_id: u16,
    index: u16,
    subindex: Option<u8>,
) -> Result<Variable, String> {
    let parameter_name = properties
        .get("ParameterName")
        .unwrap_or(&"".to_string())
        .clone();
    let storage_location = properties
        .get("StorageLocation")
        .unwrap_or(&"".to_string())
        .clone();
    let access_type = properties
        .get("AccessType")
        .unwrap_or(&"rw".to_string())
        .to_lowercase();
    let pdo_mapping = properties
        .get("PDOMapping")
        .unwrap_or(&"0".to_string())
        .parse::<i32>()
        .unwrap_or(0)
        != 0;

    let dt_val = util::parse_number(properties.get("DataType").unwrap_or(&"".to_string()));
    let dt = DataType::from_u32(dt_val);

    let min = get_value(&properties, "LowLimit", node_id, &dt);
    let max = get_value(&properties, "HighLimit", node_id, &dt);

    let default_value = get_value(&properties, "DefaultValue", node_id, &dt).unwrap();
    let parameter_value = get_value(&properties, "ParameterValue", node_id, &dt);

    let variable = Variable {
        name: parameter_name,
        storage_location: storage_location,
        data_type: dt,
        access_type: access_type,
        pdo_mappable: pdo_mapping,
        min: min,
        max: max,
        default_value: default_value,
        parameter_value: parameter_value,
        index: index,
        subindex: subindex.unwrap_or(0),
    };

    Ok(variable)
}

pub struct ObjectDirectory {
    node_id: u16,
    index_to_object: HashMap<u16, ObjectType>,
    name_to_index: HashMap<String, u16>,
}

impl ObjectDirectory {
    pub fn new(node_id: u16, eds_content: &str) -> Self {
        let mut od = ObjectDirectory {
            node_id,
            index_to_object: HashMap::new(),
            name_to_index: HashMap::new(),
        };
        od.load_from_content(eds_content)
            .expect("Failed to load EDS content");
        od
    }

    pub fn process_section(
        &mut self,
        section_name: &str,
        properties: &HashMap<String, String>,
    ) -> Result<(), String> {
        xprintln!(
            "section_name: {}, properties: {:?}",
            section_name,
            properties
        );
        let re_top = Regex::new(r"^[0-9A-Fa-f]{4}$").unwrap();
        let re_sub = Regex::new(r"^([0-9A-Fa-f]{4})[S|s]ub([0-9A-Fa-f]+)$").unwrap();
        let re_name = Regex::new(r"^([0-9A-Fa-f]{4})Name$").unwrap();

        if let Some(cap) = re_top.captures(section_name) {
            let index = u16::from_str_radix(&cap[0], 16).map_err(|_| "Invalid index")?;
            let ot = util::parse_number(properties.get("ObjectType").unwrap_or(&"0".to_string()));
            match (ot) {
                7 => {
                    if index == 0x1017 {
                        let variable =
                            build_variable(properties, self.node_id, index as u16, None)?;
                        self.name_to_index.insert(variable.name.clone(), index);
                        self.index_to_object
                            .insert(index, ObjectType::Variable(variable));
                    }
                }
                8 => {
                    // 这里处理Array的创建
                    let array = Array { /* 初始化字段 */ };
                    self.index_to_object.insert(index, ObjectType::Array(array));
                }
                9 => {
                    // 这里处理Record的创建
                    let record = Record {
                        items: HashMap::new(),
                    };
                    self.index_to_object
                        .insert(index, ObjectType::Record(record));
                }
                _ => { // ignore
                }
            }
        } else if let Some(cap) = re_sub.captures(section_name) {
            // let index = u16::from_str_radix(&cap[1], 16).map_err(|_| "Invalid index")?;
            // let sub_index = u8::from_str_radix(&cap[2], 10).map_err(|_| "Invalid sub index")?;

            // // 处理子Variable的创建
            // let variable = Variable { /* 初始化字段 */ };

            // if let Some(obj_type) = self.mm.get_mut(&index) {
            //     if let ObjectType::Record(record) = obj_type {
            //         record.items.insert(sub_index, variable);
            //     } else {
            //         return Err("Expected a Record for a SubVariable".into());
            //     }
            // }
        } else if re_name.is_match(section_name) {
            // 处理与CompactSubObj相对应的Array的逻辑
        }

        Ok(())
    }

    pub fn load_from_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_section_name: Option<String> = None;
        let mut current_properties: HashMap<String, String> = HashMap::new();

        for item in ini::Parser::new(content) {
            match item {
                ini::Item::Section(name) => {
                    // 如果存在当前 section，处理它
                    if let Some(section_name) = current_section_name.take() {
                        self.process_section(&section_name, &current_properties);
                        current_properties.clear();
                    }
                    current_section_name = Some(name.to_string());
                }
                ini::Item::Property(key, maybe_value) => {
                    let value = maybe_value.unwrap_or_default().to_string();
                    current_properties.insert(key.to_string(), value);
                }
                _ => {} // 对于其他条目，例如 comments 或 section end，我们不做处理。
            }
        }

        // 处理最后一个 section
        if let Some(section_name) = current_section_name {
            self.process_section(&section_name, &current_properties);
        }

        xprintln!(
            "Successfully load eds content, index map: {:?}",
            self.index_to_object
        );

        Ok(())
    }

    pub fn get_varible(&self, index: u16, sub_index: u8) -> Option<&Variable> {
        match self.index_to_object.get(&index) {
            Some(ObjectType::Variable(var)) => Some(var),
            _ => {
                // TOOD(zephyr): Raise error, not support yet.
                None
            }
        }
    }

    pub fn get_variable_by_name(&self, name: &str) -> Option<&Variable> {
        if let Some(id) = self.name_to_index.get(name) {
            if let Some(ObjectType::Variable(var)) = self.index_to_object.get(id) {
                xprintln!("Found variable: {:?}", var);
                return Some(var);
            }
        }
        None
    }
}
