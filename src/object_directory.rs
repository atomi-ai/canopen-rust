use crate::data_type::DataType;
use crate::error::CanAbortCode;
use crate::prelude::*;
use crate::value::{get_value, ByteConvertible, Value};
use crate::{util, xprintln};
use ini_core as ini;

#[derive(Clone, Debug, PartialEq)]
pub struct AccessType {
    read_access: bool,
    write_access: bool,
}

impl AccessType {
    pub fn new(read: bool, write: bool) -> Self {
        AccessType {
            read_access: read,
            write_access: write,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rw" => Some(AccessType::new(true, true)),
            "ro" => Some(AccessType::new(true, false)),
            "wo" => Some(AccessType::new(false, true)),
            _ => Some(AccessType::new(false, false)),
        }
    }

    pub fn can_read(&self) -> bool {
        self.read_access
    }

    pub fn can_write(&self) -> bool {
        self.write_access
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub storage_location: String,
    pub data_type: DataType,
    pub default_value: Value,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub pdo_mappable: bool,
    pub access_type: AccessType,
    pub parameter_value: Option<Value>,
    pub index: u16,
    pub sub_index: u8,
}

impl Variable {
    pub fn to_packet(&self, base_cmd: u8) -> Vec<u8> {
        let mut packet = Vec::new();
        let v = &self.default_value;
        let real_cmd = base_cmd | ((4 - v.len() as u8) << 2);
        packet.push(real_cmd);
        packet.push((self.index & 0xFF) as u8);
        packet.push((self.index >> 8) as u8);
        packet.push(self.sub_index);
        packet.extend_from_slice(v.as_slice());

        packet
    }
}

#[derive(Debug)]
pub struct Array {
    pub name: String,
    pub index: u16,
    pub storage_location: String,
    pub index_to_variable: HashMap<u8, Variable>,
    pub name_to_index: HashMap<String, u8>,
}

impl Array {
    pub fn add_member(&mut self, var: Variable) {
        self.name_to_index.insert(var.name.clone(), var.sub_index);
        self.index_to_variable.insert(var.sub_index, var);
    }

    pub fn get_mut_variable(&mut self, sub_index: u8) -> Result<&mut Variable, CanAbortCode> {
        if self.index_to_variable.contains_key(&sub_index) {
            return self
                .index_to_variable
                .get_mut(&sub_index)
                .ok_or(CanAbortCode::ObjectDoesNotExistInObjectDictionary);
        }

        if 0 < sub_index && sub_index < 0xFF {
            // TODO(zephyr): copy from python impl, which doesn't follow the spec very well.
            // Please read <CANopen CiA 306> section 4.5.2.4 for details.
            if let Some(base_var) = self.index_to_variable.get(&1) {
                let mut new_var = base_var.clone();
                new_var.name = format!("{}_{}", self.name, sub_index);
                new_var.sub_index = sub_index;
                self.add_member(new_var);
                return self
                    .index_to_variable
                    .get_mut(&sub_index)
                    .ok_or(CanAbortCode::ObjectDoesNotExistInObjectDictionary);
            }
        }
        Err(CanAbortCode::ObjectDoesNotExistInObjectDictionary)
    }
    // pub fn get_variable_by_name(&mut self, name: &str) -> Result<&Variable, CanAbortCode> {
    //     self.get_variable(*self.name_to_index.get(name).unwrap())
    // }
}

#[derive(Debug)]
pub struct Record {
    pub name: String,
    pub index: u16,
    pub storage_location: String,
    pub index_to_variable: HashMap<u8, Variable>,
    pub name_to_index: HashMap<String, u8>,
}

impl Record {
    pub fn add_member(&mut self, var: Variable) {
        self.name_to_index.insert(var.name.clone(), var.sub_index);
        self.index_to_variable.insert(var.sub_index, var);
    }

    pub fn get_mut_variable(&mut self, sub_index: u8) -> Result<&mut Variable, CanAbortCode> {
        self.index_to_variable
            .get_mut(&sub_index)
            .ok_or(CanAbortCode::ObjectDoesNotExistInObjectDictionary)
    }

    pub fn get_variable_by_name(&self, name: &str) -> Result<&Variable, CanAbortCode> {
        if let Some(idx) = self.name_to_index.get(name) {
            let t = self.index_to_variable.get(idx);
            t.ok_or(CanAbortCode::GeneralError)
        } else {
            Err(CanAbortCode::GeneralError)
        }
    }
}

#[derive(Debug)]
pub enum ObjectType {
    Variable(Variable),
    Array(Array),
    Record(Record),
}

pub fn obj_to_variable(obj: &ObjectType) -> Option<&Variable> {
    if let ObjectType::Variable(var) = obj {
        return Some(var);
    }
    None
}

pub fn obj_to_array(obj: &ObjectType) -> Option<&Array> {
    if let ObjectType::Array(arr) = obj {
        return Some(arr);
    }
    None
}

pub fn obj_to_record(obj: &ObjectType) -> Option<&Record> {
    if let ObjectType::Record(rec) = obj {
        return Some(rec);
    }
    None
}

pub struct ObjectDirectory {
    pub node_id: u16,
    index_to_object: HashMap<u16, ObjectType>,
    name_to_index: HashMap<String, u16>,
}

impl ObjectDirectory {}

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

    pub fn add_member(&mut self, index: u16, name: String, obj: ObjectType) {
        self.index_to_object.insert(index, obj);
        self.name_to_index.insert(name, index);
    }

    pub fn add_sub_member(&mut self, index: u16, var: Variable) {
        let obj = self.index_to_object.get_mut(&index).unwrap();
        match obj {
            ObjectType::Record(record) => {
                record.add_member(var);
            }
            ObjectType::Array(array) => {
                array.add_member(var);
            }
            ObjectType::Variable(_) => {
                panic!("no subindex for a Variable object");
            }
        }
    }

    pub fn set_value(
        &mut self,
        index: u16,
        sub_index: u8,
        data: &[u8],
    ) -> Result<(), CanAbortCode> {
        match self.get_mut_variable(index, sub_index) {
            Err(code) => Err(code),
            Ok(var) => {
                if !var.access_type.can_write() {
                    return Err(CanAbortCode::AttemptToWriteReadOnlyObject);
                }

                if var.data_type.size() != data.len() {
                    if var.data_type.size() > data.len() {
                        return Err(CanAbortCode::DataTypeMismatchLengthTooLow);
                    } else {
                        return Err(CanAbortCode::DataTypeMismatchLengthTooHigh);
                    }
                }

                // check data type
                xprintln!(
                    "xfguo: before set value, index: {} current value: {:?}",
                    index,
                    var
                );
                var.default_value.data = data.to_vec();
                xprintln!(
                    "xfguo: after set: get current value: {:?}",
                    self.index_to_object.get(&index)
                );
                Ok(())
            }
        }
    }

    pub fn get_variable(&mut self, index: u16, sub_index: u8) -> Result<&Variable, CanAbortCode> {
        match self.get_mut_variable(index, sub_index) {
            Ok(var) => {
                if !var.access_type.can_read() {
                    return Err(CanAbortCode::AttemptToReadWriteOnlyObject);
                }
                xprintln!("xfguo: get var: {:?}", var);
                Ok(var)
            }
            Err(code) => Err(code),
        }
    }

    pub fn get_mut_variable(
        &mut self,
        index: u16,
        sub_index: u8,
    ) -> Result<&mut Variable, CanAbortCode> {
        match self.index_to_object.get_mut(&index) {
            Some(ObjectType::Variable(var)) => {
                if sub_index == 0 {
                    Ok(var)
                } else {
                    Err(CanAbortCode::SubIndexDoesNotExist)
                }
            }
            Some(ObjectType::Array(arr)) => arr.get_mut_variable(sub_index),
            Some(ObjectType::Record(rec)) => rec.get_mut_variable(sub_index),
            None => Err(CanAbortCode::ObjectDoesNotExistInObjectDictionary),
        }
    }

    pub fn get_object_by_name(&self, name: &str) -> Option<&ObjectType> {
        if let Some(id) = self.name_to_index.get(name) {
            return self.index_to_object.get(id);
        }
        None
    }

    pub fn get_mut_object(&mut self, index: u16) -> Option<&mut ObjectType> {
        self.index_to_object.get_mut(&index)
    }

    pub fn process_section(
        &mut self,
        section_name: &str,
        properties: &HashMap<String, String>,
    ) -> Result<(), String> {
        if util::is_top(section_name) {
            let index = u16::from_str_radix(section_name, 16).map_err(|_| "Invalid index")?;
            let name = properties.get("ParameterName").unwrap();
            let ot = util::parse_number(properties.get("ObjectType").unwrap());
            match ot {
                7 => {
                    let variable =
                        build_variable(properties, self.node_id, name, index as u16, None)?;
                    self.name_to_index.insert(variable.name.clone(), index);
                    self.index_to_object
                        .insert(index, ObjectType::Variable(variable));
                }
                8 => {
                    let mut array = Array {
                        name: name.to_string(),
                        index,
                        storage_location: properties
                            .get("StorageLocation")
                            .unwrap_or(&String::from(""))
                            .clone(),
                        index_to_variable: HashMap::new(),
                        name_to_index: HashMap::new(),
                    };

                    if properties.contains_key("CompactSubObj") {
                        let last_subindex = Variable {
                            name: "Number of entries".to_string(),
                            index,
                            sub_index: 0,
                            data_type: DataType::Unsigned8,
                            default_value: Value {
                                data: 0u32.to_bytes(),
                            },
                            min: None,
                            max: None,
                            pdo_mappable: false,
                            access_type: AccessType::new(false, false),
                            storage_location: "".to_string(),
                            parameter_value: None,
                        };
                        array.add_member(last_subindex);
                        array.add_member(
                            build_variable(properties, self.node_id, name, index, Some(1u8))
                                .unwrap(),
                        );
                    }
                    self.add_member(index, name.clone(), ObjectType::Array(array));
                }
                9 => {
                    let record = Record {
                        name: name.clone(),
                        index,
                        storage_location: properties
                            .get("StorageLocation")
                            .unwrap_or(&String::from(""))
                            .clone(),
                        index_to_variable: HashMap::new(),
                        name_to_index: HashMap::new(),
                    };
                    self.name_to_index.insert(name.clone(), index);
                    self.index_to_object
                        .insert(index, ObjectType::Record(record));
                }
                _ => { // ignore
                }
            }
        } else if let Some((index, sub_index)) = util::is_sub(section_name) {
            let name = properties.get("ParameterName").unwrap();
            let variable = build_variable(properties, self.node_id, name, index, Some(sub_index))?;
            self.add_sub_member(index, variable);
        } else if let Some(index) = util::is_name(section_name) {
            // Logic related to CompactSubObj
            let num_of_entries: u8 = properties.get("NrOfEntries").unwrap().parse().unwrap();
            if let Some(ObjectType::Array(arr)) = self.index_to_object.get_mut(&index) {
                if let Some(src_var) = arr.index_to_variable.get(&1u8) {
                    let cloned_src_var = src_var.clone();
                    let mut new_vars = Vec::new();
                    for subindex in 1..=num_of_entries {
                        let mut var = cloned_src_var.clone();
                        if let Some(name) = properties.get(&subindex.to_string()) {
                            var.name = name.clone();
                            var.sub_index = subindex;
                            new_vars.push(var);
                        }
                    }
                    for var in new_vars {
                        arr.add_member(var);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn load_from_content(&mut self, content: &str) -> Result<(), Error> {
        let mut current_section_name: Option<String> = None;
        let mut current_properties: HashMap<String, String> = HashMap::new();

        for item in ini::Parser::new(content) {
            match item {
                ini::Item::Section(name) => {
                    if let Some(section_name) = current_section_name.take() {
                        self.process_section(&section_name, &current_properties)
                            .expect(section_name.as_str());
                        current_properties.clear();
                    }
                    current_section_name = Some(String::from(name));
                }
                ini::Item::Property(key, maybe_value) => {
                    let value = String::from(maybe_value.unwrap_or_default());
                    current_properties.insert(String::from(key), value);
                }
                _ => {} // 对于其他条目，例如 comments 或 section end，我们不做处理。
            }
        }

        // 处理最后一个 section
        if let Some(section_name) = current_section_name {
            self.process_section(&section_name, &current_properties)
                .expect(section_name.as_str());
        }

        Ok(())
    }
}

fn build_variable(
    properties: &HashMap<String, String>,
    node_id: u16,
    name: &String,
    index: u16,
    sub_index: Option<u8>,
) -> Result<Variable, String> {
    let storage_location = properties
        .get("StorageLocation")
        .unwrap_or(&String::from(""))
        .clone();
    let access_type = AccessType::from_str(
        &*properties
            .get("AccessType")
            .unwrap_or(&String::from("rw"))
            .to_lowercase(),
    );
    let pdo_mapping = properties
        .get("PDOMapping")
        .unwrap_or(&String::from("0"))
        .parse::<i32>()
        .unwrap_or(0)
        != 0;

    let dt_val = util::parse_number(
        properties
            .get(&String::from("DataType"))
            .unwrap_or(&String::from("")),
    );
    let dt = DataType::from_u32(dt_val);

    let min = get_value(&properties, "LowLimit", node_id, &dt);
    let max = get_value(&properties, "HighLimit", node_id, &dt);

    let default_value = get_value(&properties, "DefaultValue", node_id, &dt).unwrap_or(Value {
        data: dt.default_value(),
    });
    if index == 0x1200 && sub_index == Some(1) {
        xprintln!("xfguo: debug");
    }
    let parameter_value = get_value(&properties, "ParameterValue", node_id, &dt);

    let variable = Variable {
        name: name.clone(),
        storage_location: storage_location,
        data_type: dt,
        access_type: access_type.unwrap(),
        pdo_mappable: pdo_mapping,
        min: min,
        max: max,
        default_value: default_value,
        parameter_value: parameter_value,
        index: index,
        sub_index: sub_index.unwrap_or(0),
    };

    Ok(variable)
}
