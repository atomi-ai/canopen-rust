mod testing;
use testing::util as tu;

use async_std::future::timeout;
use canopen_rust::canopen;
use socketcan::async_io::CanSocket;
use socketcan::{CanFrame, EmbeddedFrame, Socket, StandardId};
use std::thread;
use std::time::Duration;
use tokio;

#[test]
fn test_nodes_communication_basic() {
    let node_interface = "vcan0"; // 你的虚拟 CAN 接口

    // 节点1用于发送的socket
    let socket_node1 =
        socketcan::CanSocket::open("vcan0").expect("Failed to open CAN socket for node1");

    // 节点2用于接收的socket
    let socket_node2 =
        socketcan::CanSocket::open(node_interface).expect("Failed to open CAN socket for node2");

    // 节点1发送消息
    let message_from_node1 = CanFrame::new(StandardId::new(0x123).unwrap(), &[1, 2, 3, 4])
        .expect("Failed to create CAN frame");
    socket_node1
        .write_frame(&message_from_node1)
        .expect("Failed to send CAN frame from node1");

    // 节点2接收消息
    let received_by_node2 = socket_node2
        .read_frame()
        .expect("Failed to read CAN frame by node2");
    assert_eq!(received_by_node2.data(), &[1, 2, 3, 4]);
}

#[tokio::test]
async fn test_start_a_conode() {
    let client_socket = CanSocket::open(tu::INTERFACE_NAME).unwrap();
    let read_task = client_socket.read_frame();

    thread::spawn(move || {
        let node = canopen::Node::new(tu::INTERFACE_NAME, 2, tu::EDS_PATH);
        node.run();
    });

    // Wait for the expected msg
    let timeout_duration = Duration::from_secs(5);
    let msg = timeout(timeout_duration, read_task).await.unwrap().unwrap();

    println!("Got msg {}", tu::frame_to_string(&msg));
}
