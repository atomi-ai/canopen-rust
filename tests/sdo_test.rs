#[macro_use]
extern crate lazy_static;
mod testing;

use async_std::future::timeout;
use async_std::task;
use canopen::node;
use canopen::sdo_client::SDOClient;
use socketcan::async_io::CanSocket;
use socketcan::Frame;
use socketcan::{EmbeddedFrame, Socket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use testing::util as tu;

struct TestContext {
    _socket: CanSocket,
    _node_thread: thread::JoinHandle<()>,
}

impl TestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Setting up...");
        let content = std::fs::read_to_string(tu::EDS_PATH).expect("Failed to read EDS file");

        let s = CanSocket::open(tu::INTERFACE_NAME).unwrap();
        let read_task = s.read_frame();

        let socket =
            socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();
        let node_thread = thread::spawn(move || {
            let mut node = node::Node::new(2, &content, Box::new(socket));
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
            _socket: s,
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
fn test_sdo_request() {
    let _context = CONTEXT.lock().unwrap();

    let mut client = SDOClient::new(Box::new(
        socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket"),
    ));

    // SDO expedite upload, and wait for the response.
    let response_value = client.expedited_upload(2, 0x1017, 0);
    println!("xfguo: got response: {:?}", response_value);

    // Validate the result.
    assert!(response_value.is_some());
    assert_eq!(response_value.unwrap().data, vec![0x78, 0x56, 0x34, 0x12]);
}
