use crate::object_directory::ObjectDirectory;
use crate::prelude::*;
use crate::{util, xprintln};

use embedded_can::{blocking::Can, Error, Frame, StandardId};

pub struct Node<F: Frame + Debug, E: Error> {
    node_id: u16,
    can_network: Box<dyn Can<Frame = F, Error = E>>,
    object_directory: ObjectDirectory,
}

impl<F: Frame + Debug, E: Error> Node<F, E> {
    pub fn new(
        node_id: u16,
        eds_content: &str,
        can_network: Box<dyn Can<Frame = F, Error = E>>,
    ) -> Self {
        let object_directory = ObjectDirectory::new(node_id, eds_content);
        Node {
            node_id,
            can_network,
            object_directory,
        }
    }

    fn process_sdo_expedite_upload(&mut self, node_id: u16, frame: &F) {
        let index = u16::from_le_bytes([frame.data()[1], frame.data()[2]]);
        let subindex = frame.data()[3];

        let var = self.object_directory.get_variable(index, subindex);

        // Craft a response. This will be much more nuanced in real applications.
        let response = Frame::new(
            StandardId::new(0x580 | node_id).unwrap(),
            var.unwrap().to_packet(0x43).as_slice(),
        )
        .expect("Failed to create CAN frame");

        self.can_network
            .transmit(&response)
            .expect("Failed to send CAN frame");
        xprintln!("[node] sent a frame : {:?}", frame);
    }

    fn process_sdo_request(&mut self, node_id: u16, frame: &F) {
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

    pub fn process_one_frame(&mut self) {
        xprintln!("[node] wait for a frame");
        let frame = self
            .can_network
            .receive()
            .expect("Failed to read CAN frame");
        xprintln!("[node] got frame: {:?}", frame);
        let sid = util::get_standard_can_id_from_frame(&frame).unwrap();
        if sid & 0x7F != self.node_id {
            // ignore, not my packet.
            return;
        }
        match sid & 0xFF80 {
            0x600 => {
                self.process_sdo_request(self.node_id, &frame);
            }
            _ => {
                // TODO(zephyr): raise an error for unsupported requests.
            }
        }
    }

    pub fn init(&mut self) {
        let ready_frame = Frame::new(StandardId::new(0x234).unwrap(), &[1, 2, 3, 5]).expect("");
        self.can_network
            .transmit(&ready_frame)
            .expect("Failed to send CAN frame");
    }

    pub fn run(&mut self) {
        loop {
            self.process_one_frame();
            sleep(10);
        }
    }
}
