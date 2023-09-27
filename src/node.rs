use crate::object_directory::ObjectDirectory;
use crate::prelude::*;
use crate::{util, xprintln};

use crate::error::CanAbortCode;
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

    fn create_error_frame(&self, error_code: CanAbortCode, index: u16, sub_index: u8) -> F {
        let mut packet = Vec::new();
        packet.push(0x80);
        packet.push((index & 0xFF) as u8);
        packet.push((index >> 8) as u8);
        packet.push(sub_index);
        let code_bytes = error_code.code().to_le_bytes();
        packet.extend_from_slice(&code_bytes);
        Frame::new(
            StandardId::new(0x580 | self.node_id).unwrap(),
            packet.as_slice(),
        )
        .unwrap()
    }

    fn process_sdo_expedite_upload(
        &mut self,
        index: u16,
        sub_index: u8,
    ) -> Result<F, CanAbortCode> {
        let var = self.object_directory.get_variable(index, sub_index)?;
        generate_frame(
            0x580 | self.node_id,
            0x43,
            index,
            sub_index,
            &var.default_value.data,
        )
    }

    fn process_sdo_expedite_downad(
        &mut self,
        index: u16,
        sub_index: u8,
        req: &F,
    ) -> Result<F, CanAbortCode> {
        let len = 4 - (&req.data()[0] >> 2 & 0x3);
        match self
            .object_directory
            .set_value(index, sub_index, &req.data()[4..(4 + len as usize)])
        {
            Err(code) => Err(code),
            Ok(_) => generate_frame(
                0x580 | self.node_id,
                0x60,
                index,
                sub_index,
                &vec![0, 0, 0, 0],
            ),
        }
    }

    fn command_dispatch(&mut self, frame: &F) -> F {
        let cmd = frame.data()[0];
        let index = u16::from_le_bytes([frame.data()[1], frame.data()[2]]);
        let sub_index = frame.data()[3];
        match cmd >> 5 {
            0x1 => {
                // download (or write)
                match self.process_sdo_expedite_downad(index, sub_index, frame) {
                    Ok(frame) => frame,
                    Err(code) => self.create_error_frame(code, index, sub_index),
                }
            }
            0x2 => {
                // upload (or read)
                match self.process_sdo_expedite_upload(index, sub_index) {
                    Ok(response_frame) => response_frame,
                    Err(error_code) => self.create_error_frame(error_code, index, sub_index),
                }
            }
            _ => self.create_error_frame(
                CanAbortCode::CommandSpecifierNotValidOrUnknown,
                index,
                sub_index,
            ),
        }
    }

    pub fn communication_object_dispatch(&mut self, frame: F) -> Option<F> {
        let sid = util::get_standard_can_id_from_frame(&frame).unwrap();
        if sid & 0x7F != self.node_id {
            // ignore, not my packet.
            return None;
        }
        match sid & 0xFF80 {
            0x600 => Some(self.command_dispatch(&frame)),
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
                xprintln!("[node] to send reply : {:?}", response);
                self.can_network
                    .transmit(&response)
                    .expect("Failed to send CAN frame");
                xprintln!("[node] sent a frame : {:?}", response);
            }
            sleep(10);
        }
    }
}

fn generate_frame<F: Frame + Debug>(
    can_id: u16,
    base_cmd: u8,
    index: u16,
    sub_index: u8,
    data: &Vec<u8>,
) -> Result<F, CanAbortCode> {
    // Verify the data length
    if data.len() > 4 {
        todo!();
        // return Err(CanAbortCode::DataTransferOrStoreFailed);
    }

    let mut packet = Vec::new();
    packet.push(base_cmd | ((4 - data.len() as u8) << 2));
    packet.push((index & 0xff) as u8);
    packet.push((index >> 8) as u8);
    packet.push(sub_index);
    packet.extend_from_slice(data.as_slice());
    // padding data with zeros to 8-byte.
    while packet.len() < 8 {
        packet.push(0);
    }

    let response = Frame::new(StandardId::new(can_id).unwrap(), packet.as_slice())
        .ok_or(CanAbortCode::GeneralError)?;

    Ok(response)
}
