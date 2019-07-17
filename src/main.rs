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
mod gdt;
mod memory;

use bootloader::{BootInfo, entry_point};

// the main entry point for the kernel
entry_point!(kernel_entry);
fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    // initialize console and say hi
    console::init();
    println!("storm G4 booting...");

    // running tests?
    #[cfg(test)]
    test_main();

    // initialize the hardware
    gdt::init();
    interrupts::init();
    let (mut mapper, mut frame_allocator) = unsafe { memory::init(boot_info.physical_memory_offset, &boot_info.memory_map) };
    




    use x86_64::structures::paging::{Page};
    use x86_64::VirtAddr;

    // map a previously unmapped page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};






    // we should never reach this
    panic!("Nothing left to do!");
    // loop {}
}
