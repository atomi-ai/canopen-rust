use byteorder::{ByteOrder, LittleEndian};
use canopen::{object_directory::Value, util};
use socketcan::Id::Standard;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};

pub trait CANNetwork: 'static {
    fn send_message(&mut self, can_header: u16, data: &[u8]);
    fn recv_message(&mut self) -> CanFrame;
}

impl CANNetwork for CanSocket {
    fn send_message(&mut self, can_header: u16, data: &[u8]) {
        let frame = CanFrame::new(StandardId::new(can_header).unwrap(), data).expect(&format!(
            "[socketcan] Failed to create CAN frame: {} | {:?}",
            can_header, data
        ));
        self.write_frame(&frame).expect(&format!(
            "[socketcan] Failed to write a frame: {} | {:?}",
            can_header, data
        ));
    }

    fn recv_message(&mut self) -> CanFrame {
        self.read_frame().expect("[socketcan] Failed to read frame")
    }
}

pub struct SDOClient {
    network: Box<dyn CANNetwork>,
    node_id: u32,
}

impl SDOClient {
    pub fn new(interface: &str, node_id: u32) -> Self {
        SDOClient {
            network: Box::new(
                socketcan::CanSocket::open(interface).expect("Failed to open CAN socket"),
            ),
            node_id,
        }
    }

    pub fn expedited_upload(&mut self, node_id: u16, index: u16, sub_index: u8) -> Option<Value> {
        let request = vec![0x40, index as u8, (index >> 8) as u8, sub_index];

        self.network.send_message(node_id + 0x600, &request);

        loop {
            let frame = self.network.recv_message();
            let sid = util::get_standard_can_id_from_frame(&frame).unwrap();
            if sid == node_id + 0x580 {
                return self.parse_response(&frame, index, sub_index);
            }
        }
    }

    fn parse_response(&self, frame: &CanFrame, index: u16, sub_index: u8) -> Option<Value> {
        let len = 4 - ((frame.data()[0] >> 2) & 0b11) as usize;
        assert_eq!(LittleEndian::read_u16(&frame.data()[1..3]), index);
        assert_eq!(frame.data()[3], sub_index);
        Some(Value {
            data: frame.data()[4..(4 + len)].to_vec(),
        })
    }
}
