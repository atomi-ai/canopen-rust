use canopen::util::genf;
use embedded_can::Frame;
use nix::poll::{poll, PollFd, PollFlags};
use socketcan::{CanFrame, CanSocket, Socket};
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

pub const INTERFACE_NAME: &str = "can0";
pub const DEMO_EDS_PATH: &str = "tests/fixtures/demoDevice.eds";

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

fn main() {
    let s = socketcan::CanSocket::open(INTERFACE_NAME).expect("Failed to open CAN socket");
    // send(&s, &genf(0x602, &[0x40, 0, 0x10, 0x0, 0, 0, 0, 0]));
    // exp(&s, &genf(0x582, &[0x43, 0, 0x10, 0x0, 0x91, 0x01, 0x0F, 0]));

    send(&s, &genf(0x602, &[0x40, 0, 0x10, 0x1, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0x1, 0x11, 0, 0x09, 0x06]));
    // Read object 1004h:00h => ERR 06020000h
    send(&s, &genf(0x602, &[0x40, 0x04, 0x10, 0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0x04, 0x10, 0, 0, 0, 0x02, 0x06]));
    // Read object 1000h:00h => ERR 05040001h
    send(&s, &genf(0x602, &[0xE0, 0, 0x10, 0x0, 0, 0, 0, 0]));
    exp(&s, &genf(0x582, &[0x80, 0, 0x10, 0x0, 0x01, 0, 0x04, 0x05]));
}
