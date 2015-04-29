use std::{error, fmt};

use gl;
use gl::{AttribLoc, Enum, UnifLoc};

pub struct VertexArray<'a> {
  pub data: &'a [f32],
  pub components: u32,
  pub stride: u32,
}

pub struct Program {
  id: gl::Program,
  vertex_shader: Shader,
  fragment_shader: Shader,
  pub mvp_matrix: UnifLoc,
  position: AttribLoc,
  pub texture_unit: UnifLoc,
  texture_coord: AttribLoc,
}

impl Drop for Program {
  fn drop(&mut self) {
    gl::disable_vertex_attrib_array(self.position);
    gl::disable_vertex_attrib_array(self.texture_coord);
    gl::detach_shader(self.id, self.vertex_shader.id);
    gl::detach_shader(self.id, self.fragment_shader.id);
    gl::delete_program(self.id);
  }
}

impl Program {
  pub fn new() -> Result<Program, GlError> {
    match Shader::new(VERTEX_SHADER, gl::VERTEX_SHADER) {
      Err(e) => Err(e),
      Ok(vs) => {
        match Shader::new(FRAGMENT_SHADER, gl::FRAGMENT_SHADER) {
          Err(e) => Err(e),
          Ok(fs) => Program::new_from_shaders(vs, fs),
        }
      }
    }
  }

  fn new_from_shaders(vertex_shader: Shader, fragment_shader: Shader) ->
    Result<Program, GlError> {

    let id = match gl::create_program() {
      Ok(id) => id,
      Err(e) => return Err(GlError::Generic(format!("gl::create_program() failed: {:?}", e))),
    };

    gl::attach_shader(id, vertex_shader.id);
    gl::attach_shader(id, fragment_shader.id);
    gl::bind_attrib_location(id, 0, "a_Position");
    gl::bind_attrib_location(id, 1, "a_TextureCoord");
    gl::link_program(id);

    if !gl::get_link_status(id) {
      let info_log = gl::get_program_info_log(id);
      return Err(GlError::Generic(format!("Linking program failed: {}", info_log)));
    }

    let mvp_matrix = match gl::get_uniform_location(id, "u_MVPMatrix") {
      Ok(l) => l,
      Err(e) => return Err(GlError::Generic(format!("gl::get_uniform_location(\"u_MVPMatrix\") failed: {:?}", e))),
    };
    let position = match gl::get_attrib_location(id, "a_Position") {
      Ok(l) => l,
      Err(e) => return Err(GlError::Generic(format!("gl::get_attrib_location(\"a_Position\") failed: {:?}", e))),
    };
    let texture_unit = match gl::get_uniform_location(id, "u_TextureUnit") {
      Ok(l) => l,
      Err(e) => return Err(GlError::Generic(format!("gl::get_uniform_location(\"u_TextureUnit\") failed: {:?}", e))),
    };
    let texture_coord = match gl::get_attrib_location(id, "a_TextureCoord") {
      Ok(l) => l,
      Err(e) => return Err(GlError::Generic(format!("gl::get_attrib_location(\"a_TextureCoord\") failed: {:?}", e))),
    };
    gl::use_program(id);

    let program = Program {
      id: id,
      vertex_shader: vertex_shader,
      fragment_shader: fragment_shader,
      mvp_matrix: mvp_matrix,
      position: position,
      texture_unit: texture_unit,
      texture_coord: texture_coord,
    };
    Ok(program)
  }

  /// Set the vertex attributes for position and texture coordinate.
  pub fn set_vertices(&self, vertex_coords: &VertexArray, texture_coords: &VertexArray) {
    gl::vertex_attrib_pointer_f32(self.position, vertex_coords.components as i32,
      vertex_coords.stride as i32, vertex_coords.data);
    gl::enable_vertex_attrib_array(self.position);

    gl::vertex_attrib_pointer_f32(self.texture_coord, texture_coords.components as i32,
      texture_coords.stride as i32, texture_coords.data);
    gl::enable_vertex_attrib_array(self.texture_coord);
  }
}

pub struct Shader {
  id: gl::Shader,
}

impl Drop for Shader {
  fn drop(&mut self) {
    gl::delete_shader(self.id);
  }
}

impl Shader {
  fn new(shader_string: &str, shader_type: Enum) -> Result<Shader, GlError> {
    let id = {
      match gl::create_shader(shader_type) {
        Ok(id) => id,
        Err(_) => return Err(GlError::Generic("gl::create_shader() failed".to_string())),
      }
    };
    let shader = Shader { id: id };
    gl::shader_source(shader.id, shader_string);
    gl::compile_shader(shader.id);

    if !gl::get_compile_status(shader.id) {
      let info_log = gl::get_shader_info_log(shader.id);
      return Err(GlError::Generic(format!("Compiling shader {} failed: {}", shader_type, info_log)));
    }
    Ok(shader)
  }
}

/// Error that can happen while calling GL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlError {
  Generic(String),
  ShaderCompileFailed(String),
  ProgramLinkFailed(String),
  NotSupported,
}

impl GlError {
  fn to_string(&self) -> &str {
    match *self {
      GlError::Generic(ref text) => &text,
      GlError::ShaderCompileFailed(ref text) => &text,
      GlError::ProgramLinkFailed(ref text) => &text,
      GlError::NotSupported => "Some of the requested attributes are not supported",
    }
  }
}

impl fmt::Display for GlError {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    formatter.write_str(self.to_string())
  }
}

impl error::Error for GlError {
  fn description(&self) -> &str {
    self.to_string()
  }
}

#[cfg(target_os = "android")]
static VERTEX_SHADER: &'static str = include_str!("vertex_shader.gles.glsl");
#[cfg(target_os = "android")]
static FRAGMENT_SHADER: &'static str = include_str!("fragment_shader.gles.glsl");

#[cfg(target_os = "linux")]
static VERTEX_SHADER: &'static str = include_str!("vertex_shader.mesa.glsl");
#[cfg(target_os = "linux")]
static FRAGMENT_SHADER: &'static str = include_str!("fragment_shader.mesa.glsl");
