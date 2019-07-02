#![no_std]
#![no_main]

extern crate panic_halt;

use riscv::register::mhartid;
use riscv_rt::entry;
use k210_hal::prelude::*;
use k210_hal::pac::{Peripherals, UARTHS, CLINT};
use k210_hal::stdout::Stdout;
use k210_hal::serial::Tx;

#[export_name = "_mp_hook"]
pub extern "Rust" fn user_mp_hook() -> bool {
    use riscv::register::{mie, mip};
    use riscv::asm::wfi;

    let hartid = mhartid::read();
    if hartid == 0 {
        true
    } else {
        let clint = unsafe { &*CLINT::ptr() };
        let msip = &clint.msip[hartid];

        unsafe {
            // Clear IPI
            msip.write(|w| w.bits(0));

            // Start listening for software interrupts
            mie::set_msoft();

            loop {
                wfi();
                if mip::read().msoft() {
                    break;
                }
            }

            // Stop listening for software interrupts
            mie::clear_msoft();

            // Clear IPI
            msip.write(|w| w.bits(0));
        }
        false
    }
}

pub fn wake_hart(hartid: usize) {
    unsafe {
        let clint = &*CLINT::ptr();
        clint.msip[hartid].write(|w| w.bits(1));
    }
}


#[entry]
fn main() -> ! {
    let hartid = mhartid::read();

    static mut SHARED_TX: Option<Tx<UARTHS>> = None;

    if hartid == 0 {
        let p = Peripherals::take().unwrap();

        //configure_fpioa(p.FPIOA);

        // Configure clocks (TODO)
        let clocks = k210_hal::clock::Clocks::new();

        // Configure UART
        let serial = p.UARTHS.configure(115_200.bps(), &clocks);
        let (tx, _) = serial.split();

        unsafe {
            SHARED_TX.replace(tx);
        }
    }

    // Super-unsafe UART sharing!
    let tx = unsafe {
        SHARED_TX.as_mut().unwrap()
    };
    let mut stdout = Stdout(tx);

    if hartid == 1 {
        // Add delay for hart 1
        for _ in 0..100000 {
            let _ = mhartid::read();
        }
    }

    writeln!(stdout, "Hello, Rust from hart {}", hartid).unwrap();
    if hartid == 0 {
        writeln!(stdout, "Waking other harts...").unwrap();
        wake_hart(1);
    }

    loop { }
}
