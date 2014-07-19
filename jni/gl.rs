extern crate libc;
use libc::{c_char, c_uint, uint8_t};
use std::c_str::CString;

type Enum = c_uint;

// glGetString enums:
pub static VENDOR: Enum = 0x1F00;
pub static RENDERER: Enum = 0x1F01;
pub static VERSION: Enum = 0x1F02;
pub static EXTENSIONS: Enum = 0x1F03;
pub static SHADING_LANGUAGE_VERSION: Enum = 0x8B8C;

// glEnable and glDisable enums:
pub static CULL_FACE: Enum = 0x0B44;
pub static DEPTH_TEST: Enum = 0x0B71;
pub static STENCIL_TEST: Enum = 0x0B90;
pub static DITHER: Enum = 0x0BD0;
pub static BLEND: Enum = 0x0BE2;
pub static SCISSOR_TEST: Enum = 0x0C11;
pub static POLYGON_OFFSET_FILL: Enum = 0x8037;
pub static SAMPLE_ALPHA_TO_COVERAGE: Enum = 0x809E;
pub static SAMPLE_COVERAGE: Enum = 0x80A0;

// Error codes.
static NO_ERROR: Enum = 0;
static INVALID_ENUM: Enum = 0x0500;
static INVALID_VALUE: Enum = 0x0501;
static INVALID_OPERATION: Enum = 0x0502;
static OUT_OF_MEMORY: Enum = 0x0505;

type UByte = uint8_t;

#[deriving(Show)]
enum Error {
  NoError,
  InvalidEnum,
  InvalidValue,
  InvalidOperation,
  InvalidFramebufferOperation,
  OutOfMemory,
}

pub fn get_string(name: Enum) -> Result<CString, Error> {
  unsafe {
    let c_ptr = glGetString(name) as *const c_char;
    if c_ptr != 0 as *const c_char {
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

extern {
  fn glGetString(name: Enum) -> *const UByte;
  fn glGetError() -> Enum;
  fn glEnable(cap: Enum);
  fn glDisable(cap: Enum);
}
