#![allow(dead_code)]

use alloc::{collections::VecDeque, string::String};
use core::fmt::{self, Write};
use lazy_static::lazy_static;
use num::CheckedAdd;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

const BUFFER_HIST: usize = 500;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column: 0,
        color: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        history: BufferHistory {
            lines: VecDeque::new()
        },
        scroll: 0
    });
}

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
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    character: u8,
    color: ColorCode,
}

//#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BufferHistory {
    lines: VecDeque<[ScreenChar; BUFFER_WIDTH]>,
}

pub struct Writer {
    column: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
    // history: Option<VecDeque<[Volatile<ScreenChar>; BUFFER_WIDTH]>>,
    history: BufferHistory,
    scroll: i16,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        if self.scroll != 0 {
            self.scroll = 0;
            self.print_hist();
        }
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let color = self.color;
                let column = self.column;
                self.buffer.chars[row][column].write(ScreenChar {
                    character: byte,
                    color,
                });
                self.column += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn print_hist(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            let hist_row = self.history.lines.len() - BUFFER_HEIGHT + row + self.scroll as usize;
            for col in 0..BUFFER_WIDTH {
                let c = self.history.lines[hist_row][col];
                self.buffer.chars[row][col].write(c);
            }
        }
    }

    pub fn scroll(&mut self, amount: i16) {
        let change = self.scroll.saturating_add(amount);
        if self.scroll != change && change == change.clamp(0, self.history.lines.len() as i16 - 1) {
            self.scroll = change;
            self.print_hist();
        }
    }

    fn new_line(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let mut hline = [ScreenChar {
            character: 0u8,
            color: self.color,
        }; BUFFER_WIDTH];
        for col in 0..BUFFER_WIDTH {
            let c = self.buffer.chars[row][col].read();
            hline[col] = c;
        }
        self.history.lines.push_back(hline);
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let empty = ScreenChar {
            character: b' ',
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(empty);
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn scroll_buffer(amount: i16) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().scroll(amount);
    });
}

// Tests

#[test_case]
fn can_println() {
    println!("Asserting print");
}

#[test_case]
fn can_println_long() {
    for _ in 1..100 {
        println!("Printing something that will cause the buffer to overflow");
    }
}

#[test_case]
fn println_alters_buffer() {
    let s = "Testing if println! displays text";
    x86_64::instructions::interrupts::without_interrupts(|| {
        println!("\n{}", s);
        for (i, c) in s.chars().enumerate() {
            let bufchar = &WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(bufchar.character), c);
        }
    });
}

// end of tests
