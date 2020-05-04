#![no_std]
#![no_main]

use panic_halt as _;
use k210_hal::{prelude::*, fpioa, pac, gpio::Gpio, gpiohs, stdout::Stdout};

#[riscv_rt::entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let gpio = p.GPIO.split(&mut sysctl.apb0);
    let gpiohs = p.GPIOHS.split();

    // Configure clocks (TODO)
    let clocks = k210_hal::clock::Clocks::new();

    // prepare pins
    let _uarths_tx = fpioa.io5.into_function(fpioa::UARTHS_TX);
    fpioa.io16.into_function(fpioa::GPIOHS0);
    let boot_button = gpiohs.gpiohs0.into_pull_up_input();
    // let boot_button = Gpio::new(
    //     gpio.gpio0, 
    //     fpioa.io16.into_function(fpioa::GPIO0)
    // ).into_pull_up_input();

    // Configure UART
    let serial = p.UARTHS.configure(
        115_200.bps(), 
        &clocks
    );
    let (mut tx, _) = serial.split();

    // todo: new stdout design (simple Write impl?)
    let mut stdout = Stdout(&mut tx);

    writeln!(stdout, "Hello, Rust!").ok();

    loop {
        let input_state = boot_button.is_high().unwrap();
        let dir = unsafe { &*pac::GPIO::ptr() }.direction.read().bits();
        writeln!(stdout, "Io16 input: {}; direction value: 0x{:08X}", input_state, dir);
    }
}
