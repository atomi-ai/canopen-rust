use embedded_can::{blocking::Can, Error, Frame, Id, StandardId};

use crate::object_directory::ObjectDirectory;
use crate::prelude::*;
use crate::sdo_server::SdoState;
use crate::sdo_server::SdoState::Normal;
use crate::util::get_standard_can_id_from_frame;
use crate::xprintln;

const DEFAULT_BLOCK_SIZE: u8 = 0x7F;

pub struct Node<F: Frame + Debug, E: Error> {
    pub(crate) node_id: u16,
    pub(crate) can_network: Box<dyn Can<Frame = F, Error = E>>,
    pub(crate) object_directory: ObjectDirectory,

    // SDO specific data below:
    pub(crate) sdo_state: SdoState,
    // TODO(zephyr): Let's use &Vec<u8> instead. 这个需要重点思考下。
    pub(crate) read_buf: Option<Vec<u8>>,
    pub(crate) read_buf_index: usize,
    pub(crate) next_read_toggle: u8,
    pub(crate) write_buf: Option<Vec<u8>>,
    pub(crate) reserved_index: u16,
    pub(crate) reserved_sub_index: u8,
    pub(crate) write_data_size: u32,
    pub(crate) need_crc: bool,
    pub(crate) block_size: u8, // sequences_per_block?
    pub(crate) current_seq_number: u8,
    pub(crate) crc_enabled: bool,
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
            sdo_state: Normal,
            read_buf: None,
            read_buf_index: 0,
            write_buf: None,
            reserved_index: 0,
            reserved_sub_index: 0,
            write_data_size: 0,
            need_crc: false,
            block_size: DEFAULT_BLOCK_SIZE,
            current_seq_number: 0,
            next_read_toggle: 0,
            crc_enabled: true,
        }
    }

    pub fn communication_object_dispatch(&mut self, frame: F) -> Option<F> {
        let sid = get_standard_can_id_from_frame(&frame).unwrap();
        if sid & 0x7F != self.node_id {
            // ignore, not my packet.
            return None;
        }
        match sid & 0xFF80 {
            0x600 => Some(self.dispatch_sdo_request(&frame)),
            _ => None,
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
            xprintln!("[node] wait for a frame");
            let frame = self
                .can_network
                .receive()
                .expect("Failed to read CAN frame");
            xprintln!("[node] got frame: {:?}", frame);

            if let Some(response) = self.communication_object_dispatch(frame) {
                if let Id::Standard(sid) = response.id() {
                    if sid.as_raw() == 0 {
                        // Don't need to send any reply for empty frame.
                        continue;
                    }
                }
                xprintln!("[node] to send reply : {:?}", response);
                self.can_network
                    .transmit(&response)
                    .expect("Failed to send CAN frame");
                xprintln!("[node] sent a frame : {:?}", response);
            }
        }
    }
}
