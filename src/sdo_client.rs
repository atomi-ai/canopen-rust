// It doesn't seem to be very necessary, it's not done yet.
// Please don't use.
use crate::prelude::*;
use crate::value::Value;
use crate::{util, xprintln};
use embedded_can::{blocking::Can, Error, Frame, StandardId};

pub struct SDOClient<F: Frame + Debug, E: Error> {
    network: Box<dyn Can<Frame = F, Error = E>>,
}

impl<F: Frame + Debug, E: Error> SDOClient<F, E> {
    pub fn new(network: Box<dyn Can<Frame = F, Error = E>>) -> SDOClient<F, E> {
        SDOClient { network }
    }
    pub fn expedited_upload(&mut self, node_id: u16, index: u16, sub_index: u8) -> Option<Value> {
        let request = Frame::new(
            StandardId::new(0x600 | node_id).unwrap(),
            &*vec![0x40, index as u8, (index >> 8) as u8, sub_index],
        )
        .expect("[sdo client] Failed to create CAN frame");
        self.network.transmit(&request).expect(&*format!(
            "[sdo client] Failed to send CAN frame: {:?}",
            request
        ));
        xprintln!("[sdo client] sent a frame: {:?}", request);

        loop {
            xprintln!("[sdo client] start to read a frame");
            let frame = self
                .network
                .receive()
                .expect("[sdo client] Failed to read CAN frame");
            xprintln!("[client] got frame: {:?}", frame);
            let sid = util::get_standard_can_id_from_frame(&frame).unwrap();
            let (idx, sub_idx) = util::get_index_from_can_frame(&frame);
            if sid == node_id | 0x580 && index == idx && sub_index == sub_idx {
                return self.parse_response(&frame);
            }
        }
    }

    fn parse_response(&self, frame: &F) -> Option<Value> {
        let len = 4 - ((frame.data()[0] >> 2) & 0b11) as usize;
        Some(Value {
            data: frame.data()[4..(4 + len)].to_vec(),
        })
    }
}
