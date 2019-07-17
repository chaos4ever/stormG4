use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column: 0,
        row: 0,
        width: BUFFER_WIDTH,
        height: BUFFER_HEIGHT,
        color_code: ColorCode::new(Color::White, Color::Blue),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn init() {
    let mut writer = WRITER.lock();
    writer.set_color(ColorCode::new(Color::White, Color::DarkGray));
    writer.clear();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Character {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    characters: [[Volatile<Character>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column: usize,
    row: usize,
    width: usize,
    height: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // write byte to screen
                self.buffer.characters[self.row][self.column].write(Character { 
                    ascii_character: byte,
                    color_code: self.color_code
                });

                self.column += 1;

                // do we need to scroll?
                if self.column == self.width {
                    self.new_line();
                }
            }
        }
    }

    pub fn new_line(&mut self) {
        if self.row == self.height - 1 {
            self.scroll();
        }
        else {
            self.row += 1;
        }

        self.column = 0;
    }

    fn scroll(&mut self) {
        // OPT use a memory copy function instead
        for row in 1..self.height {
            for column in 0..self.width {
                let character = self.buffer.characters[row][column].read();
                self.buffer.characters[row - 1][column].write(character);
            }
        }

        self.clear_row(self.height - 1);
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Character {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for column in 0..self.width {
            self.buffer.characters[row][column].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..self.height {
            self.clear_row(row);
        }
    }

    pub fn set_color(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);
        Ok(())
    }
}