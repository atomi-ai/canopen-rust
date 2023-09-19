#[cfg(feature = "linux")]
mod linux {
    use std::time::Duration;
    pub fn sleep(ms: u64) {
        std::thread::sleep(Duration::from_millis(ms));
    }
}

#[cfg(feature = "rp2040")]
mod rp2040 {
    use rp2040_hal::clocks::Clocks;
    use rp2040_hal::delay::Delay;
    use rp2040_hal::pac;
    use rp2040_hal::prelude::*; // Platform-specific access
    pub fn sleep(ms: u64) {
        let mut pac = pac::Peripherals::take().unwrap();
        let clocks = Clocks::new(pac.XOSC_CTR).init().enable_xtal();
        let mut delay = Delay::new(pac.TIMER, clocks.system_clock.freq().0);
        delay.delay_ms(ms);
    }
}

#[cfg(feature = "linux")]
pub use linux::sleep;

#[cfg(feature = "rp2040")]
pub use rp2040::sleep;
