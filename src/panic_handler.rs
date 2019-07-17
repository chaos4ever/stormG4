use core::panic::PanicInfo;
use x86_64::instructions::hlt;

#[cfg(not(test))]
use crate::println;
#[cfg(not(test))]
use crate::console::{WRITER, ColorCode, Color};
#[cfg(test)]
use crate::{serial_println, qemu};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    WRITER.lock().set_color(ColorCode::new(Color::White, Color::Red));
    println!("{}", info);
    loop {
        hlt();
    }
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    qemu::exit(qemu::QEmuExitCode::Failed);
    loop {
        hlt();
    }
}