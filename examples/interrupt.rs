// Ref: https://github.com/laanwj/k210-sdk-stuff/blob/master/rust/interrupt/src/main.rs
#![no_std]
#![no_main]

use k210_hal::{prelude::*, pac, stdout::Stdout};
use panic_halt as _;
use riscv::register::{mie,mstatus,mhartid,/*mvendorid,marchid,mimpid,*/mcause};
use core::sync::atomic::{AtomicBool, Ordering};
// use core::ptr;

// fn peek<T>(addr: u64) -> T {
//     unsafe { ptr::read_volatile(addr as *const T) }
// }

static INTR: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Copy, Clone)]
struct IntrInfo {
    hartid: usize,
    cause: usize,
}

static mut INTR_INFO: Option<IntrInfo> = None;

#[export_name = "DefaultHandler"]
fn my_trap_handler() {
    let hartid = mhartid::read();
    let cause = mcause::read().bits();

    unsafe { INTR_INFO = Some(IntrInfo { hartid, cause }); }

    INTR.store(true, Ordering::SeqCst);
    unsafe {
        (*pac::CLINT::ptr()).msip[hartid].write(|w| w.bits(0));
    }
}

#[riscv_rt::entry]
fn main() -> ! {
    let hartid = mhartid::read();

    static mut SHARED_TX: Option<k210_hal::serial::Tx<
        k210_hal::pac::UARTHS
    >> = None;

    if hartid == 0 {
        let p = pac::Peripherals::take().unwrap();

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
    writeln!(stdout, "Hello! Some CPU information!").unwrap();
    // writeln!(stdout, "  mvendorid {:?}", mvendorid::read()).unwrap();
    // writeln!(stdout, "  marchid {:?}", marchid::read()).unwrap();
    // writeln!(stdout, "  mimpid {:?}", mimpid::read()).unwrap();
    writeln!(stdout, "This code is running on hart {}", mhartid::read()).unwrap();

    writeln!(stdout, "Enabling interrupts").unwrap();

    unsafe {
        // Enable interrupts in general
        mstatus::set_mie();
        // Set the Machine-Software bit in MIE
        mie::set_msoft();
        // Set the Machine-External bit in MIE
        mie::set_mext();
    }

    writeln!(stdout, "Generate IPI for core {} !", hartid).unwrap();
    unsafe {
        (*pac::CLINT::ptr()).msip[hartid].write(|w| w.bits(1));
    }

    writeln!(stdout, "Waiting for interrupt").unwrap();
    while !INTR.load(Ordering::SeqCst) {
    }
    INTR.store(false, Ordering::SeqCst);
    writeln!(stdout, 
        "Interrupt was triggered! Hartid: {}, cause: {}", 
        unsafe { INTR_INFO }.unwrap().hartid,
        unsafe { INTR_INFO }.unwrap().cause,
    ).unwrap();


    if hartid == 0 {
        writeln!(stdout, "Waking other harts...").unwrap();
        wake_hart(1);
    }

    loop { unsafe { riscv::asm::wfi(); } }
}

// #[riscv_rt::entry]
// fn main() -> ! {
//     let p = pac::Peripherals::take().unwrap();
//     // sysctl::pll_set_freq(sysctl::pll::PLL0, 800_000_000).unwrap();
//     // sysctl::pll_set_freq(sysctl::pll::PLL1, 300_000_000).unwrap();
//     // sysctl::pll_set_freq(sysctl::pll::PLL2, 45_158_400).unwrap();
//     let clocks = k210_hal::clock::Clocks::new();

//     let hartid = mhartid::read();
//     if hartid == 0 {
//         wake_hart(1);
//     }
//     if hartid == 1 {
//         // Add delay for hart 1
//         for _ in 0..100000 {
//             let _ = mhartid::read();
//         }
//     }

//     // usleep(200000);

//     // Configure UART
//     let serial = p.UARTHS.configure(115_200.bps(), &clocks);
//     let (mut tx, _) = serial.split();

//     let mut stdout = Stdout(&mut tx);

//     //let x: u32 = peek::<u32>(0x80000000);
//     //writeln!(stdout, "the value is {:08x}", x).unwrap();
//     writeln!(stdout, "Some CPU information !").unwrap();
//     writeln!(stdout, "  mvendorid {:?}", mvendorid::read()).unwrap();
//     writeln!(stdout, "  marchid {:?}", marchid::read()).unwrap();
//     writeln!(stdout, "  mimpid {:?}", mimpid::read()).unwrap();
//     writeln!(stdout, "This code is running on hart {}", mhartid::read()).unwrap();

//     writeln!(stdout, "Enabling interrupts").unwrap();

//     unsafe {
//         // Enable interrupts in general
//         mstatus::set_mie();
//         // Set the Machine-Software bit in MIE
//         mie::set_msoft();
//         // Set the Machine-External bit in MIE
//         mie::set_mext();
//     }

//     writeln!(stdout, "Generate IPI for core 0 !").unwrap();
//     unsafe {
//         (*pac::CLINT::ptr()).msip[0].write(|w| w.bits(1));
//     }

//     /*
//     writeln!(stdout, "Waiting for interrupt").unwrap();
//     while !INTR.load(Ordering::SeqCst) {
//     }
//     INTR.store(false, Ordering::SeqCst);
//     writeln!(stdout, "Interrupt was triggered {:?}", unsafe { INTR_INFO }).unwrap();

//     writeln!(stdout, "Generate IPI for core 1 !").unwrap();
//     unsafe {
//         (*pac::CLINT::ptr()).msip[1].write(|w| w.bits(1));
//     }
//     writeln!(stdout, "Waiting for interrupt").unwrap();
//     while !INTR.load(Ordering::SeqCst) {
//     }
//     INTR.store(false, Ordering::SeqCst);
//     writeln!(stdout, "Interrupt was triggered {:?}", unsafe { INTR_INFO }).unwrap();
//     */
    
//     writeln!(stdout, "[end]").unwrap();
//     loop {
//         unsafe { riscv::asm::wfi(); }
//     }
// }

pub fn wake_hart(hartid: usize) {
    unsafe {
        let clint = &*pac::CLINT::ptr();
        clint.msip[hartid].write(|w| w.bits(1));
    }
}

#[export_name = "_mp_hook"]
pub extern "Rust" fn user_mp_hook() -> bool {
    use riscv::register::/*{mie, */mip/*}*/;
    use riscv::asm::wfi;

    let hartid = mhartid::read();
    if hartid == 0 {
        true
    } else {
        let clint = unsafe { &*pac::CLINT::ptr() };
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
