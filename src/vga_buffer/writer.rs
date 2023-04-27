use crate::vga_buffer::*;
use core::fmt;
use vga::{
    colors::TextModeColor,
    writers::{ScreenCharacter, Text80x25, TextWriter},
};
use volatile::Volatile;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]

pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub column_position: usize,
    pub color_code: TextModeColor,
    pub mode: Text80x25,
}

impl Writer {
    pub fn new(column_pos: usize, color_code: TextModeColor) -> Writer {
        let writer = Writer {
            column_position: column_pos,
            color_code,
            mode: Text80x25::new(),
        };

        writer.mode.set_mode();
        writer.mode.clear_screen();
        writer
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.mode
                    .write_character(col, row, ScreenCharacter::new(byte, color_code));
                self.column_position += 1;
            }
        }
    }

    pub fn draw_cursor(&mut self) {
        self.mode
            .set_cursor_position(self.column_position, BUFFER_HEIGHT - 1);
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.mode.read_character(col, row);
                self.mode.write_character(col, row - 1, character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenCharacter::new(b' ', self.color_code);
        for col in 0..BUFFER_WIDTH {
            self.mode.write_character(col, row, blank);
        }
    }

    pub fn delete_char(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            self.write_byte(b' ');
            self.column_position -= 1;
        }
        self.draw_cursor();
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
