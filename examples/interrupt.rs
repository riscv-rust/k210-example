#![no_std]
#![no_main]
//todo: finish this example

use k210_hal::{prelude::*, fpioa, pac, clock::Clocks};
use k210_hal::stdout::Stdout;
use core::mem::MaybeUninit;
use panic_halt as _;

#[export_name = "DefaultHandler"]
fn custom_interrupt_handler() {
    let stdout = unsafe { &mut *STDOUT.as_mut_ptr() };
    writeln!(stdout, "Interrupt!").ok();
}

static mut STDOUT: MaybeUninit<Stdout<
    k210_hal::serial::Tx<pac::UARTHS>>
> = core::mem::MaybeUninit::uninit();

#[riscv_rt::entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);

    let clocks = Clocks::new();

    let _uarths_tx = fpioa.io5.into_function(fpioa::UARTHS_TX);
    let serial = p.UARTHS.configure(
        115_200.bps(), 
        &clocks
    );
    let (mut tx, _) = serial.split();

    let mut stdout = Stdout(&mut tx);
    unsafe { STDOUT = MaybeUninit::new(stdout) };
    
    loop { unsafe { riscv::asm::wfi(); } }
}
