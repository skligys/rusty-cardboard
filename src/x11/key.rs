use libc::c_uint;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ElementState {
  Pressed,
  Released,
}

pub type ScanCode = u8;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum VirtualKeyCode {
  /// The '1' key over the letters.
  Key1,
  /// The '2' key over the letters.
  Key2,
  /// The '3' key over the letters.
  Key3,
  /// The '4' key over the letters.
  Key4,
  /// The '5' key over the letters.
  Key5,
  /// The '6' key over the letters.
  Key6,
  /// The '7' key over the letters.
  Key7,
  /// The '8' key over the letters.
  Key8,
  /// The '9' key over the letters.
  Key9,
  /// The '0' key over the 'O' and 'P' keys.
  Key0,

  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
  I,
  J,
  K,
  L,
  M,
  N,
  O,
  P,
  Q,
  R,
  S,
  T,
  U,
  V,
  W,
  X,
  Y,
  Z,

  /// The Escape key, next to F1.
  Escape,

  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  F13,
  F14,
  F15,

  /// `Insert`, next to Backspace.
  Insert,
  Home,
  Delete,
  End,
  PageDown,
  PageUp,

  Left,
  Up,
  Right,
  Down,

  /// The Backspace key, right over Enter.
  Backspace,
  /// The Enter key.
  Return,
  /// The space bar.
  Space,

  Add,
  Apostrophe,
  At,
  Backslash,
  Colon,
  Comma,
  Equals,
  LAlt,
  LBracket,
  LControl,
  LShift,
  Period,
  RAlt,
  RBracket,
  RControl,
  RShift,
  Semicolon,
  Slash,
  Subtract,
  Tab,
}

const XK_BACKSPACE: c_uint = 0xFF08;
const XK_TAB: c_uint = 0xFF09;
const XK_RETURN: c_uint = 0xFF0D;
const XK_ESCAPE: c_uint = 0xFF1B;
const XK_DELETE: c_uint = 0xFFFF;
const XK_HOME: c_uint = 0xFF50;
const XK_LEFT: c_uint = 0xFF51;
const XK_UP: c_uint = 0xFF52;
const XK_RIGHT: c_uint = 0xFF53;
const XK_DOWN: c_uint = 0xFF54;
const XK_PAGE_UP: c_uint = 0xFF55;
const XK_PAGE_DOWN: c_uint = 0xFF56;
const XK_END: c_uint = 0xFF57;
const XK_INSERT: c_uint = 0xFF63;
const XK_F1: c_uint = 0xFFBE;
const XK_F2: c_uint = 0xFFBF;
const XK_F3: c_uint = 0xFFC0;
const XK_F4: c_uint = 0xFFC1;
const XK_F5: c_uint = 0xFFC2;
const XK_F6: c_uint = 0xFFC3;
const XK_F7: c_uint = 0xFFC4;
const XK_F8: c_uint = 0xFFC5;
const XK_F9: c_uint = 0xFFC6;
const XK_F10: c_uint = 0xFFC7;
const XK_F11: c_uint = 0xFFC8;
const XK_F12: c_uint = 0xFFC9;
const XK_F13: c_uint = 0xFFCA;
const XK_F14: c_uint = 0xFFCB;
const XK_F15: c_uint = 0xFFCC;
const XK_SHIFT_L: c_uint = 0xFFE1;
const XK_SHIFT_R: c_uint = 0xFFE2;
const XK_CONTROL_L: c_uint = 0xFFE3;
const XK_CONTROL_R: c_uint = 0xFFE4;
const XK_ALT_L: c_uint = 0xFFE9;
const XK_ALT_R: c_uint = 0xFFEA;
const XK_SPACE: c_uint = 0x020;
const XK_APOSTROPHE: c_uint = 0x027;
const XK_PLUS: c_uint = 0x02b;
const XK_COMMA: c_uint = 0x02c;
const XK_MINUS: c_uint = 0x02d;
const XK_PERIOD: c_uint = 0x02e;
const XK_SLASH: c_uint = 0x02f;
const XK_0: c_uint = 0x030;
const XK_1: c_uint = 0x031;
const XK_2: c_uint = 0x032;
const XK_3: c_uint = 0x033;
const XK_4: c_uint = 0x034;
const XK_5: c_uint = 0x035;
const XK_6: c_uint = 0x036;
const XK_7: c_uint = 0x037;
const XK_8: c_uint = 0x038;
const XK_9: c_uint = 0x039;
const XK_COLON: c_uint = 0x03a;
const XK_SEMICOLON: c_uint = 0x03b;
const XK_EQUAL: c_uint = 0x03d;
const XK_AT: c_uint = 0x040;
const XK_UPPERCASE_A: c_uint = 0x041;
const XK_UPPERCASE_B: c_uint = 0x042;
const XK_UPPERCASE_C: c_uint = 0x043;
const XK_UPPERCASE_D: c_uint = 0x044;
const XK_UPPERCASE_E: c_uint = 0x045;
const XK_UPPERCASE_F: c_uint = 0x046;
const XK_UPPERCASE_G: c_uint = 0x047;
const XK_UPPERCASE_H: c_uint = 0x048;
const XK_UPPERCASE_I: c_uint = 0x049;
const XK_UPPERCASE_J: c_uint = 0x04a;
const XK_UPPERCASE_K: c_uint = 0x04b;
const XK_UPPERCASE_L: c_uint = 0x04c;
const XK_UPPERCASE_M: c_uint = 0x04d;
const XK_UPPERCASE_N: c_uint = 0x04e;
const XK_UPPERCASE_O: c_uint = 0x04f;
const XK_UPPERCASE_P: c_uint = 0x050;
const XK_UPPERCASE_Q: c_uint = 0x051;
const XK_UPPERCASE_R: c_uint = 0x052;
const XK_UPPERCASE_S: c_uint = 0x053;
const XK_UPPERCASE_T: c_uint = 0x054;
const XK_UPPERCASE_U: c_uint = 0x055;
const XK_UPPERCASE_V: c_uint = 0x056;
const XK_UPPERCASE_W: c_uint = 0x057;
const XK_UPPERCASE_X: c_uint = 0x058;
const XK_UPPERCASE_Y: c_uint = 0x059;
const XK_UPPERCASE_Z: c_uint = 0x05a;
const XK_BRACKET_LEFT: c_uint = 0x05b;
const XK_BACKSLASH: c_uint = 0x05c;
const XK_BRACKET_RIGHT: c_uint = 0x05d;
const XK_LOWERCASE_A: c_uint = 0x061;
const XK_LOWERCASE_B: c_uint = 0x062;
const XK_LOWERCASE_C: c_uint = 0x063;
const XK_LOWERCASE_D: c_uint = 0x064;
const XK_LOWERCASE_E: c_uint = 0x065;
const XK_LOWERCASE_F: c_uint = 0x066;
const XK_LOWERCASE_G: c_uint = 0x067;
const XK_LOWERCASE_H: c_uint = 0x068;
const XK_LOWERCASE_I: c_uint = 0x069;
const XK_LOWERCASE_J: c_uint = 0x06a;
const XK_LOWERCASE_K: c_uint = 0x06b;
const XK_LOWERCASE_L: c_uint = 0x06c;
const XK_LOWERCASE_M: c_uint = 0x06d;
const XK_LOWERCASE_N: c_uint = 0x06e;
const XK_LOWERCASE_O: c_uint = 0x06f;
const XK_LOWERCASE_P: c_uint = 0x070;
const XK_LOWERCASE_Q: c_uint = 0x071;
const XK_LOWERCASE_R: c_uint = 0x072;
const XK_LOWERCASE_S: c_uint = 0x073;
const XK_LOWERCASE_T: c_uint = 0x074;
const XK_LOWERCASE_U: c_uint = 0x075;
const XK_LOWERCASE_V: c_uint = 0x076;
const XK_LOWERCASE_W: c_uint = 0x077;
const XK_LOWERCASE_X: c_uint = 0x078;
const XK_LOWERCASE_Y: c_uint = 0x079;
const XK_LOWERCASE_Z: c_uint = 0x07a;

pub fn keycode_to_element(scancode: c_uint) -> Option<VirtualKeyCode> {
  match scancode {
    XK_BACKSPACE => Some(VirtualKeyCode::Backspace),
    XK_TAB => Some(VirtualKeyCode::Tab),
    XK_RETURN => Some(VirtualKeyCode::Return),
    XK_ESCAPE => Some(VirtualKeyCode::Escape),
    XK_DELETE => Some(VirtualKeyCode::Delete),
    XK_HOME => Some(VirtualKeyCode::Home),
    XK_LEFT => Some(VirtualKeyCode::Left),
    XK_UP => Some(VirtualKeyCode::Up),
    XK_RIGHT => Some(VirtualKeyCode::Right),
    XK_DOWN => Some(VirtualKeyCode::Down),
    XK_PAGE_UP => Some(VirtualKeyCode::PageUp),
    XK_PAGE_DOWN => Some(VirtualKeyCode::PageDown),
    XK_END => Some(VirtualKeyCode::End),
    XK_INSERT => Some(VirtualKeyCode::Insert),
    XK_F1 => Some(VirtualKeyCode::F1),
    XK_F2 => Some(VirtualKeyCode::F2),
    XK_F3 => Some(VirtualKeyCode::F3),
    XK_F4 => Some(VirtualKeyCode::F4),
    XK_F5 => Some(VirtualKeyCode::F5),
    XK_F6 => Some(VirtualKeyCode::F6),
    XK_F7 => Some(VirtualKeyCode::F7),
    XK_F8 => Some(VirtualKeyCode::F8),
    XK_F9 => Some(VirtualKeyCode::F9),
    XK_F10 => Some(VirtualKeyCode::F10),
    XK_F11 => Some(VirtualKeyCode::F11),
    XK_F12 => Some(VirtualKeyCode::F12),
    XK_F13 => Some(VirtualKeyCode::F13),
    XK_F14 => Some(VirtualKeyCode::F14),
    XK_F15 => Some(VirtualKeyCode::F15),
    XK_SHIFT_L => Some(VirtualKeyCode::LShift),
    XK_SHIFT_R => Some(VirtualKeyCode::RShift),
    XK_CONTROL_L => Some(VirtualKeyCode::LControl),
    XK_CONTROL_R => Some(VirtualKeyCode::RControl),
    XK_ALT_L => Some(VirtualKeyCode::LAlt),
    XK_ALT_R => Some(VirtualKeyCode::RAlt),
    XK_SPACE => Some(VirtualKeyCode::Space),
    XK_APOSTROPHE => Some(VirtualKeyCode::Apostrophe),
    XK_PLUS => Some(VirtualKeyCode::Add),
    XK_COMMA => Some(VirtualKeyCode::Comma),
    XK_MINUS => Some(VirtualKeyCode::Subtract),
    XK_PERIOD => Some(VirtualKeyCode::Period),
    XK_SLASH => Some(VirtualKeyCode::Slash),
    XK_0 => Some(VirtualKeyCode::Key0),
    XK_1 => Some(VirtualKeyCode::Key1),
    XK_2 => Some(VirtualKeyCode::Key2),
    XK_3 => Some(VirtualKeyCode::Key3),
    XK_4 => Some(VirtualKeyCode::Key4),
    XK_5 => Some(VirtualKeyCode::Key5),
    XK_6 => Some(VirtualKeyCode::Key6),
    XK_7 => Some(VirtualKeyCode::Key7),
    XK_8 => Some(VirtualKeyCode::Key8),
    XK_9 => Some(VirtualKeyCode::Key9),
    XK_COLON => Some(VirtualKeyCode::Colon),
    XK_SEMICOLON => Some(VirtualKeyCode::Semicolon),
    XK_EQUAL => Some(VirtualKeyCode::Equals),
    XK_AT => Some(VirtualKeyCode::At),
    XK_UPPERCASE_A => Some(VirtualKeyCode::A),
    XK_UPPERCASE_B => Some(VirtualKeyCode::B),
    XK_UPPERCASE_C => Some(VirtualKeyCode::C),
    XK_UPPERCASE_D => Some(VirtualKeyCode::D),
    XK_UPPERCASE_E => Some(VirtualKeyCode::E),
    XK_UPPERCASE_F => Some(VirtualKeyCode::F),
    XK_UPPERCASE_G => Some(VirtualKeyCode::G),
    XK_UPPERCASE_H => Some(VirtualKeyCode::H),
    XK_UPPERCASE_I => Some(VirtualKeyCode::I),
    XK_UPPERCASE_J => Some(VirtualKeyCode::J),
    XK_UPPERCASE_K => Some(VirtualKeyCode::K),
    XK_UPPERCASE_L => Some(VirtualKeyCode::L),
    XK_UPPERCASE_M => Some(VirtualKeyCode::M),
    XK_UPPERCASE_N => Some(VirtualKeyCode::N),
    XK_UPPERCASE_O => Some(VirtualKeyCode::O),
    XK_UPPERCASE_P => Some(VirtualKeyCode::P),
    XK_UPPERCASE_Q => Some(VirtualKeyCode::Q),
    XK_UPPERCASE_R => Some(VirtualKeyCode::R),
    XK_UPPERCASE_S => Some(VirtualKeyCode::S),
    XK_UPPERCASE_T => Some(VirtualKeyCode::T),
    XK_UPPERCASE_U => Some(VirtualKeyCode::U),
    XK_UPPERCASE_V => Some(VirtualKeyCode::V),
    XK_UPPERCASE_W => Some(VirtualKeyCode::W),
    XK_UPPERCASE_X => Some(VirtualKeyCode::X),
    XK_UPPERCASE_Y => Some(VirtualKeyCode::Y),
    XK_UPPERCASE_Z => Some(VirtualKeyCode::Z),
    XK_BRACKET_LEFT => Some(VirtualKeyCode::LBracket),
    XK_BACKSLASH => Some(VirtualKeyCode::Backslash),
    XK_BRACKET_RIGHT => Some(VirtualKeyCode::RBracket),
    XK_LOWERCASE_A => Some(VirtualKeyCode::A),
    XK_LOWERCASE_B => Some(VirtualKeyCode::B),
    XK_LOWERCASE_C => Some(VirtualKeyCode::C),
    XK_LOWERCASE_D => Some(VirtualKeyCode::D),
    XK_LOWERCASE_E => Some(VirtualKeyCode::E),
    XK_LOWERCASE_F => Some(VirtualKeyCode::F),
    XK_LOWERCASE_G => Some(VirtualKeyCode::G),
    XK_LOWERCASE_H => Some(VirtualKeyCode::H),
    XK_LOWERCASE_I => Some(VirtualKeyCode::I),
    XK_LOWERCASE_J => Some(VirtualKeyCode::J),
    XK_LOWERCASE_K => Some(VirtualKeyCode::K),
    XK_LOWERCASE_L => Some(VirtualKeyCode::L),
    XK_LOWERCASE_M => Some(VirtualKeyCode::M),
    XK_LOWERCASE_N => Some(VirtualKeyCode::N),
    XK_LOWERCASE_O => Some(VirtualKeyCode::O),
    XK_LOWERCASE_P => Some(VirtualKeyCode::P),
    XK_LOWERCASE_Q => Some(VirtualKeyCode::Q),
    XK_LOWERCASE_R => Some(VirtualKeyCode::R),
    XK_LOWERCASE_S => Some(VirtualKeyCode::S),
    XK_LOWERCASE_T => Some(VirtualKeyCode::T),
    XK_LOWERCASE_U => Some(VirtualKeyCode::U),
    XK_LOWERCASE_V => Some(VirtualKeyCode::V),
    XK_LOWERCASE_W => Some(VirtualKeyCode::W),
    XK_LOWERCASE_X => Some(VirtualKeyCode::X),
    XK_LOWERCASE_Y => Some(VirtualKeyCode::Y),
    XK_LOWERCASE_Z => Some(VirtualKeyCode::Z),
    _ => None,
  }
}
