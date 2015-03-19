extern crate android_glue;
extern crate cgmath;
extern crate png;

use std::default::Default;
use std::ptr;

use android_glue::AssetError;
use cgmath::{Matrix4, Point3, Vector3};

use egl;
use egl::{Context, Display, Surface};
use gl;

// TODO: Figure out how to put macros in a separate module and import when needed.

/// Logs the error to Android error logging and fails.
macro_rules! a_panic(
  ($msg: expr) => (
    panic!($msg);
  );
  ($fmt: expr, $($arg:tt)*) => (
    panic!($fmt, $($arg)*);
  );
);

/// Logs to Android info logging.
macro_rules! a_info(
  ($msg: expr) => (
    println!($msg);
  );
  ($fmt: expr, $($arg:tt)*) => (
    println!($fmt, $($arg)*);
  );
);

/// On error, logs the error and terminates.  On success, returns the result.
macro_rules! gl_try(
  ($e: expr) => (
    match $e {
      Ok(e) => e,
      Err(e) => a_panic!("{} failed: {:?}", stringify!($e), e),
    }
  )
);

// Saved state data.  Compatible with C.
struct SavedState {
  angle: f32,  // in degrees.
}

impl Default for SavedState {
  fn default() -> SavedState {
    SavedState { angle: 0.0 }
  }
}

// RAII managed EGL pointers.  Cleaned up automatically via Drop.
struct EglContext {
  display: Display,
  surface: Surface,
  context: Context,
  width: i32,
  height: i32,
}

impl Default for EglContext {
  fn default() -> EglContext {
    EglContext {
      display: egl::NO_DISPLAY,
      surface: egl::NO_SURFACE,
      context: egl::NO_CONTEXT,
      width: 0,
      height: 0,
    }
  }
}

impl EglContext {
  fn swap_buffers(&self) {
    gl_try!(egl::swap_buffers(self.display, self.surface));
  }
}

impl Drop for EglContext {
  fn drop(&mut self) {
    if self.display != egl::NO_DISPLAY {
      gl_try!(egl::make_current(self.display, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT));
      if self.context != egl::NO_CONTEXT {
        gl_try!(egl::destroy_context(self.display, self.context));
        self.context = egl::NO_CONTEXT;
      }
      if self.surface != egl::NO_SURFACE {
        gl_try!(egl::destroy_surface(self.display, self.surface));
        self.surface = egl::NO_SURFACE;
      }
      gl_try!(egl::terminate(self.display));
      self.display = egl::NO_DISPLAY;
    }
  }
}

// Shared state for our app.
// TODO: Find a way not to declare all fields public.
pub struct Engine {
//  pub jvm: &'static mut android_glue::ffi::JavaVM,
//  pub asset_manager: &'static mut android_glue::ffi::AAssetManager,
//  pub accelerometer_sensor: Option<&'static android_glue::ffi::ASensor>,
//  pub sensor_event_queue: &'static mut android_glue::ffi::ASensorEventQueue,
  pub animating: bool,
  pub egl_context: Option<Box<EglContext>>,
  pub state: SavedState,
  // GL bound variables.
  pub mvp_matrix: gl::UnifLoc,
  pub position: gl::AttribLoc,
  pub texture_unit: gl::UnifLoc,
  pub texture_coord: gl::AttribLoc,
  /// GL matrix
  pub view_projection_matrix: Matrix4<f32>,
  /// Texture atlas.
  pub texture: gl::Texture,
}

const NUMBERS_PER_VERTEX: i32 = 5;
const BYTES_PER_F32: i32 = 4;
const STRIDE: i32 = NUMBERS_PER_VERTEX * BYTES_PER_F32;

// X, Y, Z,
// S, T (note: T axis is going from top down)
const VERTICES: [f32; 180] = [
  // Front face.
  -0.5, -0.5, 0.5,
  0.5, 1.0,

  0.5, -0.5, 0.5,
  1.0, 1.0,

  0.5, 0.5, 0.5,
  1.0, 0.5,

  0.5, 0.5, 0.5,
  1.0, 0.5,

  -0.5, 0.5, 0.5,
  0.5, 0.5,

  -0.5, -0.5, 0.5,
  0.5, 1.0,

  // Right face.
  0.5, -0.5, 0.5,
  0.5, 1.0,

  0.5, -0.5, -0.5,
  1.0, 1.0,

  0.5, 0.5, -0.5,
  1.0, 0.5,

  0.5, 0.5, -0.5,
  1.0, 0.5,

  0.5, 0.5, 0.5,
  0.5, 0.5,

  0.5, -0.5, 0.5,
  0.5, 1.0,

  // Back face.
  0.5, -0.5, -0.5,
  0.5, 1.0,

  -0.5, -0.5, -0.5,
  1.0, 1.0,

  -0.5, 0.5, -0.5,
  1.0, 0.5,

  -0.5, 0.5, -0.5,
  1.0, 0.5,

  0.5, 0.5, -0.5,
  0.5, 0.5,

  0.5, -0.5, -0.5,
  0.5, 1.0,

  // Left face.
  -0.5, -0.5, -0.5,
  0.5, 1.0,

  -0.5, -0.5, 0.5,
  1.0, 1.0,

  -0.5, 0.5, 0.5,
  1.0, 0.5,

  -0.5, 0.5, 0.5,
  1.0, 0.5,

  -0.5, 0.5, -0.5,
  0.5, 0.5,

  -0.5, -0.5, -0.5,
  0.5, 1.0,

  // Top face.
  -0.5, 0.5, 0.5,
  0.0, 1.0,

  0.5, 0.5, 0.5,
  0.5, 1.0,

  0.5, 0.5, -0.5,
  0.5, 0.5,

  0.5, 0.5, -0.5,
  0.5, 0.5,

  -0.5, 0.5, -0.5,
  0.0, 0.5,

  -0.5, 0.5, 0.5,
  0.0, 1.0,

  // Bottom face.
  0.5, -0.5, 0.5,
  0.0, 0.5,

  -0.5, -0.5, 0.5,
  0.5, 0.5,

  -0.5, -0.5, -0.5,
  0.5, 0.0,

  -0.5, -0.5, -0.5,
  0.5, 0.0,

  0.5, -0.5, -0.5,
  0.0, 0.0,

  0.5, -0.5, 0.5,
  0.0, 0.5,
];

const VERTEX_SHADER: &'static str = "\
  uniform mat4 u_MVPMatrix;\n\
  attribute vec4 a_Position;\n\
  attribute vec2 a_TextureCoord;\n\
  varying vec2 v_TextureCoord;\n\
  void main() {\n\
    v_TextureCoord = a_TextureCoord;\n\
    gl_Position = u_MVPMatrix * a_Position;
  }\n";

const FRAGMENT_SHADER: &'static str = "\
  precision mediump float;\n\
  uniform sampler2D u_TextureUnit;\n\
  varying vec2 v_TextureCoord;\n\
  void main() {\n\
    gl_FragColor = texture2D(u_TextureUnit, v_TextureCoord);\n\
  }\n";

impl Engine {
  /// Initialize the engine.
  pub fn init(&mut self, egl_context: Box<EglContext>) {
    self.egl_context = Some(egl_context);

    // Set the background clear color to sky blue.
    gl::clear_color(0.5, 0.69, 1.0, 1.0);

    // Enable reverse face culling and depth test.
    gl_try!(gl::enable(gl::CULL_FACE));
    gl_try!(gl::enable(gl::DEPTH_TEST));

    let (mvp_matrix, position, texture_unit, texture_coord) = load_program(VERTEX_SHADER, FRAGMENT_SHADER);
    self.mvp_matrix = mvp_matrix;
    self.position = position;
    self.texture_unit = texture_unit;
    self.texture_coord = texture_coord;

    // Set up textures.
    self.texture = self.load_texture_atlas();
    gl_try!(gl::active_texture(gl::TEXTURE0));
    gl_try!(gl::bind_texture_2d(self.texture));
    gl_try!(gl::uniform_int(self.texture_unit, 0));

    // Set the vertex attributes for position and color.
    gl_try!(gl::vertex_attrib_pointer_f32(self.position, 3, STRIDE, &VERTICES));
    gl_try!(gl::enable_vertex_attrib_array(self.position));
    gl_try!(gl::vertex_attrib_pointer_f32(self.texture_coord, 2, STRIDE, &VERTICES[3..]));
    gl_try!(gl::enable_vertex_attrib_array(self.texture_coord));

    match self.egl_context {
      Some(ref ec) => {
        gl_try!(gl::viewport(0, 0, ec.width, ec.height));
        self.view_projection_matrix = view_projection_matrix(ec.width, ec.height);
      },
      None => a_panic!("self.egl_context should be present"),
    }
  }

  /// Load texture atlas from assets folder.
  fn load_texture_atlas(&mut self) -> gl::Texture {
    let vec = match android_glue::load_asset("atlas.png") {
      Ok(v) => v,
      Err(e) => {
        let mess = match e {
          AssetError::AssetMissing => "asset missing",
          AssetError::EmptyBuffer => "couldn't read asset",
        };
        a_panic!("Loading atlas.png failed: {}", mess)
      },
    };

    let image = png::load_png_from_memory(&vec)
      .unwrap_or_else(|s| a_panic!("load_png_from_memory() failed: {}", s));

    let pixels = match image.pixels {
      png::PixelsByColorType::RGBA8(v) => v,
      _ => {
        let color_type = match image.pixels {
            png::PixelsByColorType::K8(_) => "K8",
            png::PixelsByColorType::KA8(_) => "KA8",
            png::PixelsByColorType::RGB8(_) => "RGB8",
            png::PixelsByColorType::RGBA8(_) => panic!("Should not happen"),
        };
        a_panic!("Only RGBA8 image format supported, was: {}", color_type);
      }
    };

    let texture = gl_try!(gl::gen_texture());
    gl_try!(gl::bind_texture_2d(texture));
    gl_try!(gl::texture_2d_param(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR));
    gl_try!(gl::texture_2d_param(gl::TEXTURE_MAG_FILTER, gl::LINEAR));
    gl_try!(gl::texture_2d_param(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE));
    gl_try!(gl::texture_2d_param(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE));
    gl_try!(gl::texture_2d_image_rgba(image.width as i32, image.height as i32, &pixels));
    gl_try!(gl::generate_mipmap_2d());
    gl_try!(gl::bind_texture_2d(0));

    texture
  }

  /// Draw a frame.
  pub fn draw(&mut self) {
    if !self.animating {
      return;
    }

    match self.egl_context {
      None => return,  // No display.
      Some(ref egl_context) => {
        gl_try!(gl::clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT));

        // Create the model matrix based on the angle.
        let model_matrix = from_angle_y(self.state.angle);
        // Compute the composite mvp_matrix and send it to program.
        let mvp_matrix = self.view_projection_matrix * model_matrix;
        gl_try!(gl::uniform_matrix4_f32(self.mvp_matrix, &mvp_matrix));

        // Finally, draw the triangle.
        gl_try!(gl::draw_arrays_triangles(VERTICES.len() as i32 / NUMBERS_PER_VERTEX));

        egl_context.swap_buffers();
      }
    }
  }

  /// Update for time passed and draw a frame.
  pub fn update_draw(&mut self) {
    if self.animating {
      // Done processing events; draw next animation frame.
      // Do a complete rotation every 10 seconds, assuming 60 FPS.
      self.state.angle += 360.0 / 600.0;
      if self.state.angle > 360.0 {
        self.state.angle = 0.0;
      }

      // Drawing is throttled to the screen update rate, so there is no need to do timing here.
      self.draw();
    }
  }

  /// Terminate the engine.
  pub fn term(&mut self) {
    self.animating = false;
    self.egl_context = None;  // This closes the existing context via Drop.
    a_info!("Renderer terminated");
  }

  /// Called when window gains input focus.
  pub fn gained_focus(&mut self) {
    self.animating = true;
  }

  /// Called when window loses input focus.
  pub fn lost_focus(&mut self) {
    self.animating = false;
  }
}

fn view_projection_matrix(width: i32, height: i32) -> Matrix4<f32> {
  // Initialize a static view matrix.
  let eye = Point3::new(0.0, 1.0, 2.5);
  let center = Point3::new(0.0, 0.0, -5.0);
  let up = Vector3::new(0.0, 1.0, 0.0);
  let view_matrix = Matrix4::look_at(&eye, &center, &up);

  // Initialize perspective projection matrix as frustum matrix.
  let ratio = width as f32 / height as f32;
  let left = -ratio;
  let right = ratio;
  let bottom = -1.0;
  let top = 1.0;
  let near = 1.0;
  let far = 10.0;
  let projection_matrix = cgmath::frustum(left, right, bottom, top, near, far);

  projection_matrix * view_matrix
}

/// Create a matrix from a rotation around the `y` axis (yaw).
fn from_angle_y(degrees: f32) -> Matrix4<f32> {
    // http://en.wikipedia.org/wiki/Rotation_matrix#Basic_rotations
    use std::num::Float;
    let (s, c) = degrees.to_radians().sin_cos();
    Matrix4::new(   c, 0.0,  -s, 0.0,
                  0.0, 1.0, 0.0, 0.0,
                    s, 0.0,   c, 0.0,
                  0.0, 0.0, 0.0, 1.0)
}

pub fn create_egl_context(window: *mut android_glue::ffi::ANativeWindow) -> EglContext {
  let display = egl::get_display(egl::DEFAULT_DISPLAY);

  gl_try!(egl::initialize(display));

  // Here specify the attributes of the desired configuration.  Below, we select an EGLConfig with
  // at least 8 bits per color component compatible with OpenGL ES 2.0.  A very simplified
  // selection process, where we pick the first EGLConfig that matches our criteria.
  let attribs_config = [
    egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
    egl::BLUE_SIZE, 8,
    egl::GREEN_SIZE, 8,
    egl::RED_SIZE, 8,
    egl::NONE
  ];
  let mut configs = vec!(ptr::null());
  gl_try!(egl::choose_config(display, &attribs_config, &mut configs));
  if configs.len() == 0 {
    a_panic!("choose_config() did not find any configurations");
  }
  let config = configs[0];

  // EGL_NATIVE_VISUAL_ID is an attribute of the EGLConfig that is guaranteed to be accepted by
  // ANativeWindow_setBuffersGeometry().  As soon as we picked a EGLConfig, we can safely
  // reconfigure the NativeWindow buffers to match, using EGL_NATIVE_VISUAL_ID.
  let format = gl_try!(egl::get_config_attrib(display, config, egl::NATIVE_VISUAL_ID));

  unsafe {
    android_glue::ffi::ANativeWindow_setBuffersGeometry(window, 0, 0, format);
  }

  let surface = gl_try!(egl::create_window_surface(display, config, window));

  let attribs_context = [
    egl::CONTEXT_CLIENT_VERSION, 2,
    egl::NONE
  ];
  let context = gl_try!(egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, &attribs_context));

  gl_try!(egl::make_current(display, surface, surface, context));

  let w = gl_try!(egl::query_surface(display, surface, egl::WIDTH));
  let h = gl_try!(egl::query_surface(display, surface, egl::HEIGHT));

  EglContext {
    display: display,
    surface: surface,
    context: context,
    width: w,
    height: h,
  }
}

fn compile_shader(shader_string: &str, shader_type: gl::Enum) -> gl::Shader {
  let shader = gl_try!(gl::create_shader(shader_type));
  gl_try!(gl::shader_source(shader, shader_string));
  gl_try!(gl::compile_shader(shader));
  let status = gl_try!(gl::get_compile_status(shader));
  if !status {
    let info_log = gl_try!(gl::get_shader_info_log(shader));
    gl_try!(gl::delete_shader(shader));
    a_panic!("Compiling shader {} failed: {}", shader_type, info_log);
  }
  shader
}

fn load_program(vertex_shader_string: &str, fragment_shader_string: &str) ->
  (gl::UnifLoc, gl::AttribLoc, gl::UnifLoc, gl::AttribLoc) {
  let vertex_shader = compile_shader(vertex_shader_string, gl::VERTEX_SHADER);
  let fragment_shader = compile_shader(fragment_shader_string, gl::FRAGMENT_SHADER);
  let program = gl_try!(gl::create_program());
  gl_try!(gl::attach_shader(program, vertex_shader));
  gl_try!(gl::attach_shader(program, fragment_shader));
  gl_try!(gl::bind_attrib_location(program, 0, "a_Position"));
  gl_try!(gl::bind_attrib_location(program, 1, "a_TextureCoord"));
  gl_try!(gl::link_program(program));
  let status = gl_try!(gl::get_link_status(program));
  if !status {
    let info_log = gl_try!(gl::get_program_info_log(program));
    gl_try!(gl::delete_program(program));
    a_panic!("Linking program failed: {}", info_log);
  }
  let mvp_matrix = gl_try!(gl::get_uniform_location(program, "u_MVPMatrix"));
  let position = gl_try!(gl::get_attrib_location(program, "a_Position"));
  let texture_unit = gl_try!(gl::get_uniform_location(program, "u_TextureUnit"));
  let texture_coord = gl_try!(gl::get_attrib_location(program, "a_TextureCoord"));
  gl_try!(gl::use_program(program));
  (mvp_matrix, position, texture_unit, texture_coord)
}
