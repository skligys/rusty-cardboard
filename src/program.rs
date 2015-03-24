use std::{error, fmt};

use gl;
use mesh;

pub struct Program {
  id: gl::Program,
  vertex_shader: Shader,
  fragment_shader: Shader,
  pub mvp_matrix: gl::UnifLoc,
  #[allow(dead_code)]
  position: gl::AttribLoc,
  pub texture_unit: gl::UnifLoc,
  #[allow(dead_code)]
  texture_coord: gl::AttribLoc,
}

impl Drop for Program {
  fn drop(&mut self) {
    gl::detach_shader(self.id, self.vertex_shader.id);
    println!("***** Vertex shader(id: {}) detached", self.vertex_shader.id);
    gl::detach_shader(self.id, self.fragment_shader.id);
    println!("***** Fragment shader(id: {}) detached", self.fragment_shader.id);
    gl::delete_program(self.id);
    println!("***** Program(id: {}) deleted", self.id);
  }
}

impl Program {
  pub fn new() -> Result<Program, GlError> {
    match Shader::new(VERTEX_SHADER, gl::VERTEX_SHADER) {
      Err(e) => Err(e),
      Ok(vs) => {
        println!("***** Vertex shader(id: {}) compiled", vs.id);
        match Shader::new(FRAGMENT_SHADER, gl::FRAGMENT_SHADER) {
          Err(e) => Err(e),
          Ok(fs) => {
            println!("***** Fragment shader(id: {}) compiled", vs.id);
            Program::new_from_shaders(vs, fs)
          }
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

    println!("***** Using program(id: {})", id);

    // Set the vertex attributes for position and texture coordinate.
    {
      let (vcs, n, s) = mesh::vertex_coords();
      gl::vertex_attrib_pointer_f32(position, n, s, vcs);
    }
    gl::enable_vertex_attrib_array(position);
    {
      let (tcs, n, s) = mesh::texture_coords();
      gl::vertex_attrib_pointer_f32(texture_coord, n, s, tcs);
    }
    gl::enable_vertex_attrib_array(texture_coord);

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
}

pub struct Shader {
  id: gl::Shader,
}

impl Drop for Shader {
  fn drop(&mut self) {
    gl::delete_shader(self.id);
    println!("***** Shader(id: {}) deleted", self.id);
  }
}

impl Shader {
  fn new(shader_string: &str, shader_type: gl::Enum) -> Result<Shader, GlError> {
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

static VERTEX_SHADER: &'static str = include_str!("vertex_shader.glsl");
static FRAGMENT_SHADER: &'static str = include_str!("fragment_shader.glsl");
