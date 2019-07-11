use crate::{serial_print, serial_println, println, qemu};

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    qemu::exit(qemu::QEmuExitCode::Success);
}

#[test_case]
fn test_println() {
    serial_print!("testing println... ");
    println!("Some test string");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_scrolling() {
    serial_print!("testing println scrolling... ");
    for _ in 0..200 {
        println!("Some test string");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_wrapping() {
    serial_print!("testing println wrapping... ");
    for _ in 0..200 {
        println!("Some test string. Some test string. Some test string. Some test string. Some test string. Some test string. Some test string. Some test string. Some test string. Some test string. ");
    }
    serial_println!("[ok]");
}