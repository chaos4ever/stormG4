#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod panic_handler;
mod console;
#[cfg(test)]
mod tests;
mod qemu;
mod serial;
mod interrupts;

// the main entry point for the kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // initialize console and say hi
    console::init();
    println!("storm G4 booting...");

    // running tests?
    #[cfg(test)]
    test_main();

    // initialize the hardware
    interrupts::init();



    x86_64::instructions::interrupts::int3();


    // we should never reach this
    panic!("Nothing left to do!");
    // loop {}
}
