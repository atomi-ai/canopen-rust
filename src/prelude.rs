#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod std_items {
    pub use std::boxed::Box;
    pub use std::collections::HashMap;
    pub use std::fmt::Debug;
    pub use std::fmt::Error;
    pub use std::*;
    //
    // pub fn sleep(ms: u64) {
    //     use std::time::Duration;
    //     std::thread::sleep(Duration::from_millis(ms));
    // }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub use std_items::*;

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod no_std_items {
    extern crate alloc;
    pub use alloc::boxed::Box;
    pub use alloc::fmt::Debug;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
    pub use core::fmt::Error;
    pub use core::*;
    pub use hashbrown::HashMap;

    // pub fn sleep(_ms: u64) {}
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use no_std_items::*;

#[macro_export]
macro_rules! xprintln {
    ($($arg:tt)*) => {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            println!($($arg)*);
        }
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            // TODO(zephyr): Add logging solution for RP2040.
        }
    };
}
