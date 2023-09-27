mod testing;

#[cfg(test)]
mod eds_tests {
    use canopen::data_type::DataType;
    use canopen::object_directory::{
        obj_to_record, obj_to_variable, AccessType, ObjectDirectory, ObjectType,
    };
    use lazy_static::lazy_static;
    use std::panic;
    use std::sync::Mutex;

    lazy_static! {
        static ref EDS_DATA: Mutex<String> = {
            use crate::testing::util as tu;
            Mutex::new(std::fs::read_to_string(tu::EDS_PATH).expect("Failed to read EDS file"))
            // Mutex::new(ObjectDirectory::new(2, &content))
        };
    }

    #[test]
    fn test_variable() {
        // for debug
        panic::set_hook(Box::new(|info| {
            eprintln!("custom panic handler: {:?}", info);
        }));
        let od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let var = obj_to_variable(
            od.get_object_by_name("Producer heartbeat time")
                .expect("Object not found"),
        )
        .expect("Not a variable");

        // 对于C++中的属性检查，我们在Rust中使用assert_eq!宏
        assert_eq!(var.index, 0x1017);
        assert_eq!(var.sub_index, 0);
        assert_eq!(var.name, "Producer heartbeat time");
        assert_eq!(var.data_type, DataType::Unsigned32);
        assert_eq!(var.access_type, AccessType::new(true, true));
        assert_eq!(var.default_value.to::<u32>(), 0x12345678);
    }

    #[test]
    fn test_relative_variable() {
        let od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let var = obj_to_record(
            od.get_object_by_name("Receive PDO 0 Communication Parameter")
                .unwrap(),
        )
        .unwrap()
        .get_variable_by_name("COB-ID use by RPDO 1")
        .expect("Expected to find the variable");
        assert_eq!(var.default_value.to::<u32>(), 512 + od.node_id as u32); // 假设default_value是u32类型
    }

    #[test]
    fn test_record() {
        let od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let obj = od
            .get_object_by_name("Identity object")
            .expect("Identity object not found");
        if let ObjectType::Record(record) = obj {
            assert_eq!(record.name, "Identity object");
            assert_eq!(record.index, 0x1018);
            assert_eq!(record.index_to_variable.len(), 5);

            let variable = record
                .name_to_index
                .get("Vendor-ID")
                .and_then(|&idx| record.index_to_variable.get(&idx))
                .expect("Variable not found");

            assert_eq!(variable.name, "Vendor-ID");
            assert_eq!(variable.index, 0x1018);
            assert_eq!(variable.sub_index, 1);
            assert_eq!(variable.data_type, DataType::Unsigned32);
            assert_eq!(variable.access_type, AccessType::new(true, false));
        } else {
            panic!("Expected a Record named 'Identity object'");
        }
    }

    #[test]
    fn test_record_with_limits() {
        let mut od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let int8 = od
            .get_variable(0x3020, 0)
            .expect("Expected to find the variable");
        assert_eq!(int8.min.as_ref().unwrap().to::<i8>(), 0);
        assert_eq!(int8.max.as_ref().unwrap().to::<i8>(), 127);

        let uint8 = od
            .get_variable(0x3021, 0)
            .expect("Expected to find the variable");
        assert_eq!(uint8.min.as_ref().unwrap().to::<u8>(), 2);
        assert_eq!(uint8.max.as_ref().unwrap().to::<u8>(), 10);

        let int32 = od
            .get_variable(0x3030, 0)
            .expect("Expected to find the variable");
        assert_eq!(int32.min.as_ref().unwrap().to::<i32>(), -1);
        assert_eq!(int32.max.as_ref().unwrap().to::<i32>(), 0);

        // 获取int64
        let int64 = od
            .get_variable(0x3040, 0)
            .expect("Expected to find the variable");
        assert_eq!(
            int64.min.as_ref().unwrap().to::<i64>(),
            -9223372036854775799
        );
        assert_eq!(int64.max.as_ref().unwrap().to::<i64>(), 10);
    }

    #[test]
    fn test_array_compact_sub_obj() {
        let mut od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());

        if let ObjectType::Array(array_obj) = od.get_mut_object(0x1003).expect("Array not found") {
            assert_eq!(array_obj.index, 0x1003);
            assert_eq!(array_obj.name, "Pre-defined error field");
        } else {
            panic!("Expect array at index 0x1003");
        }

        match od.get_variable(0x1003, 5) {
            Ok(var) => {
                assert_eq!(var.name, "Pre-defined error field_5");
                assert_eq!(var.index, 0x1003);
                assert_eq!(var.sub_index, 5);
                assert_eq!(var.data_type, DataType::Unsigned32);
                assert_eq!(var.access_type, AccessType::new(true, false));
            }
            Err(err) => panic!("Expected an Array at index 0x1003, err: {:?}", err),
        }
    }

    #[test]
    fn test_explicit_name_subobj() {
        let mut od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let array = od.get_mut_object(0x3004).expect("Array not found");

        if let ObjectType::Array(array_obj) = &array {
            assert_eq!(array_obj.name, "Sensor Status");
        } else {
            panic!("Expected an Array at index 0x3004");
        }

        match od.get_variable(0x3004, 1) {
            Ok(var) => assert_eq!(var.name, "Sensor Status 1"),
            Err(err) => panic!("Expected an Variable at (0x3004, 1), err: {:?}", err),
        }

        match od.get_variable(0x3004, 3) {
            Ok(var) => {
                assert_eq!(var.name, "Sensor Status 3");
                assert_eq!(var.default_value.to::<u16>(), 3);
            }
            Err(err) => panic!("Expected an Variable at (0x3004, 3), err: {:?}", err),
        }
    }

    #[test]
    fn test_parameter_name_with_percent() {
        let mut od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let array = od.get_mut_object(0x3003).expect("Array not found");

        if let ObjectType::Array(array_obj) = &array {
            assert_eq!(array_obj.name, "Valve % open");
        } else {
            panic!("Expected an Array at index 0x3003");
        }
    }

    #[test]
    fn test_compact_subobj_parameter_name_with_percent() {
        let mut od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());
        let array = od.get_mut_object(0x3006).expect("Array not found");

        if let ObjectType::Array(array_obj) = &array {
            assert_eq!(array_obj.name, "Valve 1 % Open");
        } else {
            panic!("Expected an Array at index 0x3006");
        }
    }

    #[test]
    fn test_sub_index_with_capital_s() {
        let mut od = ObjectDirectory::new(2, &EDS_DATA.lock().unwrap());

        match od.get_mut_object(0x3010).expect("Record not found") {
            ObjectType::Record(record_obj) => {
                let sub_obj = record_obj
                    .get_mut_variable(0)
                    .expect("Sub-object not found");
                assert_eq!(sub_obj.name, "Temperature");
            }
            _ => panic!("Expected a Record at index 0x3010"),
        }
    }
}
