pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

use super::writer::ScreenChar;

#[repr(transparent)]
pub struct Buffer {
    pub(super) chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
