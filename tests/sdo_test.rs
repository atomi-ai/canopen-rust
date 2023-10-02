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
fn test_write_and_read() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // read / write / read for 1017h:00h
    send(&s, &genf(0x602, &[0x40, 0x17, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x4B, 0x17, 0x10, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0x2B, 0x17, 0x10, 0, 0x12, 0x34, 0, 0]));
    exp(&s, &genf(0x582, &[0x60, 0x17, 0x10, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0x40, 0x17, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x4B, 0x17, 0x10, 0, 0x12, 0x34, 0, 0]));
}

#[test]
fn test_error_write() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Write object 1000h:00h (ro) => ERR 06010002h
    send(&s, &genf(0x602, &[0x23, 0, 0x10, 0, 0x91, 0x01, 0x0F, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0, 0x02, 0, 0x01, 0x06]));
}

#[test]
fn test_error_read() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    // Read object 1000h:01h => ERR 06090011h
    send(&s, &genf(0x602, &[0x40, 0, 0x10, 0x1, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0x1, 0x11, 0, 0x09, 0x06]));
    // Read object 1004h:00h => ERR 06020000h
    send(&s, &genf(0x602, &[0x40, 0x04, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x04, 0x10, 0, 0, 0, 0x02, 0x06]));
    // Read object 1000h:00h => ERR 05040001h
    send(&s, &genf(0x602, &[0xE0, 0, 0x10, 0x0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0x0, 0x01, 0, 0x04, 0x05]));
}

#[test]
// Expedite upload
fn test_read_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1000h:00h => 0xF0191
    send(&s, &genf(0x602, &[0x40, 0, 0x10, 0x0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0, 0x10, 0x0, 0x91, 0x01, 0x0F, 0]));
}

#[test]
// SDO 08 & 09
fn test_error_mismatch_length() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1003h:00h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x03, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x4F, 0x03, 0x10, 0, 0, 0, 0, 0]));

    // Write object 1003h:00h with 0x00000000 (u32) => ERR 06070012
    send(&s, &genf(0x602, &[0x23, 0x03, 0x10, 0x0, 0x12, 0x34, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x03, 0x10, 0, 0x12, 0, 0x07, 0x6]));

    // Read object 1005h:00h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x05, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0x05, 0x10, 0, 0x80, 0, 0, 0]));

    // Write object 1005h:00h with 0x00000000 (u32) => ERR 06070012
    send(&s, &genf(0x602, &[0x2F, 0x05, 0x10, 0, 0x12, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x05, 0x10, 0, 0x13, 0, 0x07, 0x6]));
}

#[test]
// SDO 12
fn test_with_node_id_in_expr() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1200h:01h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x00, 0x12, 0x1, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0x00, 0x12, 0x1, 0x02, 0x06, 0, 0]));

    // Read object 1200h:02h => 0x0 (u8)
    send(&s, &genf(0x602, &[0x40, 0x00, 0x12, 0x2, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x43, 0x00, 0x12, 0x2, 0x82, 0x05, 0, 0]));
}

#[test]
// SDO 16
fn test_segment_upload() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1008h:00h => 0x10 (u8)
    send(&s, &genf(0x602, &[0x40, 0x08, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x41, 0x08, 0x10, 0x0, 0x10, 0, 0, 0]));

    let t = [0x0, 0x43, 0x41, 0x4E, 0x6F, 0x70, 0x65, 0x6E];
    send(&s, &genf(0x602, &[0x60, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &t));

    let t = [0x10, 0x44, 0x65, 0x6D, 0x6F, 0x50, 0x49, 0x43];
    send(&s, &genf(0x602, &[0x70, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &t));

    send(&s, &genf(0x602, &[0x60, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x0B, 0x33, 0x32, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 17
fn test_segment_upload_with_toggle_bit_error() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    // Read object 1008h:00h => 0x10 (u8)
    send(&s, &genf(0x602, &[0x40, 0x08, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x41, 0x08, 0x10, 0x0, 0x10, 0, 0, 0]));

    send(&s, &genf(0x602, &[0x70, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x08, 0x10, 0, 0, 0, 0x03, 0x05]));
}

#[test]
// SDO 15
fn test_segment_download_basic() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    // Write object 1017h:00h with 0x0002 (u16)
    send(&s, &genf(0x602, &[0x21, 0x17, 0x10, 0x0, 0x02, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x60, 0x17, 0x10, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0x0B, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x20, 0, 0, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 19, read
fn test_block_upload_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xA0, 0x00, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x00, 0x10, 0x00, 0x04, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x81, 0x91, 0x01, 0x0F, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x01, 0x14, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xCD, 0, 0, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 21, read
fn test_block_upload_string_without_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xA0, 0x08, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x08, 0x10, 0x00, 0x10, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    let t = [0x01, 0x43, 0x41, 0x4E, 0x6F, 0x70, 0x65, 0x6E];
    exp(&s, &genf(0x582, &t));
    let t = [0x02, 0x44, 0x65, 0x6D, 0x6F, 0x50, 0x49, 0x43];
    exp(&s, &genf(0x582, &t));
    exp(&s, &genf(0x582, &[0x83, 0x33, 0x32, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x03, 0x14, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xD5, 0, 0, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 23, read
fn test_block_upload_string_with_crc() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xA4, 0x08, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x08, 0x10, 0x00, 0x10, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    let t = [0x01, 0x43, 0x41, 0x4E, 0x6F, 0x70, 0x65, 0x6E];
    exp(&s, &genf(0x582, &t));
    let t = [0x02, 0x44, 0x65, 0x6D, 0x6F, 0x50, 0x49, 0x43];
    exp(&s, &genf(0x582, &t));
    exp(&s, &genf(0x582, &[0x83, 0x33, 0x32, 0, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x03, 0x14, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xD5, 0xF3, 0x43, 0, 0, 0, 0, 0]));
    send(&s, &genf(0x602, &[0xA1, 0, 0, 0, 0, 0, 0, 0]));
}

#[test]
// SDO 26, read
fn test_block_upload_with_wrong_blocksize() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xA0, 0x00, 0x10, 0x00, 0x80, 0, 0, 0]));
    let t = [0x80, 0x00, 0x10, 0x00, 0x02, 0x00, 0x04, 0x05];
    exp(&s, &genf(0x582, &t));
}

#[test]
// SDO 27, read
fn test_block_upload_with_wrong_ack_seqno() {
    let _context = CONTEXT.lock().unwrap();
    let s = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    send(&s, &genf(0x602, &[0xA0, 0x00, 0x10, 0x00, 0x14, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0xC6, 0x00, 0x10, 0x00, 0x04, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA3, 0, 0, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x81, 0x91, 0x01, 0x0F, 0, 0, 0, 0]));

    send(&s, &genf(0x602, &[0xA2, 0x80, 0x14, 0, 0, 0, 0, 0]));
    let t = [0x80, 0x00, 0x10, 0x00, 0x01, 0x00, 0x04, 0x05];
    exp(&s, &genf(0x582, &t));
}
