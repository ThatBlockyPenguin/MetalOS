use super::buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, Buffer};
use super::colour::Colour;
use super::colour::ColourCode;
use core::fmt;
use spin::LazyLock;
use spin::mutex::FairMutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    x: usize,
    y: usize,
}

pub struct Writer {
    cursor: Cursor,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
    // is_interpreting_ansi: bool = false,
    // ansi_buffer: BufMut = BufMut {},
}

impl Writer {
    const fn is_valid_char(byte: u8) -> bool {
        //                     ESC | ASCII chars
        return matches!(byte, 0x1b | 0x20..=0x7e | b'\n' | b'\r');
    }

    fn set_char(&mut self, row: usize, col: usize, char: ScreenChar) {
        unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row][col], char) };
    }

    pub fn set_colour(&mut self, foreground: Colour, background: Colour) {
        self.colour_code = ColourCode::new(foreground, background);
    }

    pub fn write_byte(&mut self, byte: u8) {
        // if self.is_interpreting_ansi {
        //     self.ansi_buffer += byte;
        // }

        match byte {
            // Newline
            b'\n' => self.new_line(),
            // Carriage Return
            b'\r' => self.cursor.x = 0,
            // // ANSI Escape Codes
            // 0x1b => self.is_interpreting_ansi = true,
            // Other
            byte => {
                if self.cursor.x >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.cursor.x;

                let char = ScreenChar {
                    ascii_character: byte,
                    colour_code: self.colour_code,
                };

                self.set_char(row, col, char);
                self.cursor.x += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Loop over every row except the first, replacing the previous row with the current one
        for row in 1..BUFFER_HEIGHT {
            self.buffer.chars[row - 1] = self.buffer.chars[row];
        }

        // Then blank out the last row
        self.clear_row(BUFFER_HEIGHT - 1);
        // And set the column to 0
        self.cursor.x = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.set_char(row, col, blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if Self::is_valid_char(byte) {
                // printable ASCII byte or control character
                self.write_byte(byte);
            } else {
                // not part of printable ASCII range
                self.write_byte(0xfe);
            }
        }
    }
}

impl fmt::Write for Writer {
    #[doc(hidden)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static WRITER: LazyLock<FairMutex<Writer>> = LazyLock::new(|| {
    FairMutex::new(Writer {
        cursor: Cursor { x: 0, y: 0 },
        colour_code: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    })
});

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::text::writer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
