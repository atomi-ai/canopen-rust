use socketcan::Frame;

pub const INTERFACE_NAME: &str = "vcan0";

pub fn frame_to_string<F: Frame>(frame: &F) -> String {
    let id = frame.raw_id();
    let data_string = frame
        .data()
        .iter()
        .fold(String::from(""), |a, b| format!("{} {:02x}", a, b));

    format!("{:X}  [{}] {}", id, frame.dlc(), data_string)
}
