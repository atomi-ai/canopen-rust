#[macro_use]
extern crate lazy_static;
mod testing;

use async_std::future::timeout;
use async_std::task;
use canopen_rust::canopen;
use socketcan::async_io::CanSocket;
use socketcan::EmbeddedFrame;
use socketcan::Frame;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use testing::sdo_client::SDOClient;
use testing::util as tu;

struct TestContext {
    socket: CanSocket,
    node_thread: thread::JoinHandle<()>,
}

impl TestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Setting up...");
        let content = std::fs::read_to_string(tu::EDS_PATH).expect("Failed to read EDS file");

        let s = CanSocket::open(tu::INTERFACE_NAME).unwrap();
        let read_task = s.read_frame();

        let node = canopen::Node::new(tu::INTERFACE_NAME, 2, &content);
        let shared_node = Arc::new(node);
        let clone_node = Arc::clone(&shared_node);
        let node_thread = thread::spawn(move || {
            clone_node.run();
        });
        shared_node.wait_until_ready();

        let msg = timeout(Duration::from_secs(3), read_task).await??;

        if msg.raw_id() != 0x234 || msg.data() != &[0x01, 0x02, 0x03, 0x05] {
            panic!(
                "Received unexpected CanFrame: {}",
                tu::frame_to_string(&msg)
            );
        }

        Ok(TestContext {
            socket: s,
            node_thread,
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
    // 获取已经初始化的TestContext
    let context = CONTEXT.lock().unwrap();

    // 创建SDO客户端
    let mut client = SDOClient::new(tu::INTERFACE_NAME, 2); // 2作为node_id

    // 使用SDO客户端发送expedited upload请求
    let response_value = client.expedited_upload(2, 0x1017, 0);
    println!("xfguo: got response: {:?}", response_value);

    // 验证我们收到了预期的值
    assert!(response_value.is_some());
    assert_eq!(response_value.unwrap().data, vec![0x78, 0x56, 0x34, 0x12]);
}
