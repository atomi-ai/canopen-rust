use crate::{sleep, util, ObjectDirectory};
use core::sync::atomic::{AtomicBool, Ordering};
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};

pub struct Node {
    node_id: u16,
    socket: CanSocket,
    is_running: AtomicBool,
    object_directory: ObjectDirectory,
}

impl Node {
    pub fn new(interface: &str, node_id: u16, eds_content: &str) -> Self {
        let socket = socketcan::CanSocket::open(interface).expect("Failed to open CAN socket");
        let object_directory = ObjectDirectory::new(node_id, eds_content);

        Node {
            node_id,
            socket,
            object_directory,
            is_running: AtomicBool::new(false),
        }
    }

    fn process_sdo_expedite_upload(&self, node_id: u16, frame: CanFrame) {
        let index = u16::from_le_bytes([frame.data()[1], frame.data()[2]]);
        let subindex = frame.data()[3];

        let var = self.object_directory.get_varible(index, subindex);

        // Craft a response. This will be much more nuanced in real applications.
        let response = CanFrame::new(
            StandardId::new(0x580 | node_id).unwrap(),
            var.unwrap().to_packet(0x43).as_slice(),
        )
        .expect("Failed to create CAN frame");

        self.socket
            .write_frame(&response)
            .expect("Failed to send CAN frame");
    }

    fn process_sdo_request(&self, node_id: u16, frame: CanFrame) {
        let cmd = frame.data()[0];
        match cmd >> 5 {
            0x2 => {
                // upload
                self.process_sdo_expedite_upload(node_id, frame);
            }
            _ => {
                // TODO(zephyr): raise an error for unsupported requests.
            }
        }
    }

    pub fn process_one_frame(&self) {
        let frame = self.socket.read_frame().expect("Failed to read CAN frame");
        let sid = util::get_standard_can_id_from_frame(&frame).unwrap();
        if sid & 0x7F != self.node_id {
            // ignore, not my packet.
            return;
        }
        match sid & 0xFF80 {
            0x600 => {
                self.process_sdo_request(self.node_id, frame);
            }
            _ => {
                // TODO(zephyr): raise an error for unsupported requests.
            }
        }
    }

    pub fn run(&self) {
        let ready_frame = CanFrame::new(StandardId::new(0x234).unwrap(), &[1, 2, 3, 5]).expect("");
        self.socket
            .write_frame(&ready_frame)
            .expect("Failed to send CAN frame");
        self.is_running.store(true, Ordering::Relaxed);
        loop {
            self.process_one_frame();
            // TODO(zephyr): not a good idea to sleep(10) mill-seconds, let's figure out another way in the future.
            sleep(10);
        }
    }

    pub fn wait_until_ready(&self) {
        while !self.is_running.load(Ordering::Relaxed) {}
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
