#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use k210_hal::{
    fpioa,
    gpiohs::Edge,
    pac::{self, Interrupt},
    plic::*,
    prelude::*,
    stdout::Stdout,
};
use panic_halt as _;
use riscv::register::{mcause, mhartid, mie, mstatus};

static INTR: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Copy, Clone)]
struct IntrInfo {
    hart_id: usize,
    cause: usize,
}

static mut INTR_INFO: Option<IntrInfo> = None;

#[export_name = "MachineExternal"]
fn my_trap_handler() {
    let hart_id = mhartid::read();
    let threshold = pac::PLIC::get_threshold(hart_id);

    let irq = pac::PLIC::claim(hart_id).unwrap();
    let prio = pac::PLIC::get_priority(irq);
    unsafe {
        pac::PLIC::set_threshold(hart_id, prio);
        mie::clear_msoft();
        mie::clear_mtimer();
    }

    // actual handle process starts
    let stdout = unsafe { &mut *SHARED_STDOUT.as_mut_ptr() };
    let gpiohs0 = unsafe { &mut *GPIOHS0.as_mut_ptr() };

    let cause = mcause::read().bits();

    writeln!(
        stdout,
        "[Interrupt] Hart #{}, Cause: {:016X}, Edges: {:?}",
        hart_id,
        cause,
        gpiohs0.check_edges()
    )
    .ok();

    unsafe {
        INTR_INFO = Some(IntrInfo { hart_id, cause });
    }

    INTR.store(true, Ordering::SeqCst);

    gpiohs0.clear_interrupt_pending_bits();
    // actual handle process ends

    unsafe {
        mie::set_msoft();
        mie::set_mtimer();
        pac::PLIC::set_threshold(hart_id, threshold);
    }
    pac::PLIC::complete(hart_id, irq);
}

static mut SHARED_STDOUT: core::mem::MaybeUninit<
    k210_hal::stdout::Stdout<k210_hal::serial::Tx<pac::UARTHS>>,
> = core::mem::MaybeUninit::uninit();
static mut GPIOHS0: core::mem::MaybeUninit<
    k210_hal::gpiohs::Gpiohs0<k210_hal::gpiohs::Input<k210_hal::gpiohs::PullUp>>,
> = core::mem::MaybeUninit::uninit();

#[riscv_rt::entry]
fn main() -> ! {
    let p = unsafe { pac::Peripherals::steal() };

    let mut sysctl = p.SYSCTL.constrain();
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    let gpiohs = p.GPIOHS.split();
    fpioa.io16.into_function(fpioa::GPIOHS0);
    let mut boot = gpiohs.gpiohs0.into_pull_up_input();

    // Configure clocks (TODO)
    let clocks = k210_hal::clock::Clocks::new();

    // Configure UART
    let serial = p.UARTHS.configure(115_200.bps(), &clocks);
    let (mut tx, _) = serial.split();

    let mut stdout = Stdout(&mut tx);

    writeln!(stdout, "This code is running on hart {}", mhartid::read()).ok();

    writeln!(stdout, "Initializing interrupts").ok();
    let hart_id = mhartid::read();
    unsafe {
        // set PLIC threshold for current core
        pac::PLIC::set_threshold(hart_id, Priority::P0);
        // Enable interrupts in general
        mstatus::set_mie();
        // Set the Machine-External bit in MIE
        mie::set_mext();
    }

    writeln!(stdout, "Enabling interrupt trigger for GPIOHS0").ok();
    boot.trigger_on_edge(Edge::RISING | Edge::FALLING);

    // enable IRQ for gpiohs0 interrupt
    writeln!(stdout, "Enabling IRQ for GPIOHS0").ok();
    unsafe {
        pac::PLIC::set_priority(Interrupt::GPIOHS0, Priority::P1);
        pac::PLIC::unmask(hart_id, Interrupt::GPIOHS0);
    }

    writeln!(stdout, "Configuration finished!").ok();

    loop {
        writeln!(stdout, "Waiting for interrupt").ok();
        unsafe {
            riscv::asm::wfi();
        }

        while !INTR.load(Ordering::SeqCst) {
            use core::sync::atomic::{self, Ordering};
            atomic::compiler_fence(Ordering::SeqCst);
        }
        INTR.store(false, Ordering::SeqCst);

        writeln!(
            stdout,
            "Interrupt was triggered! hart_id: {}, cause: {:16X}",
            unsafe { INTR_INFO }.unwrap().hart_id,
            unsafe { INTR_INFO }.unwrap().cause,
        )
        .ok();
    }
}
