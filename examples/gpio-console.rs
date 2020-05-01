#![no_std]
#![no_main]

use panic_halt as _;
use k210_hal::{prelude::*, fpioa, pac, gpio::Gpio, gpiohs::Gpiohs, stdout::Stdout};

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

    // let boot_button = Gpio::new(
    //     gpio.gpio2, 
    //     fpioa.io16.into_function(fpioa::GPIO2)
    // ).into_pull_up_input();
    let io14 = fpioa.io14.into_function(fpioa::GPIO6);
    let mut blue = Gpio::new(gpio.gpio6, io14).into_push_pull_output();

    // Configure UART
    let serial = p.UARTHS.configure(
        115_200.bps(), 
        &clocks
    );
    let (mut tx, _rx) = serial.split();

    let mut stdout = Stdout(&mut tx);

    writeln!(stdout, "Hello, Rust!").ok();

    unsafe { &*pac::GPIO::ptr() }.source.write(|w| unsafe { w.bits(0xaa) });
    loop {
        for i in 8..16 {
            let io = unsafe { &*pac::FPIOA::ptr() }.io[i].read();
            writeln!(stdout, 
                "[{}] CH {}, DS {:02X}, OE {}, OEI {}, DO {}, DOI {}, PU {}, PD {}, SL {}, IE {}, IEI {}, DII {}, ST {}, PA {}",
                i, io.ch_sel().bits(), io.ds().bits(), io.oe_en().bit(), io.oe_inv().bit(),
                io.do_sel().bit(), io.do_inv().bit(), io.pu().bit(), io.pd().bit(), 
                io.sl().bit(), io.ie_en().bit(), io.ie_inv().bit(), io.di_inv().bit(), io.st().bit(), io.pad_di().bit()
            ).ok();
        }
        let data_output = unsafe { &*pac::GPIO::ptr() }.data_output.read().bits();
        let direction = unsafe { &*pac::GPIO::ptr() }.direction.read().bits();
        let source = unsafe { &*pac::GPIO::ptr() }.source.read().bits();
        let data_input = unsafe { &*pac::GPIO::ptr() }.data_input.read().bits();
        let sync_level = unsafe { &*pac::GPIO::ptr() }.sync_level.read().bits();
        let id_code = unsafe { &*pac::GPIO::ptr() }.id_code.read().bits();
        writeln!(stdout, 
            "O {:08b}, D {:08b}, S {:08b}, I {:08b}, SY {:08b}, ID 0x{:08X}",
            data_output, direction, source, data_input, sync_level, id_code
        ).ok();
    }
}
