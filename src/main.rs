#![no_std]
#![no_main]

mod vga;

use crate::vga::text::{colour::Colour, writer::WRITER};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    WRITER.lock().write_string("Hello\nWorld\rHow\r\nAre You");
    WRITER.lock().set_colour(Colour::White, Colour::Green);
    WRITER.lock().write_string("Boop.\n");

    loop {}
}
