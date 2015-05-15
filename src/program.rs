use std::{error, fmt};

use cgmath::Matrix4;

use gl;
use gl::{AttribLoc, Buffer, Enum, UnifLoc};
use mesh::{Coords, Vertices};

pub struct VertexArray {
  pub components: u32,
  pub stride: u32,
}

pub struct Buffers {
  pub vertex_buffer: Buffer,
  pub index_buffer: Buffer,
  pub index_count: i32,
}

impl Drop for Buffers {
  fn drop(&mut self) {
    // Debug.
    log!("*** Deleting buffers: {}, {}", self.vertex_buffer, self.index_buffer);

    gl::delete_buffers(&[self.vertex_buffer, self.index_buffer]);
  }
}

pub struct Program {
  id: gl::Program,
  vertex_shader: Shader,
  fragment_shader: Shader,
  mvp_matrix: UnifLoc,
  position: AttribLoc,
  texture_unit: UnifLoc,
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

  /// Uploads given vertices into GPU, returns handles to OpenGL buffers.
  pub fn upload_vertices(&self, vertices: &Vertices) -> Buffers {
    let buffers = upload_vertices(&vertices);
    gl::bind_array_buffer(buffers.vertex_buffer);

    let position_coords = vertices.position_coord_array();
    gl::vertex_attrib_pointer_f32(self.position, position_coords.components as i32,
      position_coords.stride as i32, 0);
    gl::enable_vertex_attrib_array(self.position);

    let texture_coords = vertices.texture_coord_array();
    gl::vertex_attrib_pointer_u16(self.texture_coord, texture_coords.components as i32,
      texture_coords.stride as i32, Coords::texture_offset());
    gl::enable_vertex_attrib_array(self.texture_coord);

    // Debug:
    log!("*** Triangle count: {}, vertex count: {}, bytes: {}",
      vertices.coord_count() / 2, vertices.coord_count(),
      vertices.coord_count() * Coords::size_bytes() as usize);

    buffers
  }

  pub fn set_mvp_matrix(&self, mvp_matrix: Matrix4<f32>) {
    gl::uniform_matrix4_f32(self.mvp_matrix, &mvp_matrix);
  }

  pub fn set_texture_unit(&self, unit: i32) {
    gl::uniform_int(self.texture_unit, unit);
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

fn upload_vertices(vertices: &Vertices) -> Buffers {
  match &gl::generate_buffers(2)[..] {
    [vbo, ibo] => {
      gl::bind_array_buffer(vbo);
      gl::array_buffer_data_coords(vertices.coords());
      gl::unbind_array_buffer();

      gl::bind_index_buffer(ibo);
      gl::index_buffer_data_u16(vertices.indices());
      gl::unbind_index_buffer();

      Buffers {
        vertex_buffer: vbo,
        index_buffer: ibo,
        index_count: vertices.index_count() as i32,
      }
    },
    _ => panic!("gl::generate_buffers(2) should return 2 buffers"),
  }
}
