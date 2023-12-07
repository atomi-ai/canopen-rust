#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod std_items {
    extern crate alloc;
    pub use std::collections::HashMap;
    pub use std::fmt::Debug;
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
    pub use alloc::fmt::Debug;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
    pub use core::*;
    pub use hashbrown::HashMap;
    // pub fn sleep(_ms: u64) {}
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use no_std_items::*;

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let value_str = alloc::format!($($arg)*);
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            log::info!("[node] {}", value_str);
            // println!($($arg)*);
        }
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            defmt::info!("[node] {}", defmt::Debug2Format(&value_str));
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let value_str = alloc::format!($($arg)*);
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            log::debug!("[node] {}", value_str);
        }
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            defmt::debug!("[node] {}", defmt::Debug2Format(&value_str));
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        let value_str = alloc::format!($($arg)*);
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            log::warn!("[node] {}", value_str);
        }
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            defmt::warn!("[node] {}", defmt::Debug2Format(&value_str));
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let value_str = alloc::format!($($arg)*);
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            log::error!("[node] {}", value_str);
        }
        #[cfg(all(target_arch = "arm", target_os = "none"))]
        {
            defmt::error!("[node] {}", defmt::Debug2Format(&value_str));
        }
    };
}
