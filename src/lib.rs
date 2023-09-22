// TODO(zephyr): enable "#![no_std]" later, and start to build for multiple targets.
//   cargo build --target thumbv6m-none-eabi
//   cargo build --target x86_64-unknown-linux-gnu

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]

pub mod node;
pub mod object_directory;
pub mod util;

mod prelude;
