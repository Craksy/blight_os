use core::convert::TryInto;

use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;

pub fn decode(scancode: u8) -> Option<KeyboardEvent> {
    if let Some(key) = FromPrimitive::from_u8(scancode) {
        return Some(KeyboardEvent::Make(key));
    }

    // break keys seems to just be corresponding make, offset by 128
    if let Some(key) = FromPrimitive::from_u8(scancode + 0x80) {
        return Some(KeyboardEvent::Break(key));
    }

    None
}

pub enum KeyboardEvent {
    Make(Key),  // Press
    Break(Key), // Release
}

impl TryInto<char> for Key {
    type Error = ();
    #[rustfmt::skip]
    fn try_into(self) -> Result<char, ()> {
        match self {
            Self::Num0       => Ok('0'),
            Self::Num1       => Ok('1'),
            Self::Num2       => Ok('2'),
            Self::Num3       => Ok('3'),
            Self::Num4       => Ok('4'),
            Self::Num5       => Ok('5'),
            Self::Num6       => Ok('6'),
            Self::Num7       => Ok('7'),
            Self::Num8       => Ok('8'),
            Self::Num9       => Ok('9'),
            Self::A          => Ok('a'),
            Self::B          => Ok('b'),
            Self::C          => Ok('c'),
            Self::D          => Ok('d'),
            Self::E          => Ok('e'),
            Self::F          => Ok('f'),
            Self::G          => Ok('g'),
            Self::H          => Ok('h'),
            Self::I          => Ok('i'),
            Self::J          => Ok('j'),
            Self::K          => Ok('k'),
            Self::L          => Ok('l'),
            Self::M          => Ok('m'),
            Self::N          => Ok('n'),
            Self::O          => Ok('o'),
            Self::P          => Ok('p'),
            Self::Q          => Ok('q'),
            Self::R          => Ok('r'),
            Self::S          => Ok('s'),
            Self::T          => Ok('t'),
            Self::U          => Ok('u'),
            Self::V          => Ok('v'),
            Self::W          => Ok('w'),
            Self::X          => Ok('x'),
            Self::Y          => Ok('y'),
            Self::Z          => Ok('z'),
            Self::Minus      => Ok('-'),
            Self::Dot        => Ok('.'),
            Self::Comma      => Ok(','),
            Self::SemiColon  => Ok(';'),
            Self::Apostrophe => Ok('\''),
            Self::Tab        => Ok('\t'),
            Self::Enter      => Ok('\n'),
            Self::Spacebar   => Ok(' '),
            _                => Err(()),
        }
    }
}

#[derive(FromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum Key {
    NOKEY,        // 0x00
    Esc,          // 0x1
    Num1,         // 0x2
    Num2,         // 0x3
    Num3,         // 0x4
    Num4,         // 0x5
    Num5,         // 0x6
    Num6,         // 0x7
    Num7,         // 0x8
    Num8,         // 0x9
    Num9,         // 0xa
    Num0,         // 0xb
    Minus,        // 0xc
    Equals,       // 0xd
    Backspace,    // 0xe
    Tab,          // 0xf
    Q,            // 0x10
    W,            // 0x11
    E,            // 0x12
    R,            // 0x13
    T,            // 0x14
    Y,            // 0x15
    U,            // 0x16
    I,            // 0x17
    O,            // 0x18
    P,            // 0x19
    LeftBracket,  // 0x1a
    RightBracket, // 0x1b
    Enter,        // 0x1c
    LeftControl,  // 0x1d
    A,            // 0x1e
    S,            // 0x1f
    D,            // 0x20
    F,            // 0x21
    G,            // 0x22
    H,            // 0x23
    J,            // 0x24
    K,            // 0x25
    L,            // 0x26
    SemiColon,    // 0x27
    Apostrophe,   // 0x28
    Backtick,     // 0x29
    LeftShift,    // 0x2a
    Backslash,    // 0x2b
    Z,            // 0x2c
    X,            // 0x2d
    C,            // 0x2e
    V,            // 0x2f
    B,            // 0x30
    N,            // 0x31
    M,            // 0x32
    Comma,        // 0x33
    Dot,          // 0x34
    Slash,        // 0x35
    RightShift,   // 0x36
    KeypadStar,   // 0x37
    LeftCtrl,     // 0x38
    Spacebar,     // 0x39
    CapsLock,     // 0x3a
    F1,           // 0x3b
    F2,           // 0x3c
    F3,           // 0x3d
    F4,           // 0x3e
    F5,           // 0x3f
    F6,           // 0x40
    F7,           // 0x41
    F8,           // 0x42
    F9,           // 0x43
    F10,          // 0x44
    NumLock,      // 0x45
    ScrollLock,   // 0x46
    Keypad7,      // 0x47
    Keypad8,      // 0x48
    Keypad9,      // 0x49
    KeypadMinus,  // 0x4a
    Keypad4,      // 0x4b
    Keypad5,      // 0x4c
    Keypad6,      // 0x4d
    KeypadPlus,   // 0x4e
    Keypad1,      // 0x4f
    Keypad2,      // 0x50
    Keypad3,      // 0x51
    Keypad0,      // 0x52
    KeypadDot,    // 0x53
    F11 = 0x57,   // 0x54
    F12,          // 0x55
                  // TODO: Everything from here on are escaped sequences. i.e. 2 byte sequences initiated with 0xE0
}
