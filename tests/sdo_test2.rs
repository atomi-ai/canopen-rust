/*
The file document was separated from sdo_test.rs.

Since the block size on the server side can be modified by incoming
requests, I've moved all test cases that require the default block
size to this file.
 */
#[macro_use]
extern crate lazy_static;
mod testing;

use crate::testing::util::{exp, send};
use async_std::future::timeout;
use async_std::task;
use canopen::node;
use canopen::util::genf;
use socketcan::Frame;
use socketcan::{EmbeddedFrame, Socket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use testing::util as tu;

struct TestContext {
    _node_thread: thread::JoinHandle<()>,
}

impl TestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Wait for the server up...");
        let content = std::fs::read_to_string(tu::DEMO_EDS_PATH).expect("Failed to read EDS file");
        let s = socketcan::async_io::CanSocket::open(tu::INTERFACE_NAME).unwrap();
        let read_task = s.read_frame();

        println!("Start the testing server thread");
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();
        let node_thread = thread::spawn(move || {
            let mut node = node::Node::new(
                2,
                &content,
                Box::new(
                    socketcan::CanSocket::open(tu::INTERFACE_NAME)
                        .expect("Failed to open CAN socket"),
                ),
            );
            node.init();
            is_running_clone.store(true, Ordering::Relaxed);
            node.run();
        });
        while !is_running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }

        let msg = timeout(Duration::from_secs(3), read_task).await??;
        if msg.raw_id() != 0x234 || msg.data() != &[0x01, 0x02, 0x03, 0x05] {
            panic!(
                "Received unexpected CanFrame: {}",
                tu::frame_to_string(&msg)
            );
        }

        Ok(TestContext {
            _node_thread: node_thread,
        })
    }
}

lazy_static! {
    static ref CONTEXT: Arc<Mutex<TestContext>> = {
        let ctx = task::block_on(TestContext::new()).unwrap();
        Arc::new(Mutex::new(ctx))
    };
}

#[test]
// SDO 21, write
fn test_block_download_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xC2, 0x17, 0x10, 0x00, 0x02, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xA4, 0x17, 0x10, 0x00, 0x7F, 0, 0, 0]));

    send(&s, &genf(0x602, &[0x81, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xA2, 0x01, 0x7F, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xD5, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 25, write
// Where is CRC?
fn test_block_download_with_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xC6, 0x17, 0x10, 0x00, 0x02, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xA4, 0x17, 0x10, 0x00, 0x7F, 0, 0, 0]));

    send(&s, &genf(0x602, &[0x81, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xA2, 0x01, 0x7F, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xD5, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}
