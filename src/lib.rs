// To build for different platform:
//   cargo build --target thumbv6m-none-eabi
//   cargo build --target x86_64-unknown-linux-gnu

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]

extern crate alloc;

pub mod data_type;
pub mod error;
pub mod node;
pub mod object_directory;
pub mod util;
pub mod value;
pub mod pdo;

mod cmd_header;
mod prelude;
mod sdo_server;
mod emergency;
