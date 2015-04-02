extern crate cgmath;
extern crate libc;

use libc::{c_char, c_float, c_int, c_uchar, c_uint, c_void, uint8_t};
use std::ffi::{CStr, CString};
use std::ptr;
use std::str;

use cgmath::Matrix4;

pub type Enum = c_uint;

// glGetString enums:
#[allow(dead_code)]
pub const VENDOR: Enum = 0x1F00;
#[allow(dead_code)]
pub const RENDERER: Enum = 0x1F01;
#[allow(dead_code)]
pub const VERSION: Enum = 0x1F02;
#[allow(dead_code)]
pub const EXTENSIONS: Enum = 0x1F03;
#[allow(dead_code)]
pub const SHADING_LANGUAGE_VERSION: Enum = 0x8B8C;

// glEnable and glDisable enums:
pub const CULL_FACE: Enum = 0x0B44;
pub const DEPTH_TEST: Enum = 0x0B71;
#[allow(dead_code)]
pub const STENCIL_TEST: Enum = 0x0B90;
#[allow(dead_code)]
pub const DITHER: Enum = 0x0BD0;
#[allow(dead_code)]
pub const BLEND: Enum = 0x0BE2;
#[allow(dead_code)]
pub const SCISSOR_TEST: Enum = 0x0C11;
#[allow(dead_code)]
pub const POLYGON_OFFSET_FILL: Enum = 0x8037;
#[allow(dead_code)]
pub const SAMPLE_ALPHA_TO_COVERAGE: Enum = 0x809E;
#[allow(dead_code)]
pub const SAMPLE_COVERAGE: Enum = 0x80A0;

// Error codes.
#[allow(dead_code)]
const NO_ERROR: Enum = 0;
const INVALID_ENUM: Enum = 0x0500;
const INVALID_VALUE: Enum = 0x0501;
#[allow(dead_code)]
const INVALID_OPERATION: Enum = 0x0502;
#[allow(dead_code)]
const OUT_OF_MEMORY: Enum = 0x0505;
#[allow(dead_code)]
const INVALID_FRAMEBUFFER_OPERATION: Enum = 0x0506;

type UByte = uint8_t;
pub type Clampf = c_float;
pub type Bitfield = c_uint;
type UInt = c_uint;
type SizeI = c_int;
type Char = c_char;
pub type Int = c_int;
type Boolean = c_uchar;
type Float = c_float;
type Void = c_void;

// glClear mask bits:
pub const DEPTH_BUFFER_BIT: Enum = 0x00000100;
#[allow(dead_code)]
pub const STENCIL_BUFFER_BIT: Enum = 0x00000400;
pub const COLOR_BUFFER_BIT: Enum = 0x00004000;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
  NoError,
  InvalidEnum,
  InvalidValue,
  InvalidOperation,
  InvalidFramebufferOperation,
  OutOfMemory,
}

#[allow(dead_code)]
pub fn get_string(name: Enum) -> Result<String, Error> {
  unsafe {
    let c_str = glGetString(name) as *const c_char;
    if c_str != ptr::null() {
      return Ok(cstr_to_string(c_str));
    }
  }
  let err = unsafe { glGetError() };
  match err {
    INVALID_ENUM => Err(Error::InvalidEnum),
    _ => panic!("Unknown error from glGetString(): {}", err),
  }
}

fn cstr_to_string(c_str: *const c_char) -> String {
  let bytes = unsafe { CStr::from_ptr(c_str) }.to_bytes();
  str::from_utf8(bytes).unwrap().to_string()
}

#[allow(dead_code)]
pub fn enable(cap: Enum) {
  unsafe {
    glEnable(cap);
  }
}

#[allow(dead_code)]
pub fn disable(cap: Enum) {
  unsafe {
    glDisable(cap);
  }
}

pub fn clear_color(red: Clampf, green: Clampf, blue: Clampf, alpha: Clampf) {
  unsafe {
    glClearColor(red, green, blue, alpha);
  }
}

pub fn clear(mask: Bitfield) {
  unsafe {
    glClear(mask);
  }
}

// Shader types:
pub const FRAGMENT_SHADER: Enum = 0x8B30;
pub const VERTEX_SHADER: Enum = 0x8B31;

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
      INVALID_ENUM => Err(Error::InvalidEnum),
      _ => panic!("Unknown error from glCreateShader(): {}", err),
    }
  }
}

pub fn shader_source(shader: Shader, string: &str) {
  let string_ptr: *const Char = string.as_ptr() as *const Char;
  let lengths = string.len() as i32;  // in bytes
  unsafe {
    glShaderSource(shader, 1, &string_ptr, &lengths);
  }
}

pub fn compile_shader(shader: Shader) {
  unsafe {
    glCompileShader(shader);
  }
}

// Shader parameter names:
#[allow(dead_code)]
pub const SHADER_TYPE: Enum = 0x8B4F;
pub const COMPILE_STATUS: Enum = 0x8B81;
#[allow(dead_code)]
pub const SHADER_SOURCE_LENGTH: Enum = 0x8B88;

// Both shader and program parameter names:
#[allow(dead_code)]
pub const DELETE_STATUS: Enum = 0x8B80;
pub const INFO_LOG_LENGTH: Enum = 0x8B84;

// Boolean values:
pub const FALSE: Int = 0;
pub const TRUE: Int = 1;

pub fn get_shader_param(shader: Shader, param_name: Enum) -> Int {
  let mut out_param: Int = 0;
  unsafe {
    glGetShaderiv(shader, param_name, &mut out_param);
  }
  out_param
}

pub fn get_compile_status(shader: Shader) -> bool {
  match get_shader_param(shader, COMPILE_STATUS) {
    TRUE => true,
    FALSE => false,
    i => panic!("Unknown result from get_shader_param(COMPILE_STATUS): {}", i),
  }
}

pub fn get_shader_info_log(shader: Shader) -> String {
  let buffer_size = get_shader_param(shader, INFO_LOG_LENGTH);
  let mut buff = vec![0; buffer_size as usize];
  unsafe {
    glGetShaderInfoLog(shader, buffer_size, ptr::null_mut(), buff.as_mut_ptr());
  }
  string_from_chars(&buff)
}

fn string_from_chars(chars: &[i8]) -> String {
  chars.iter().map(|c| *c as u8 as char).collect()
}

pub fn delete_shader(shader: Shader) {
  unsafe {
    glDeleteShader(shader);
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
    panic!("Unknown error from glCreateProgram(): {}", err);
  }
}

pub fn attach_shader(program: Program, shader: Shader) {
  unsafe {
    glAttachShader(program, shader);
  }
}

pub fn detach_shader(program: Program, shader: Shader) {
  unsafe {
    glDetachShader(program, shader);
  }
}

pub fn bind_attrib_location(program: Program, index: u32, name: &str) {
  let name_c_string = CString::new(name).unwrap();
  unsafe {
    glBindAttribLocation(program, index, name_c_string.as_ptr() as *const i8);
  }
}

pub fn link_program(program: Program) {
  unsafe {
    glLinkProgram(program);
  }
}

// Program parameter names:
// See also: "Both shader and program parameter names".
#[allow(dead_code)]
pub const LINK_STATUS: Enum = 0x8B82;
#[allow(dead_code)]
pub const VALIDATE_STATUS: Enum = 0x8B83;
#[allow(dead_code)]
pub const ATTACHED_SHADERS: Enum = 0x8B85;
#[allow(dead_code)]
pub const ACTIVE_UNIFORMS: Enum = 0x8B86;
#[allow(dead_code)]
pub const ACTIVE_UNIFORM_MAX_LENGTH: Enum = 0x8B87;
#[allow(dead_code)]
pub const ACTIVE_ATTRIBUTES: Enum = 0x8B89;
#[allow(dead_code)]
pub const ACTIVE_ATTRIBUTE_MAX_LENGTH: Enum = 0x8B8A;

pub fn get_program_param(program: Program, param_name: Enum) -> Int {
  let mut out_param: Int = 0;
  unsafe {
    glGetProgramiv(program, param_name, &mut out_param);
  }
  out_param
}

pub fn get_link_status(program: Program) -> bool {
  match get_program_param(program, LINK_STATUS) {
    TRUE => true,
    FALSE => false,
    i => panic!("Unknown result from get_program_param(LINK_STATUS): {}", i),
  }
}

pub fn get_program_info_log(program: Program) -> String {
  let buffer_size = get_program_param(program, INFO_LOG_LENGTH);
  let mut buff = vec![0; buffer_size as usize];
  unsafe {
    glGetProgramInfoLog(program, buffer_size, ptr::null_mut(), buff.as_mut_ptr());
  }
  string_from_chars(&buff)
}

pub fn delete_program(program: Program) {
  unsafe {
    glDeleteProgram(program);
  }
}

pub type UnifLoc = Int;

pub fn get_uniform_location(program: Program, name: &str) -> Result<UnifLoc, Error> {
  let name_c_string = CString::new(name).unwrap();
  let res = unsafe {
    glGetUniformLocation(program, name_c_string.as_ptr())
  };
  if res >= 0 {
    Ok(res)
  } else {
    let err = unsafe { glGetError() };
    match err {
      INVALID_VALUE => Err(Error::InvalidValue),
      INVALID_OPERATION => Err(Error::InvalidOperation),
      _ => panic!("Unknown error from glGetUniformLocation(): {}", err),
    }
  }
}

pub type AttribLoc = Int;

pub fn get_attrib_location(program: Program, name: &str) -> Result<AttribLoc, Error> {
  let name_c_string = CString::new(name).unwrap();
  let res = unsafe {
    glGetAttribLocation(program, name_c_string.as_ptr())
  };
  if res >= 0 {
    Ok(res)
  } else {
    let err = unsafe { glGetError() };
    match err {
      INVALID_VALUE => Err(Error::InvalidValue),
      INVALID_OPERATION => Err(Error::InvalidOperation),
      _ => panic!("Unknown error from glGetAttribLocation(): {}", err),
    }
  }
}

pub fn use_program(program: Program) {
  unsafe {
    glUseProgram(program);
  }
}

pub fn viewport(x: i32, y: i32, width: i32, height: i32) {
  unsafe {
    glViewport(x, y, width, height);
  }
}

pub fn uniform_matrix4_f32(location: UnifLoc, matrix: &Matrix4<f32>) {
  unsafe {
    use cgmath::Array2;
    glUniformMatrix4fv(location, 1, FALSE as u8, matrix.ptr());
  }
}

pub fn uniform_int(location: UnifLoc, value: Int) {
  unsafe {
    glUniform1i(location, value);
  }
}

// Data types:
#[allow(dead_code)]
const BYTE: Enum = 0x1400;
#[allow(dead_code)]
const UNSIGNED_BYTE: Enum = 0x1401;
#[allow(dead_code)]
const SHORT: Enum = 0x1402;
#[allow(dead_code)]
const UNSIGNED_SHORT: Enum = 0x1403;
#[allow(dead_code)]
const INT: Enum = 0x1404;
#[allow(dead_code)]
const UNSIGNED_INT: Enum = 0x1405;
const FLOAT: Enum = 0x1406;
#[allow(dead_code)]
const FIXED: Enum = 0x140C;

pub fn vertex_attrib_pointer_f32(location: AttribLoc, components: i32, stride: i32, values: &[f32]) {
  unsafe {
    glVertexAttribPointer(location as u32, components, FLOAT, FALSE as u8, stride,
      values.as_ptr() as *const Void);
  }
}

pub fn enable_vertex_attrib_array(location: AttribLoc) {
  unsafe {
    glEnableVertexAttribArray(location as u32);
  }
}

// glDrawArrays modes:
const TRIANGLES: Enum = 0x0004;

pub fn draw_arrays_triangles(count: i32) {
  unsafe {
    glDrawArrays(TRIANGLES, 0, count);
  }
}

pub type Texture = UInt;

pub fn gen_texture() -> Texture {
  let mut texture: Texture = 0;
  unsafe {
    glGenTextures(1, &mut texture);
  }
  texture
}

const TEXTURE_2D: Enum = 0x0DE1;

pub fn bind_texture_2d(texture: Texture) {
  unsafe {
    glBindTexture(TEXTURE_2D, texture);
  }
}

// Texture parameter names:
pub const TEXTURE_MAG_FILTER: Enum = 0x2800;
pub const TEXTURE_MIN_FILTER: Enum = 0x2801;
pub const TEXTURE_WRAP_S: Enum = 0x2802;
pub const TEXTURE_WRAP_T: Enum = 0x2803;

// Texture parameter values for TEXTURE_MAG_FILTER:
#[allow(dead_code)]
pub const NEAREST: Int = 0x2600;
pub const LINEAR: Int = 0x2601;

// Texture parameter values for TEXTURE_MIN_FILTER:
#[allow(dead_code)]
pub const NEAREST_MIPMAP_NEAREST: Int = 0x2700;
#[allow(dead_code)]
pub const LINEAR_MIPMAP_NEAREST: Int = 0x2701;
#[allow(dead_code)]
pub const NEAREST_MIPMAP_LINEAR: Int = 0x2702;
pub const LINEAR_MIPMAP_LINEAR: Int = 0x2703;

// Texture parameter values for TEXTURE_WRAP_S, TEXTURE_WRAP_T:
#[allow(dead_code)]
pub const REPEAT: Int = 0x2901;
pub const CLAMP_TO_EDGE: Int = 0x812F;
#[allow(dead_code)]
pub const MIRRORED_REPEAT: Int = 0x8370;

pub fn texture_2d_param(param_name: Enum, param_value: Int) {
  unsafe {
    glTexParameteri(TEXTURE_2D, param_name, param_value);
  }
}

const RGBA: Enum = 0x1908;

pub fn texture_2d_image_rgba(width: Int, height: Int, data: &[u8]) {
  unsafe {
    glTexImage2D(TEXTURE_2D, 0, RGBA as i32, width, height, 0, RGBA, UNSIGNED_BYTE, data.as_ptr() as *const Void);
  }
}

pub fn generate_mipmap_2d() {
  unsafe {
    glGenerateMipmap(TEXTURE_2D);
  }
}

// Texture units:
pub const TEXTURE0: Enum = 0x84C0;

pub fn active_texture(texture_unit: Enum) {
  unsafe {
    glActiveTexture(texture_unit);
  }
}

#[link(name = "GLESv2")]
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
  fn glDetachShader(program: UInt, shader: UInt);
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
  fn glUniform1i(location: Int, value: Int);
  fn glVertexAttribPointer(index: UInt, size: Int, data_type: Enum, normalized: Boolean, stride: SizeI, pointer: *const Void);
  fn glEnableVertexAttribArray(index: UInt);
  fn glDrawArrays(mode: Enum, first: Int, count: SizeI);
  fn glGenTextures(count: SizeI, textures: *mut UInt);
  fn glBindTexture(target: Enum, texture: UInt);
  fn glTexParameteri(target: Enum, param_name: Enum, param_value: Int);
  fn glTexImage2D(target: Enum, level: Int, internal_format: Int, width: SizeI, height: SizeI, border: Int, format: Enum, data_type: Enum, data: *const Void);
  fn glGenerateMipmap(target: Enum);
  fn glActiveTexture(texture_unit: Enum);
}
