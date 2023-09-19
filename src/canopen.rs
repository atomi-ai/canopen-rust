use socketcan::{CanFrame, EmbeddedFrame, Socket, StandardId};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

pub struct Node {
    thread_handle: Option<thread::JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
    is_ready: Arc<AtomicBool>,
}

impl Node {
    pub fn new(interface: &str) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let is_ready = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();
        let is_ready_clone = is_ready.clone();
        let interface_clone = interface.to_string();

        let handle = thread::spawn(move || {
            let socket =
                socketcan::CanSocket::open(&interface_clone).expect("Failed to open CAN socket");

            // Send ready signal
            let ready_frame =
                CanFrame::new(StandardId::new(0x234).unwrap(), &[1, 2, 3, 5]).expect("");
            socket
                .write_frame(&ready_frame)
                .expect("Failed to send CAN frame");

            is_ready_clone.store(true, Ordering::Relaxed);

            // Node's main loop
            while is_running_clone.load(Ordering::Relaxed) {
                // Here, your node can handle any other messages or add some delay, for now, it just checks the running flag.
            }
        });

        Node {
            thread_handle: Some(handle),
            is_running,
            is_ready,
        }
    }

    pub fn start_and_wait_until_ready(&self) {
        while !self.is_ready.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            handle.join().expect("Failed to join CONode thread");
        }
    }
}
