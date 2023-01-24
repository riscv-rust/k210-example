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
    let io12 = fpioa.io12.into_function(fpioa::GPIO7);
    let io13 = fpioa.io13.into_function(fpioa::GPIO5);
    let io14 = fpioa.io14.into_function(fpioa::GPIO6);
    let mut green = Gpio::new(gpio.gpio7, io12).into_push_pull_output();
    let mut red = Gpio::new(gpio.gpio5, io13).into_push_pull_output();
    let mut blue = Gpio::new(gpio.gpio6, io14).into_push_pull_output();

    red.set_high().unwrap();
    green.set_high().unwrap();
    blue.set_high().unwrap();

    let mut last_update = riscv::register::mcycle::read();
    let mut i = 0;
    loop {
        let cur = riscv::register::mcycle::read();
        if cur - last_update >= 100_000_000 {
            last_update = cur;

            red.toggle().unwrap();
            if i % 2 == 0 {
              green.toggle().unwrap();
            }
            if i % 3 == 0 {
              blue.toggle().unwrap();
            }

            i += 1;
        }
    }
}
