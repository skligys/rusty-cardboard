extern crate libc;
use libc::{c_char, c_float, c_uint, uint8_t};
use std::c_str::CString;
use std::ptr;

type Enum = c_uint;

// glGetString enums:
#[allow(dead_code)]
pub static VENDOR: Enum = 0x1F00;
#[allow(dead_code)]
pub static RENDERER: Enum = 0x1F01;
#[allow(dead_code)]
pub static VERSION: Enum = 0x1F02;
#[allow(dead_code)]
pub static EXTENSIONS: Enum = 0x1F03;
#[allow(dead_code)]
pub static SHADING_LANGUAGE_VERSION: Enum = 0x8B8C;

// glEnable and glDisable enums:
pub static CULL_FACE: Enum = 0x0B44;
pub static DEPTH_TEST: Enum = 0x0B71;
#[allow(dead_code)]
pub static STENCIL_TEST: Enum = 0x0B90;
#[allow(dead_code)]
pub static DITHER: Enum = 0x0BD0;
#[allow(dead_code)]
pub static BLEND: Enum = 0x0BE2;
#[allow(dead_code)]
pub static SCISSOR_TEST: Enum = 0x0C11;
#[allow(dead_code)]
pub static POLYGON_OFFSET_FILL: Enum = 0x8037;
#[allow(dead_code)]
pub static SAMPLE_ALPHA_TO_COVERAGE: Enum = 0x809E;
#[allow(dead_code)]
pub static SAMPLE_COVERAGE: Enum = 0x80A0;

// Error codes.
static NO_ERROR: Enum = 0;
static INVALID_ENUM: Enum = 0x0500;
static INVALID_VALUE: Enum = 0x0501;
#[allow(dead_code)]
static INVALID_OPERATION: Enum = 0x0502;
#[allow(dead_code)]
static OUT_OF_MEMORY: Enum = 0x0505;

type UByte = uint8_t;
type Clampf = c_float;
type Bitfield = c_uint;

// glClear mask bits:
#[allow(dead_code)]
pub static DEPTH_BUFFER_BIT: Enum = 0x00000100;
#[allow(dead_code)]
pub static STENCIL_BUFFER_BIT: Enum = 0x00000400;
pub static COLOR_BUFFER_BIT: Enum = 0x00004000;

#[deriving(Show)]
enum Error {
  NoError,
  InvalidEnum,
  InvalidValue,
  InvalidOperation,
  InvalidFramebufferOperation,
  OutOfMemory,
}

#[allow(dead_code)]
pub fn get_string(name: Enum) -> Result<CString, Error> {
  unsafe {
    let c_ptr = glGetString(name) as *const c_char;
    if c_ptr != ptr::null() {
      // false because we don't own this string, it is static
      return Ok(CString::new(c_ptr, false));
    }
  }
  let err = unsafe { glGetError() };
  match err {
    INVALID_ENUM => Err(InvalidEnum),
    _ => fail!("Unknown error from glGetString(): {}", err),
  }
}

pub fn enable(cap: Enum) -> Result<(), Error> {
  unsafe {
    glEnable(cap);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_ENUM => Err(InvalidEnum),
    _ => fail!("Unknown error from glEnable(): {}", err),
  }
}

pub fn disable(cap: Enum) -> Result<(), Error> {
  unsafe {
    glDisable(cap);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_ENUM => Err(InvalidEnum),
    _ => fail!("Unknown error from glDisable(): {}", err),
  }
}

pub fn clear_color(red: Clampf, green: Clampf, blue: Clampf, alpha: Clampf) {
  unsafe {
    glClearColor(red, green, blue, alpha);
  }
}

pub fn clear(mask: Bitfield) -> Result<(), Error> {
  unsafe {
    glClear(mask);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    _ => fail!("Unknown error from glClear(): {}", err),
  }
}

extern {
  fn glGetString(name: Enum) -> *const UByte;
  fn glGetError() -> Enum;
  fn glEnable(cap: Enum);
  fn glDisable(cap: Enum);
  fn glClearColor(red: Clampf, green: Clampf, blue: Clampf, alpha: Clampf);
  fn glClear(mask: Bitfield);
}
