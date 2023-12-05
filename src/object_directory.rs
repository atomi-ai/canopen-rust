use alloc::borrow::ToOwned;

use ini_core as ini;

use crate::{info, util};
use crate::data_type::DataType;
use crate::error::CanAbortCode;
use crate::prelude::*;
use crate::value::{ByteConvertible, get_value, Value};

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

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "rw" => Ok(AccessType::new(true, true)),
            "ro" => Ok(AccessType::new(true, false)),
            "wo" => Ok(AccessType::new(false, true)),
            _ => Ok(AccessType::new(false, false)),
        }
    }

    pub fn is_readable(&self) -> bool { self.read_access }
    pub fn is_writable(&self) -> bool {
        self.write_access
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Variable {
    name: String,
    storage_location: String,
    data_type: DataType,
    default_value: Value,
    min: Option<Value>,
    max: Option<Value>,
    pdo_mappable: bool,
    access_type: AccessType,
    parameter_value: Option<Value>,
    index: u16,
    sub_index: u8,
}

impl Variable {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    pub fn default_value(&self) -> &Value {
        &self.default_value
    }
    pub fn min(&self) -> &Option<Value> {
        &self.min
    }
    pub fn max(&self) -> &Option<Value> {
        &self.max
    }
    pub fn access_type(&self) -> &AccessType {
        &self.access_type
    }
    pub fn index(&self) -> u16 {
        self.index
    }
    pub fn sub_index(&self) -> u8 {
        self.sub_index
    }
    pub fn pdo_mappable(&self) -> bool {
        self.pdo_mappable
    }
}

fn add_member_to_container(name_to_index: &mut HashMap<String, u8>, index_to_variable: &mut HashMap<u8, Variable>, var: Variable) {
    name_to_index.insert(var.name.clone(), var.sub_index);
    index_to_variable.insert(var.sub_index, var);
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Array {
    name: String,
    index: u16,
    storage_location: String,
    index_to_variable: HashMap<u8, Variable>,
    name_to_index: HashMap<String, u8>,
}

impl Array {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn index(&self) -> u16 {
        self.index
    }
}

impl Array {
    pub fn add_member(&mut self, var: Variable) {
        add_member_to_container(&mut self.name_to_index, &mut self.index_to_variable, var);
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
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Record {
    name: String,
    index: u16,
    storage_location: String,
    index_to_variable: HashMap<u8, Variable>,
    name_to_index: HashMap<String, u8>,
}

impl Record {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn index(&self) -> u16 {
        self.index
    }
    pub fn index_to_variable(&self) -> &HashMap<u8, Variable> {
        &self.index_to_variable
    }
    pub fn name_to_index(&self) -> &HashMap<String, u8> {
        &self.name_to_index
    }
}

impl Record {
    pub fn add_member(&mut self, var: Variable) {
        add_member_to_container(&mut self.name_to_index, &mut self.index_to_variable, var);
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct ObjectDirectory {
    node_id: u8,
    index_to_object: HashMap<u16, ObjectType>,
    name_to_index: HashMap<String, u16>,
}

impl ObjectDirectory {
    pub fn new(node_id: u8, eds_content: &str) -> Result<Self, String> {
        let mut od = ObjectDirectory {
            node_id,
            index_to_object: HashMap::new(),
            name_to_index: HashMap::new(),
        };
        od.load_from_content(eds_content)?;
        Ok(od)
    }

    pub fn node_id(&self) -> u8 {
        self.node_id
    }
}

impl ObjectDirectory {
    pub fn add_member(&mut self, index: u16, name: String, obj: ObjectType) {
        self.index_to_object.insert(index, obj);
        self.name_to_index.insert(name, index);
    }

    pub fn add_sub_member(&mut self, index: u16, var: Variable) -> Result<(), String> {
        match self.index_to_object.get_mut(&index) {
            None => { Err(format!("No id:{:x?}", index)) }
            Some(ObjectType::Record(record)) => { Ok(record.add_member(var)) }
            Some(ObjectType::Array(array)) => { Ok(array.add_member(var)) }
            _ => { Err(format!("no subindex for a Variable object")) }
        }
    }

    pub fn set_value_with_fitting_size(&mut self, index: u16, sub_index: u8, data: &[u8]) {
        match self.get_mut_variable(index, sub_index) {
            Err(_) => {}
            Ok(var) => {
                if !var.access_type.is_writable() {
                    return;
                }
                if var.data_type.size() > data.len() {
                    return;
                }
                var.default_value.set_data(data[0..var.data_type.size()].to_vec());
                // info!("set_value_with_fitting_size(), var = {:#x?}", var);
            }
        }
    }

    pub fn set_value(
        &mut self,
        index: u16,
        sub_index: u8,
        data: &[u8],
        ignore_access_check: bool,
    ) -> Result<&Variable, CanAbortCode> {
        match self.get_mut_variable(index, sub_index) {
            Err(code) => Err(code),
            Ok(var) => {
                if !ignore_access_check && !var.access_type.is_writable() {
                    return Err(CanAbortCode::AttemptToWriteReadOnlyObject);
                }

                if var.data_type.size() != data.len() {
                    info!("set_value() error: expect data_type size = {}, input data len = {}, data: {:?}",
                        var.data_type.size(), data.len(), data);
                    if var.data_type.size() > data.len() {
                        return Err(CanAbortCode::DataTypeMismatchLengthTooLow);
                    } else {
                        return Err(CanAbortCode::DataTypeMismatchLengthTooHigh);
                    }
                }

                // // check data type
                // info!(
                //     "xfguo: before set value, index: {} current value: {:?}",
                //     index,
                //     var
                // );
                var.default_value.set_data(data.to_vec());
                // info!(
                //     "xfguo: after set: get current value: {:?}",
                //     self.index_to_object.get(&index)
                // );
                Ok(var)
            }
        }
    }

    pub fn get_variable(&mut self, index: u16, sub_index: u8) -> Result<&Variable, CanAbortCode> {
        match self.get_mut_variable(index, sub_index) {
            Ok(var) => {
                if !var.access_type.is_readable() {
                    return Err(CanAbortCode::AttemptToReadWriteOnlyObject);
                }
                // info!("xfguo: get var: {:?}", var);
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
            let name = properties.get("ParameterName").ok_or_else(
                || format!("No 'ParameterName' in section <{}>", section_name))?;
            let ot = util::parse_number(properties.get("ObjectType").ok_or_else(
                || format!("No 'ObjectType' in section <{}>", section_name))?);
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
                        storage_location: properties.get("StorageLocation")
                            .unwrap_or(&String::from("")).to_owned(),
                        index_to_variable: HashMap::new(),
                        name_to_index: HashMap::new(),
                    };

                    if properties.contains_key("CompactSubObj") {
                        let last_subindex = Variable {
                            name: "Number of entries".to_string(),
                            index,
                            sub_index: 0,
                            data_type: DataType::Unsigned8,
                            default_value: Value::new(0u32.to_bytes()),
                            min: None,
                            max: None,
                            pdo_mappable: false,
                            access_type: AccessType::new(false, false),
                            storage_location: "".to_string(),
                            parameter_value: None,
                        };
                        array.add_member(last_subindex);
                        array.add_member(
                            build_variable(properties, self.node_id, name, index, Some(1u8))?
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
            let name = properties.get("ParameterName").ok_or_else(
                || format!("No name in section <{}>", section_name))?;
            let variable = build_variable(properties, self.node_id, name, index, Some(sub_index))?;
            self.add_sub_member(index, variable)?;
        } else if let Some(index) = util::is_name(section_name) {
            // Logic related to CompactSubObj
            let t = properties.get("NrOfEntries").ok_or_else(
                || format!("No NrOfEntries in section <{}>", section_name))?;
            let num_of_entries = t.parse().or_else(
                |err| Err(format!("Errors in parsing '{}' in section <{}>, err: {:?}",
                                  t, section_name, err)))?;
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

    pub fn load_from_content(&mut self, content: &str) -> Result<(), String> {
        let mut current_section_name: Option<String> = None;
        let mut current_properties: HashMap<String, String> = HashMap::new();

        for item in ini::Parser::new(content) {
            match item {
                ini::Item::Section(name) => {
                    if let Some(section_name) = current_section_name.take() {
                        // Get all properties, process the section.
                        self.process_section(&section_name, &current_properties)?;
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
            self.process_section(&section_name, &current_properties)?
        }

        Ok(())
    }
}

fn build_variable(
    properties: &HashMap<String, String>,
    node_id: u8,
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
    )?;
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

    let default_value = get_value(&properties, "DefaultValue", node_id, &dt).unwrap_or(
        Value::new(dt.default_value()));
    let parameter_value = get_value(&properties, "ParameterValue", node_id, &dt);

    let variable = Variable {
        name: name.clone(),
        storage_location,
        data_type: dt,
        access_type,
        pdo_mappable: pdo_mapping,
        min,
        max,
        default_value,
        parameter_value,
        index,
        sub_index: sub_index.unwrap_or(0),
    };

    Ok(variable)
}
