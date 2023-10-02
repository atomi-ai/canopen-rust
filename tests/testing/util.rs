use embedded_can::Frame;
use nix::poll::{poll, PollFd, PollFlags};
use socketcan::{CanFrame, CanSocket, Socket};
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

pub const INTERFACE_NAME: &str = "vcan0";
pub const EDS_PATH: &str = "tests/fixtures/sample.eds";
pub const DEMO_EDS_PATH: &str = "tests/fixtures/demoDevice.eds";

pub fn frame_to_string<F: socketcan::Frame>(frame: &F) -> String {
    let id = frame.raw_id();
    let data_string = frame
        .data()
        .iter()
        .fold(String::from(""), |a, b| format!("{} {:02x}", a, b));

    format!("{:X}  [{}] {}", id, frame.dlc(), data_string)
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

pub fn send(socket: &CanSocket, req: &socketcan::CanFrame) {
    socket
        .write_frame(req)
        .expect("Failed to send request frame");
}

pub fn exp(socket: &CanSocket, exp_resp: &socketcan::CanFrame) {
    // 设置等待响应的超时
    let timeout = Duration::from_millis(100);
    let start_time = Instant::now();

    loop {
        if let Ok(response_frame) = read_frame_with_timeout(socket, timeout) {
            if response_frame.id() == exp_resp.id() && response_frame.data() == exp_resp.data() {
                return;
            }
        }
        if start_time.elapsed() >= Duration::from_secs(1) {
            break;
        }
    }
    assert!(false, "Timeout in getting response of: {:?}", exp_resp);
}
