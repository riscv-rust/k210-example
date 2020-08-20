#![no_std]
#![no_main]

use panic_halt as _;
use k210_hal::{prelude::*, fpioa, pac, gpio::Gpio};

#[riscv_rt::entry]
fn main() -> ! {
    // todo
    let p = pac::Peripherals::take().unwrap();
    
    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let gpio = p.GPIO.split(&mut sysctl.apb0);
    let io14 = fpioa.io14.into_function(fpioa::GPIO6);
    let mut blue = Gpio::new(gpio.gpio6, io14).into_push_pull_output();

    blue.try_set_low().ok();

    let mut last_update = riscv::register::mcycle::read();
    loop {
        let cur = riscv::register::mcycle::read();
        if cur - last_update >= 100_000_000 {
            last_update = cur;

            blue.try_toggle().ok();
        }
    }
}
