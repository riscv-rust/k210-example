#![no_std]
#![no_main]

use k210_hal::{prelude::*, pac, clint::msip, fpioa, stdout::Stdout, gpiohs::Gpiohs};
use panic_halt as _;
use riscv::register::{mie,mstatus,mhartid,mcause};
use core::sync::atomic::{AtomicBool, Ordering};

static INTR: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Copy, Clone)]
struct IntrInfo {
    hart_id: usize,
    cause: usize,
}

static mut INTR_INFO: Option<IntrInfo> = None;

#[export_name = "MachineExternal"]
fn my_trap_handler() {
    let stdout = unsafe { &mut *SHARED_STDOUT.as_mut_ptr() };

    let hart_id = mhartid::read();
    writeln!(stdout, "Interrupt 1!").unwrap();
    // let ie_flag = mie::read().bits();
    let irq_number = unsafe {
        (*pac::PLIC::ptr()).targets[hart_id].claim.read().bits()
    };
    writeln!(stdout, "Interrupt 2 {}!", irq_number).unwrap();
    let int_threshold = unsafe { 
        (*pac::PLIC::ptr()).targets[hart_id].threshold.read().bits()
    };
    writeln!(stdout, "Interrupt 3 {}!", int_threshold).unwrap();
    unsafe { 
        // not hart_id! it's irq_number!
        let bits = (*pac::PLIC::ptr()).priority[irq_number as usize].read().bits();
        (*pac::PLIC::ptr()).targets[hart_id].threshold.write(|w| 
            w.bits(bits));
        mie::clear_msoft();
        mie::clear_mtimer();
        // mstatus::set_mie();
        writeln!(stdout, "Interrupt 4 {}!", bits).unwrap();
    }

    unsafe { 
        &(*pac::GPIOHS::ptr()).rise_ie.write(|w| w.pin0().clear_bit());
        &(*pac::GPIOHS::ptr()).rise_ip.write(|w| w.pin0().set_bit());
        &(*pac::GPIOHS::ptr()).rise_ie.write(|w| w.pin0().set_bit());
    
        &(*pac::GPIOHS::ptr()).fall_ie.write(|w| w.pin0().clear_bit());
        &(*pac::GPIOHS::ptr()).fall_ip.write(|w| w.pin0().set_bit());
        &(*pac::GPIOHS::ptr()).fall_ie.write(|w| w.pin0().set_bit());
    }

    // actual handle process
    let cause = mcause::read().bits();

    writeln!(stdout, "Interrupt!!! {} {:016X}", hart_id, cause).unwrap();

    unsafe { INTR_INFO = Some(IntrInfo { hart_id, cause }); }

    INTR.store(true, Ordering::SeqCst);

    // msip::set_value(hart_id, false);
    writeln!(stdout, "Interrupt 5!").unwrap();

    unsafe { 
        writeln!(stdout, "Interrupt 5.1! {}", irq_number).unwrap();
        // complete
        (*pac::PLIC::ptr()).targets[hart_id].claim.write(|w| w.bits(irq_number));
        // writeln!(stdout, "Interrupt 5.2!").unwrap();
        // mstatus::clear_mie();
        writeln!(stdout, "Interrupt 5.3!").unwrap();
        mie::set_msoft();
        writeln!(stdout, "Interrupt 5.4!").unwrap();
        mie::set_mtimer();
        writeln!(stdout, "Interrupt 5.5!").unwrap();
    }
    // let irq_number = unsafe {
    //     (*pac::PLIC::ptr()).targets[hart_id].claim.read().bits()
    // };
    // writeln!(stdout, "Interrupt 5: claim: {}!", irq_number).unwrap();
    writeln!(stdout, "Interrupt 6!").unwrap();
    // mie::write(ie_flag);
    unsafe { 
        (*pac::PLIC::ptr()).targets[hart_id].threshold.write(|w| w.bits(int_threshold))
    };
    writeln!(stdout, "Interrupt 7!").unwrap();
}

static mut SHARED_STDOUT: core::mem::MaybeUninit<
    k210_hal::stdout::Stdout<k210_hal::serial::Tx<pac::UARTHS>>
> = core::mem::MaybeUninit::uninit();

#[riscv_rt::entry]
fn main() -> ! {
    let hart_id = mhartid::read();

    let p = pac::Peripherals::take().unwrap();

    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let gpiohs = p.GPIOHS.split();
    let _boot = Gpiohs::new(
        gpiohs.gpiohs0,
        fpioa.io16.into_function(fpioa::GPIOHS0)
    ).into_pull_up_input();

    // Configure clocks (TODO)
    let clocks = k210_hal::clock::Clocks::new();

    // Configure UART
    let serial = p.UARTHS.configure(115_200.bps(), &clocks);
    let (mut tx, _) = serial.split();

    let mut stdout = Stdout(&mut tx);

    writeln!(stdout, "This code is running on hart {}", mhartid::read()).unwrap();

    writeln!(stdout, "Initializing interrupts").unwrap();
    unsafe {
        writeln!(stdout, "1").unwrap();
        &(*pac::PLIC::ptr()).targets[hart_id].threshold.write(|w| w.bits(0));

        // Enable interrupts in general
        writeln!(stdout, "2").unwrap();
        mstatus::set_mie();
        // // Set the Machine-Software bit in MIE
        // mie::set_msoft();
        // Set the Machine-External bit in MIE
        writeln!(stdout, "3").unwrap();
        mie::set_mext();
    }
    writeln!(stdout, "Initialize PLIC").unwrap();
    for reg_id in 1..((65 + 32) / 32) {
        unsafe { 
            (*pac::PLIC::ptr()).target_enables[hart_id].enable[reg_id]
                .write(|w| w.bits(0));
        }
    }
    for irq_number in 1..=65 {
        unsafe { 
            (*pac::PLIC::ptr()).priority[irq_number]
                .write(|w| w.bits(0));
        }
    }
    unsafe {
        (*pac::PLIC::ptr()).targets[hart_id].threshold
            .write(|w| w.bits(0));
    }
    loop {
        let complete = unsafe { 
            (*pac::PLIC::ptr()).targets[hart_id].claim.read().bits()
        };
        writeln!(stdout, "Complete: {}", complete).ok();
        if complete == 0 {
            break;
        }
    }
    // enable both edge interrupt trigger for gpiohs0
    writeln!(stdout, "Enabling interrupt trigger for GPIOHS0").unwrap();
    unsafe {
        &(*pac::GPIOHS::ptr()).rise_ie.write(|w| w.pin0().set_bit());
        &(*pac::GPIOHS::ptr()).rise_ip.write(|w| w.pin0().set_bit());

        &(*pac::GPIOHS::ptr()).fall_ie.write(|w| w.pin0().set_bit());
        &(*pac::GPIOHS::ptr()).fall_ip.write(|w| w.pin0().set_bit());

        &(*pac::GPIOHS::ptr()).low_ie.write(|w| w.pin0().clear_bit());
        &(*pac::GPIOHS::ptr()).low_ip.write(|w| w.pin0().set_bit());

        &(*pac::GPIOHS::ptr()).high_ie.write(|w| w.pin0().clear_bit());
        &(*pac::GPIOHS::ptr()).high_ip.write(|w| w.pin0().set_bit());
    }
    // enable IRQ for gpiohs0 interrupt 
    writeln!(stdout, "Enabling IRQ for GPIOHS0").unwrap();
    unsafe {
        const IRQN_GPIOHS0_INTERRUPT: usize = 34;
        let irq_number = IRQN_GPIOHS0_INTERRUPT;
        let priority = 1;
        // should be 'modify'
        writeln!(stdout, "1").unwrap();
        &(*pac::PLIC::ptr()).priority[irq_number].write(|w| w.bits(priority));
        writeln!(stdout, "2").unwrap();
        &(*pac::PLIC::ptr()).target_enables[hart_id].enable[irq_number / 32]
            .modify(|r, w| w.bits(r.bits() | 1 << (irq_number % 32)));
        writeln!(stdout, "3").unwrap();
    }

    // verify irq write 
    for irq_number in 1..=65 {
        let enabled = unsafe {
            &(*pac::PLIC::ptr()).target_enables[hart_id].enable[irq_number / 32]
                .read().bits() & (1 << (irq_number % 32)) != 0
        };
        if !enabled { 
            continue;
        }
        let priority = unsafe {
            &(*pac::PLIC::ptr()).priority[irq_number].read().bits()
        };
        writeln!(stdout, 
            "Irq: {}; Enabled: {}; Priority: {}", 
            irq_number, enabled, priority
        ).ok();
    }

    // writeln!(stdout, "Generate IPI for core {} !", hart_id).unwrap();
    // msip::set_value(hart_id, true);

    writeln!(stdout, "Configuration finished!").unwrap();

    loop { 
        writeln!(stdout, "Waiting for interrupt").unwrap();
        unsafe { riscv::asm::wfi(); } 

        while !INTR.load(Ordering::SeqCst) {
            // use core::sync::atomic::{self, Ordering};
            // atomic::compiler_fence(Ordering::SeqCst);
        }
        INTR.store(false, Ordering::SeqCst);

        writeln!(stdout, 
            "Interrupt was triggered! hart_id: {:16X}, cause: {:16X}", 
            unsafe { INTR_INFO }.unwrap().hart_id,
            unsafe { INTR_INFO }.unwrap().cause,
        ).unwrap();
    }
}
