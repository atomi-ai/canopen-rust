use core::sync::atomic::{AtomicBool, Ordering};
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};

pub struct Node {
    socket: CanSocket,
    is_running: AtomicBool,
}

impl Node {
    pub fn new(interface: &str) -> Self {
        let socket = socketcan::CanSocket::open(interface).expect("Failed to open CAN socket");

        Node {
            socket: socket,
            is_running: AtomicBool::new(false),
        }
    }

    pub fn run(&self) {
        let ready_frame = CanFrame::new(StandardId::new(0x234).unwrap(), &[1, 2, 3, 5]).expect("");
        self.socket
            .write_frame(&ready_frame)
            .expect("Failed to send CAN frame");
        self.is_running.store(true, Ordering::Relaxed);
        loop {
            use crate::sleep;
            sleep(10);
            // infinite request / response loop.
        }
    }

    pub fn start_and_wait_until_ready(&self) {
        while !self.is_running.load(Ordering::Relaxed) {}
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
