#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    BackSpace = 0x08,
    Tab = 0x09,
    Clear = 0x0C,
    Enter = 0x0D,
    Shift = 0x10,
    Ctrl = 0x11,

    Menu = 0x12,
    Pause = 0x13,
    Caps = 0x14,

    Esc = 0x1B,

    Convert = 0x1C,
    NonConvert = 0x1D,
    Accept = 0x1E,
    ModeChange = 0x1F,

    Space = 0x20,
    Prior = 0x21,
    Next = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    Select = 0x29,
    Print = 0x2A,
    Execute = 0x2B,
    Snapshot = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,
    Help = 0x2F,

    Digit0 = 0x30,
    Digit1 = 0x31,
    Digit2 = 0x32,
    Digit3 = 0x33,
    Digit4 = 0x34,
    Digit5 = 0x35,
    Digit6 = 0x36,
    Digit7 = 0x37,
    Digit8 = 0x38,
    Digit9 = 0x39,

    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,

    LWin = 0x5B,
    RWin = 0x5C,
    Apps = 0x5D,

    Sleep = 0x5F,

    Num0 = 0x60,
    Num1 = 0x61,
    Num2 = 0x62,
    Num3 = 0x63,
    Num4 = 0x64,
    Num5 = 0x65,
    Num6 = 0x66,
    Num7 = 0x67,
    Num8 = 0x68,
    Num9 = 0x69,
    Multiply = 0x6A,
    Add = 0x6B,
    Seperator = 0x6C,
    Subtract = 0x6D,
    Decimal = 0x6E,
    Divide = 0x6F,

    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,

    NumLock = 0x90,
    Scroll = 0x91,

    NumEqual = 0x92,

    LShift = 0xA0,
    RShift = 0xA1,
    LCtrl = 0xA2,
    RCtrl = 0xA3,
    LAlt = 0xA4,
    RAlt = 0xA5,

    VolumeMute = 0xAD,
    VolumeDown = 0xAE,
    VolumeUp = 0xAF,

    MediaNext = 0xB0,
    MediaPause = 0xB1,
    MediaPrev = 0xB2,
    MediaStop = 0xB3,

    Semicolon = 0xBA,
    Plus = 0xBB,
    Comma = 0xBC,
    Minus = 0xBD,
    Period = 0xBE,
    Slash = 0xBF,
    Grave = 0xC0,

    Bracket = 0xE2,

    PageDown = 0xE3,
    PageUp = 0xE4,

    Unidentified = 0x0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MouseButton {
    Left,
    Right,
    Middle,

    /// max Value 252
    Custom(u8),
}

impl MouseButton {
    #[inline]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Middle => 2,
            Self::Custom(x) => x.wrapping_add(3),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseWheelDirection {
    Up = 1,
    Down = -1,
}
