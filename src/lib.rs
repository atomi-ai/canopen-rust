// TODO(zephyr): enable "#![no_std]" later, and start to build for multiple targets.
//   cargo build --target thumbv6m-none-eabi
//   cargo build --target x86_64-unknown-linux-gnu

#![cfg_attr(not(feature = "linux"), no_std)]
pub mod canopen;

mod multi_platform;
pub use multi_platform::sleep;

pub mod object_directory;
pub use object_directory::ObjectDirectory;

mod util;
