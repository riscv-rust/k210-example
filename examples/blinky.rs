#![no_std]
#![no_main]

use panic_halt as _;
use k210_hal::{prelude::*, fpioa, pac, gpio::Gpio};

#[riscv_rt::entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    
    let fpioa = p.FPIOA.split();
    let io14 = fpioa.io14.into_function(fpioa::Gpio6);

    // Configure clocks (TODO)
    let clocks = k210_hal::clock::Clocks::new();

    let gpio = p.GPIO.split();
    let mut gpio6 = Gpio::new(gpio.gpio6, io14).into_push_pull_output();

    gpio6.set_low().ok();

    loop {}
}
