extern crate libc;
use libc::{c_char, c_uint, uint8_t};
use std::c_str::CString;

type Enum = c_uint;
pub static VENDOR: Enum = 0x1F00;
pub static RENDERER: Enum = 0x1F01;
pub static VERSION: Enum = 0x1F02;
pub static EXTENSIONS: Enum = 0x1F03;
pub static SHADING_LANGUAGE_VERSION: Enum = 0x8B8C;

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

extern {
  fn glGetString(name: Enum) -> *const UByte;
  fn glGetError() -> Enum;
}
