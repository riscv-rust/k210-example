#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::ToggleableOutputPin;
use k210_hal::{fpioa, gpio::Gpio, pac, prelude::*};
use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    let p = unsafe { pac::Peripherals::steal() };

    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let gpio = p.GPIO.split(&mut sysctl.apb0);
    let io14 = fpioa.io14.into_function(fpioa::GPIO6);
    let mut blue = Gpio::new(gpio.gpio6, io14).into_push_pull_output();

    blue.set_low().unwrap();

    let mut last_update = riscv::register::mcycle::read();
    loop {
        let cur = riscv::register::mcycle::read();
        if cur - last_update >= 100_000_000 {
            last_update = cur;

            blue.toggle().unwrap();
        }
    }
}
