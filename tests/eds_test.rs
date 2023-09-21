mod testing;

#[cfg(test)]
mod tests {
    use canopen_rust::object_directory as od;
    use canopen_rust::ObjectDirectory;
    use lazy_static::lazy_static;
    use std::panic;

    lazy_static! {
        static ref OD: ObjectDirectory = {
            use crate::testing::util as tu;
            let content = std::fs::read_to_string(tu::EDS_PATH).expect("Failed to read EDS file");
            ObjectDirectory::new(2, &content)
        };
    }

    #[test]
    fn test_variable() {
        // for debug
        panic::set_hook(Box::new(|info| {
            eprintln!("custom panic handler: {:?}", info);
        }));

        let var = OD
            .get_variable_by_name("Producer heartbeat time")
            .expect("Variable not found");

        // 对于C++中的属性检查，我们在Rust中使用assert_eq!宏
        assert_eq!(var.index, 0x1017);
        assert_eq!(var.subindex, 0);
        assert_eq!(var.name, "Producer heartbeat time");
        assert_eq!(var.data_type, od::DataType::UNSIGNED32);
        assert_eq!(var.access_type, "rw");
        assert_eq!(var.default_value.to::<u32>(), 0x12345678);
    }

    // #[test]
    // fn test_calculate_with_node_id() {
    //     assert_eq!(
    //         Data::INTEGER32(102),
    //         to_int_with_node_id(2, Data::INTEGER32, "$NODEID+100")
    //     );
    //     // ... rest of your assertions ...
    // }

    // Additional helper functions like to_int_with_node_id should also be defined here.
    // ...
}
