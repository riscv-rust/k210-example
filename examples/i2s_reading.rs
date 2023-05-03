#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::ToggleableOutputPin;
use k210_hal::sysctl;
use k210_hal::time::Hertz;
use k210_hal::{fpioa, gpio::Gpio, i2s, pac, prelude::*};
use pac::SYSCTL;
use panic_halt as _;

#[riscv_rt::entry]
fn main() -> ! {
    let p = unsafe { pac::Peripherals::steal() };

    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let gpio = p.GPIO.split(&mut sysctl.apb0);
    let io12 = fpioa.io12.into_function(fpioa::GPIO7);
    let io11 = fpioa.io11.into_function(fpioa::GPIO5);
    let io10 = fpioa.io10.into_function(fpioa::GPIO6);
    let mut green = Gpio::new(gpio.gpio7, io12).into_push_pull_output();
    let mut red = Gpio::new(gpio.gpio5, io11).into_push_pull_output();
    let mut blue = Gpio::new(gpio.gpio6, io10).into_push_pull_output();

    // SW SPI?
    let gpiohs27 = fpioa.io13.into_function(fpioa::GPIOHS27);
    let gpiohs28 = fpioa.io14.into_function(fpioa::GPIOHS28);

    // IO for embedded mic
    let io18 = fpioa.io18.into_function(fpioa::I2S0_SCLK);
    let io19 = fpioa.io19.into_function(fpioa::I2S0_WS);
    let io20 = fpioa.io20.into_function(fpioa::I2S0_IN_D3);

    // IO for mic array
    // let io22 = fpioa.io22.into_function(fpioa::I2S0_SCLK);
    // let io21 = fpioa.io21.into_function(fpioa::I2S0_WS);
    // let io32 = fpioa.io32.into_function(fpioa::I2S0_IN_D0);
    // let io15 = fpioa.io15.into_function(fpioa::I2S0_IN_D1);
    // let io23 = fpioa.io23.into_function(fpioa::I2S0_IN_D2);
    // let io24 = fpioa.io24.into_function(fpioa::I2S0_IN_D3);

    let i2s0 = i2s::I2s::new(p.I2S0, &mut sysctl.pll2);
    i2s0.set_rx_word_length(i2s::WordLength::Resolution16Bit);
    i2s0.set_sample_rate(Hertz(44_000));
    i2s0.configure_master(
        i2s::WordSelectCycle::SclkCycles24,
        i2s::GatingCycles::ClockCycles16,
        i2s::AlignMode::StandardMode,
    );
    /*
       from Maix import MIC_ARRAY as mic
       import lcd

       lcd.init()
       mic.init()
       # reconfigure pins after mic.init() to match your wiring

       while True:
           imga = mic.get_map()
           b = mic.get_dir(imga)
           a = mic.set_led(b,(0,0,255))
           imgb = imga.resize(240,160)
           imgc = imgb.to_rainbow(1)
           a = lcd.display(imgc)
       mic.deinit()
    */

    red.set_high().unwrap();
    green.set_high().unwrap();
    blue.set_high().unwrap();

    //i2s_

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
