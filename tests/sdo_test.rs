#[macro_use]
extern crate lazy_static;
mod testing;
use async_std::future::timeout;
use async_std::task;
use canopen::node;
use embedded_can::StandardId;
use nix::poll::{poll, PollFd, PollFlags};
use socketcan::CanFrame;
use socketcan::CanSocket;
use socketcan::Frame;
use socketcan::{EmbeddedFrame, Socket};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
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

fn gen_frame(node_id: u16, cmd: u8, index: u16, sub_index: u8, data: Vec<u8>) -> CanFrame {
    let mut packet = Vec::new();
    packet.push(cmd);
    packet.push((index & 0xFF) as u8);
    packet.push((index >> 8) as u8);
    packet.push(sub_index);
    packet.extend_from_slice(data.as_slice());
    while packet.len() < 8 {
        packet.push(0);
    }

    let t = CanFrame::new(StandardId::new(node_id).unwrap(), packet.as_slice());
    t.unwrap()
}

fn read_frame_with_timeout(
    socket: &socketcan::CanSocket,
    timeout: std::time::Duration,
) -> Result<CanFrame, &'static str> {
    let mut fds = [PollFd::new(socket.as_raw_fd(), PollFlags::POLLIN)];

    match poll(&mut fds, timeout.as_millis() as i32) {
        Ok(n) => {
            if n == 0 {
                // 超时
                return Err("Timeout");
            }
            match socket.read_frame() {
                Ok(frame) => Ok(frame),
                Err(_) => Err("Error reading frame"),
            }
        }
        Err(_) => Err("Poll error"),
    }
}

fn send_and_expect(socket: &CanSocket, req: &socketcan::CanFrame, exp_resp: &socketcan::CanFrame) {
    // 发送请求帧
    socket
        .write_frame(req)
        .expect("Failed to send request frame");

    // 设置等待响应的超时
    let timeout = Duration::from_millis(100);
    let start_time = Instant::now();

    loop {
        if let Ok(response_frame) = read_frame_with_timeout(socket, timeout) {
            println!("response_frame: {:?}", response_frame);
            if response_frame.id() == exp_resp.id() && response_frame.data() == exp_resp.data() {
                return;
            }
        }
        println!("here");
        if start_time.elapsed() >= Duration::from_secs(1) {
            break;
        }
    }
    assert!(false, "Timeout in getting response of: {:?}", exp_resp);
}
//
// fn test_exp() {
//     let _context = CONTEXT.lock().unwrap();
//     let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
// }

#[test]
fn test_write_and_read() {
    let _context = CONTEXT.lock().unwrap();
    let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // read / write / read for 1017h:00h
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1017, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x4B, 0x1017, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
    );
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x2B, 0x1017, 0x0, vec![0x12, 0x34, 0x0, 0x0]),
        &gen_frame(0x582, 0x60, 0x1017, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
    );
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1017, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x4B, 0x1017, 0x0, vec![0x12, 0x34, 0x0, 0x0]),
    );
}

#[test]
fn test_error_write() {
    let _context = CONTEXT.lock().unwrap();
    let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Write object 1000h:00h (ro) => ERR 06010002h
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x23, 0x1000, 0x0, vec![0x91, 0x01, 0x0F, 0x0]),
        &gen_frame(0x582, 0x80, 0x1000, 0x0, vec![0x02, 0x00, 0x01, 0x06]),
    );
}

#[test]
fn test_error_read() {
    let _context = CONTEXT.lock().unwrap();
    let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");
    // Read object 1000h:01h => ERR 06090011h
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1000, 0x1, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x80, 0x1000, 0x1, vec![0x11, 0x00, 0x09, 0x06]),
    );
    // Read object 1004h:00h => ERR 06020000h
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1004, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x80, 0x1004, 0x0, vec![0x00, 0x00, 0x02, 0x06]),
    );
    // Read object 1000h:00h => ERR 05040001h
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0xE0, 0x1000, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x80, 0x1000, 0x0, vec![0x01, 0x00, 0x04, 0x05]),
    );
}

#[test]
// Expedite upload
fn test_sdo_read_basic() {
    let _context = CONTEXT.lock().unwrap();
    let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1000h:00h => 0xF0191
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1000, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x43, 0x1000, 0x0, vec![0x91, 0x01, 0x0F, 0x00]),
    );
}

#[test]
// SDO 08 & 09
fn test_sdo_error_mismatch_length() {
    let _context = CONTEXT.lock().unwrap();
    let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1003h:00h => 0x0 (u8)
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1003, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x4F, 0x1003, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
    );

    // Write object 1003h:00h with 0x00000000 (u32) => ERR 06070012
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x23, 0x1003, 0x0, vec![0x12, 0x34, 0x0, 0x0]),
        &gen_frame(0x582, 0x80, 0x1003, 0x0, vec![0x12, 0x00, 0x07, 0x06]),
    );

    // Read object 1003h:00h => 0x0 (u8)
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1005, 0x0, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x43, 0x1005, 0x0, vec![0x80, 0x0, 0x0, 0x0]),
    );

    // Write object 1003h:00h with 0x00000000 (u32) => ERR 06070012
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x2F, 0x1005, 0x0, vec![0x12]),
        &gen_frame(0x582, 0x80, 0x1005, 0x0, vec![0x13, 0x00, 0x07, 0x06]),
    );
}

#[test]
// SDO 12
fn test_sdo_with_node_id_in_expr() {
    let _context = CONTEXT.lock().unwrap();
    let socket = socketcan::CanSocket::open(tu::INTERFACE_NAME).expect("Failed to open CAN socket");

    // Read object 1200h:01h => 0x0 (u8)
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1200, 0x1, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x43, 0x1200, 0x1, vec![0x02, 0x06, 0x0, 0x0]),
    );
    // Read object 1200h:02h => 0x0 (u8)
    send_and_expect(
        &socket,
        &gen_frame(0x602, 0x40, 0x1200, 0x2, vec![0x0, 0x0, 0x0, 0x0]),
        &gen_frame(0x582, 0x43, 0x1200, 0x2, vec![0x82, 0x05, 0x0, 0x0]),
    );
}
