use crate::{sleep, util, ObjectDirectory};
use core::sync::atomic::{AtomicBool, Ordering};
use embedded_can::{blocking::Can, Error, Frame, Id::Standard, StandardId};

pub struct Node<F, E>
where
    F: Frame,
    E: Error,
{
    node_id: u16,
    can_network: Box<dyn Can<Frame = F, Error = E>>,
    object_directory: ObjectDirectory,
}

impl<F, E> Node<F, E>
where
    F: Frame,
    E: Error,
{
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

        let var = self.object_directory.get_varible(index, subindex);

        // Craft a response. This will be much more nuanced in real applications.
        let response = Frame::new(
            StandardId::new(0x580 | node_id).unwrap(),
            var.unwrap().to_packet(0x43).as_slice(),
        )
        .expect("Failed to create CAN frame");

        self.can_network
            .transmit(&response)
            .expect("Failed to send CAN frame");
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
        let frame = self
            .can_network
            .receive()
            .expect("Failed to read CAN frame");
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
            // TODO(zephyr): not a good idea to sleep(10) mill-seconds, let's figure out another way in the future.
            sleep(10);
        }
    }
}