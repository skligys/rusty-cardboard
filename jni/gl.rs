extern crate cgmath;
extern crate libc;

use libc::{c_char, c_float, c_int, c_uchar, c_uint, c_void, uint8_t};
use std::c_str::CString;
use std::ptr;
use std::vec::Vec;

use self::cgmath::array::Array2;
use self::cgmath::matrix::Matrix4;
use self::cgmath::vector::Vector4;

use log;

pub type Enum = c_uint;

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
#[allow(dead_code)]
pub static CULL_FACE: Enum = 0x0B44;
#[allow(dead_code)]
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
static INVALID_FRAMEBUFFER_OPERATION: Enum = 0x0506;

type UByte = uint8_t;
type Clampf = c_float;
type Bitfield = c_uint;
type UInt = c_uint;
type SizeI = c_int;
type Char = c_char;
type Int = c_int;
type Boolean = c_uchar;
type Float = c_float;
type Void = c_void;

// glClear mask bits:
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
    _ => {
      log::e_f(format!("Unknown error from glGetString(): {}", err));
      fail!();
    },
  }
}

#[allow(dead_code)]
pub fn enable(cap: Enum) -> Result<(), Error> {
  unsafe {
    glEnable(cap);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_ENUM => Err(InvalidEnum),
    _ => {
      log::e_f(format!("Unknown error from glEnable(): {}", err));
      fail!();
    },
  }
}

#[allow(dead_code)]
pub fn disable(cap: Enum) -> Result<(), Error> {
  unsafe {
    glDisable(cap);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_ENUM => Err(InvalidEnum),
    _ => {
      log::e_f(format!("Unknown error from glDisable(): {}", err));
      fail!();
    },
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
    _ => {
      log::e_f(format!("Unknown error from glClear(): {}", err));
      fail!();
    },
  }
}

// Shader types:
pub static FRAGMENT_SHADER: Enum = 0x8B30;
pub static VERTEX_SHADER: Enum = 0x8B31;

pub type Shader = UInt;

pub fn create_shader(shader_type: Enum) -> Result<Shader, Error> {
  let res = unsafe {
    glCreateShader(shader_type)
  };
  if res != 0 {
    Ok(res)
  } else {
    let err = unsafe { glGetError() };
    match err {
      INVALID_ENUM => Err(InvalidEnum),
      _ => {
        log::e_f(format!("Unknown error from glCreateShader(): {}", err));
        fail!();
      },
    }
  }
}

pub fn shader_source(shader: Shader, string: &str) -> Result<(), Error> {
  let string_ptr: *const Char = string.as_ptr() as *const Char;
  let lengths = string.len() as i32;  // in bytes
  unsafe {
    glShaderSource(shader, 1, &string_ptr, &lengths);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glShaderSource(): {}", err));
      fail!();
    },
  }
}

pub fn compile_shader(shader: Shader) -> Result<(), Error> {
  unsafe {
    glCompileShader(shader);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glCompileShader(): {}", err));
      fail!();
    },
  }
}

// Shader parameter names:
#[allow(dead_code)]
pub static SHADER_TYPE: Enum = 0x8B4F;
pub static COMPILE_STATUS: Enum = 0x8B81;
#[allow(dead_code)]
pub static SHADER_SOURCE_LENGTH: Enum = 0x8B88;

// Both shader and program parameter names:
#[allow(dead_code)]
pub static DELETE_STATUS: Enum = 0x8B80;
pub static INFO_LOG_LENGTH: Enum = 0x8B84;

// Boolean values:
pub static FALSE: Int = 0;
pub static TRUE: Int = 1;

pub fn get_shader_param(shader: Shader, param_name: Enum) -> Result<Int, Error> {
  let mut out_param: Int = 0;
  unsafe {
    glGetShaderiv(shader, param_name, &mut out_param);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(out_param),
    INVALID_ENUM => Err(InvalidEnum),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glGetShaderiv(): {}", err));
      fail!();
    },
  }
}

pub fn get_compile_status(shader: Shader) -> Result<bool, Error> {
  match get_shader_param(shader, COMPILE_STATUS) {
    Ok(TRUE) => Ok(true),
    Ok(FALSE) => Ok(false),
    Ok(i) => {
      log::e_f(format!("Unknown result from get_shader_param(COMPILE_STATUS): {}", i));
      fail!();
    },
    Err(e) => Err(e),
  }
}

pub fn get_shader_info_log(shader: Shader) -> Result<String, Error> {
  match get_shader_param(shader, INFO_LOG_LENGTH) {
    Ok(buffer_size) => {
      let mut buff = Vec::from_elem(buffer_size as uint, 0);
      unsafe {
        glGetShaderInfoLog(shader, buffer_size, ptr::mut_null(), buff.as_mut_ptr());
      }
      let err = unsafe { glGetError() };
      match err {
        NO_ERROR => Ok(string_from_chars(buff.as_slice())),
        INVALID_VALUE => Err(InvalidValue),
        INVALID_OPERATION => Err(InvalidOperation),
        _ => {
          log::e_f(format!("Unknown error from glGetShaderInfoLog(): {}", err));
          fail!();
        },
      }
    },
    Err(e) => Err(e),
  }
}

fn string_from_chars(chars: &[i8]) -> String {
  chars.iter().map(|c| *c as u8 as char).collect()
}

pub fn delete_shader(shader: Shader) -> Result<(), Error> {
  unsafe {
    glDeleteShader(shader);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    _ => {
      log::e_f(format!("Unknown error from glDeleteShader(): {}", err));
      fail!();
    },
  }
}

pub type Program = UInt;

pub fn create_program() -> Result<Program, Error> {
  let res = unsafe {
    glCreateProgram()
  };
  if res != 0 {
    Ok(res)
  } else {
    let err = unsafe { glGetError() };
    log::e_f(format!("Unknown error from glCreateProgram(): {}", err));
    fail!();
  }
}

pub fn attach_shader(program: Program, shader: Shader) -> Result<(), Error> {
  unsafe {
    glAttachShader(program, shader);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glAttachShader(): {}", err));
      fail!();
    },
  }
}

pub fn bind_attrib_location(program: Program, index: u32, name: &str) -> Result<(), Error> {
  let name_c_string = name.to_c_str();
  unsafe {
    glBindAttribLocation(program, index, name_c_string.as_ptr() as *const i8);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glBindAttribLocation(): {}", err));
      fail!();
    },
  }
}

pub fn link_program(program: Program) -> Result<(), Error> {
  unsafe {
    glLinkProgram(program);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glLinkProgram(): {}", err));
      fail!();
    },
  }
}

// Program parameter names:
// See also: "Both shader and program parameter names".
#[allow(dead_code)]
pub static LINK_STATUS: Enum = 0x8B82;
#[allow(dead_code)]
pub static VALIDATE_STATUS: Enum = 0x8B83;
#[allow(dead_code)]
pub static ATTACHED_SHADERS: Enum = 0x8B85;
#[allow(dead_code)]
pub static ACTIVE_UNIFORMS: Enum = 0x8B86;
#[allow(dead_code)]
pub static ACTIVE_UNIFORM_MAX_LENGTH: Enum = 0x8B87;
#[allow(dead_code)]
pub static ACTIVE_ATTRIBUTES: Enum = 0x8B89;
#[allow(dead_code)]
pub static ACTIVE_ATTRIBUTE_MAX_LENGTH: Enum = 0x8B8A;

pub fn get_program_param(program: Program, param_name: Enum) -> Result<Int, Error> {
  let mut out_param: Int = 0;
  unsafe {
    glGetProgramiv(program, param_name, &mut out_param);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(out_param),
    INVALID_ENUM => Err(InvalidEnum),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glGetProgramiv(): {}", err));
      fail!();
    },
  }
}

pub fn get_link_status(program: Program) -> Result<bool, Error> {
  match get_program_param(program, LINK_STATUS) {
    Ok(TRUE) => Ok(true),
    Ok(FALSE) => Ok(false),
    Ok(i) => {
      log::e_f(format!("Unknown result from get_program_param(LINK_STATUS): {}", i));
      fail!();
    },
    Err(e) => Err(e),
  }
}

pub fn get_program_info_log(program: Program) -> Result<String, Error> {
  match get_program_param(program, INFO_LOG_LENGTH) {
    Ok(buffer_size) => {
      let mut buff = Vec::from_elem(buffer_size as uint, 0);
      unsafe {
        glGetProgramInfoLog(program, buffer_size, ptr::mut_null(), buff.as_mut_ptr());
      }
      let err = unsafe { glGetError() };
      match err {
        NO_ERROR => Ok(string_from_chars(buff.as_slice())),
        INVALID_VALUE => Err(InvalidValue),
        INVALID_OPERATION => Err(InvalidOperation),
        _ => {
          log::e_f(format!("Unknown error from glGetProgramInfoLog(): {}", err));
          fail!();
        },
      }
    },
    Err(e) => Err(e),
  }
}

pub fn delete_program(program: Program) -> Result<(), Error> {
  unsafe {
    glDeleteProgram(program);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    _ => {
      log::e_f(format!("Unknown error from glDeleteProgram(): {}", err));
      fail!();
    },
  }
}

pub type UnifLoc = Int;

pub fn get_uniform_location(program: Program, name: &str) -> Result<UnifLoc, Error> {
  let name_c_string = name.to_c_str();
  let res = unsafe {
    glGetUniformLocation(program, name_c_string.as_ptr())
  };
  if res >= 0 {
    Ok(res)
  } else {
    let err = unsafe { glGetError() };
    match err {
      INVALID_VALUE => Err(InvalidValue),
      INVALID_OPERATION => Err(InvalidOperation),
      _ => {
        log::e_f(format!("Unknown error from glGetUniformLocation(): {}", err));
        fail!();
      },
    }
  }
}

pub type AttribLoc = Int;

pub fn get_attrib_location(program: Program, name: &str) -> Result<AttribLoc, Error> {
  let name_c_string = name.to_c_str();
  let res = unsafe {
    glGetAttribLocation(program, name_c_string.as_ptr())
  };
  if res >= 0 {
    Ok(res)
  } else {
    let err = unsafe { glGetError() };
    match err {
      INVALID_VALUE => Err(InvalidValue),
      INVALID_OPERATION => Err(InvalidOperation),
      _ => {
        log::e_f(format!("Unknown error from glGetAttribLocation(): {}", err));
        fail!();
      },
    }
  }
}

pub fn use_program(program: Program) -> Result<(), Error> {
  unsafe {
    glUseProgram(program);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glUseProgram(): {}", err));
      fail!();
    },
  }
}

pub fn viewport(x: i32, y: i32, width: i32, height: i32) -> Result<(), Error> {
  unsafe {
    glViewport(x, y, width, height);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    _ => {
      log::e_f(format!("Unknown error from glViewport(): {}", err));
      fail!();
    },
  }
}

pub fn uniform_matrix4_f32(location: UnifLoc, matrix: &Matrix4<f32>) -> Result<(), Error> {
  let arr: &Array2<Vector4<f32>, Vector4<f32>, f32> = matrix;
  unsafe {
    glUniformMatrix4fv(location, 1, FALSE as u8, arr.ptr());
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_OPERATION => Err(InvalidOperation),
    _ => {
      log::e_f(format!("Unknown error from glUniformMatrix4fv(): {}", err));
      fail!();
    },
  }
}

// Data types:
#[allow(dead_code)]
static BYTE: Enum = 0x1400;
#[allow(dead_code)]
static UNSIGNED_BYTE: Enum = 0x1401;
#[allow(dead_code)]
static SHORT: Enum = 0x1402;
#[allow(dead_code)]
static UNSIGNED_SHORT: Enum = 0x1403;
#[allow(dead_code)]
static INT: Enum = 0x1404;
#[allow(dead_code)]
static UNSIGNED_INT: Enum = 0x1405;
static FLOAT: Enum = 0x1406;
#[allow(dead_code)]
static FIXED: Enum = 0x140C;


pub fn vertex_attrib_pointer_f32(location: AttribLoc, components: i32, stride: i32, values: &[f32]) -> Result<(), Error> {
  unsafe {
    glVertexAttribPointer(location as u32, components, FLOAT, FALSE as u8, stride, values.as_ptr() as *const Void);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_ENUM => Err(InvalidEnum),
    INVALID_VALUE => Err(InvalidValue),
    _ => {
      log::e_f(format!("Unknown error from glVertexAttribPointer(): {}", err));
      fail!();
    },
  }
}

pub fn enable_vertex_attrib_array(location: AttribLoc) -> Result<(), Error> {
  unsafe {
    glEnableVertexAttribArray(location as u32);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_VALUE => Err(InvalidValue),
    _ => {
      log::e_f(format!("Unknown error from glEnableVertexAttribArray(): {}", err));
      fail!();
    },
  }
}

// glDrawArrays modes:
static TRIANGLES: Enum = 0x0004;

pub fn draw_arrays_triangles(count: i32) -> Result<(), Error> {
  unsafe {
    glDrawArrays(TRIANGLES, 0, count);
  }
  let err = unsafe { glGetError() };
  match err {
    NO_ERROR => Ok(()),
    INVALID_ENUM => Err(InvalidEnum),
    INVALID_VALUE => Err(InvalidValue),
    INVALID_FRAMEBUFFER_OPERATION => Err(InvalidFramebufferOperation),
    _ => {
      log::e_f(format!("Unknown error from glDrawArrays(): {}", err));
      fail!();
    },
  }
}

extern {
  fn glGetString(name: Enum) -> *const UByte;
  fn glGetError() -> Enum;
  fn glEnable(cap: Enum);
  fn glDisable(cap: Enum);
  fn glClearColor(red: Clampf, green: Clampf, blue: Clampf, alpha: Clampf);
  fn glClear(mask: Bitfield);
  fn glCreateShader(shader_type: Enum) -> UInt;
  fn glShaderSource(shader: UInt, count: SizeI, strings: *const *const Char, lengths: *const Int);
  fn glCompileShader(shader: UInt);
  fn glGetShaderiv(shader: UInt, param_name: Enum, out_params: *mut Int);
  fn glGetShaderInfoLog(shader: UInt, buffer_size: SizeI, out_length: *mut SizeI, out_log: *mut Char);
  fn glDeleteShader(shader: UInt);
  fn glCreateProgram() -> UInt;
  fn glAttachShader(program: UInt, shader: UInt);
  fn glBindAttribLocation(program: UInt, index: UInt, name: *const Char);
  fn glLinkProgram(program: UInt);
  fn glGetProgramiv(program: UInt, param_name: Enum, out_params: *mut Int);
  fn glGetProgramInfoLog(program: UInt, buffer_size: SizeI, out_length: *mut SizeI, out_log: *mut Char);
  fn glDeleteProgram(program: UInt);
  fn glGetUniformLocation(program: UInt, name: *const Char) -> Int;
  fn glGetAttribLocation(program: UInt, name: *const Char) -> Int;
  fn glUseProgram(program: UInt);
  fn glViewport(x: Int, y: Int, width: SizeI, height: SizeI);
  fn glUniformMatrix4fv(location: Int, count: SizeI, transpose: Boolean, value: *const Float);
  fn glVertexAttribPointer(index: UInt, size: Int, data_type: Enum, normalized: Boolean, stride: SizeI, pointer: *const Void);
  fn glEnableVertexAttribArray(index: UInt);
  fn glDrawArrays(mode: Enum, first: Int, count: SizeI);
}
