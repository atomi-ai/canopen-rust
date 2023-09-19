mod testing_util;
use testing_util as tu;

use async_std::task;
use std::sync::Arc;

#[macro_use]
extern crate lazy_static;

use async_std::future::timeout;
use canopen_rust::canopen;
use socketcan::async_io::CanSocket;
use socketcan::EmbeddedFrame;
use socketcan::Frame;
use std::sync::Mutex;
use std::time::Duration;

const INTERFACE_NAME: &str = "vcan0";

struct TestContext {
    socket: CanSocket,
    node: canopen::Node,
}

impl TestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Setting up...");

        let s = CanSocket::open(INTERFACE_NAME).unwrap();
        let read_task = s.read_frame();
        let n = canopen::Node::new(INTERFACE_NAME);
        n.start_and_wait_until_ready();

        let msg = timeout(Duration::from_secs(3), read_task).await??;

        if msg.raw_id() != 0x234 || msg.data() != &[0x01, 0x02, 0x03, 0x05] {
            panic!(
                "Received unexpected CanFrame: {}",
                tu::frame_to_string(&msg)
            );
        }

        Ok(TestContext { socket: s, node: n })
    }
}

lazy_static! {
    static ref CONTEXT: Arc<Mutex<TestContext>> = {
        let ctx = task::block_on(TestContext::new()).unwrap();
        Arc::new(Mutex::new(ctx))
    };
}

#[test]
fn test_case_1() {
    let context = task::block_on(TestContext::new()).unwrap();
}

#[test]
fn test_case_2() {
    let context = task::block_on(TestContext::new()).unwrap();
}
