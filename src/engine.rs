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
use mesh;
use program::Program;

// Saved state data.  Compatible with C.
pub struct SavedState {
  angle: f32,  // in degrees.
}

impl Default for SavedState {
  fn default() -> SavedState {
    SavedState { angle: 0.0 }
  }
}

// RAII managed EGL pointers.  Cleaned up automatically via Drop.
pub struct EglContext {
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
  pub fn new(window: *mut android_glue::ffi::ANativeWindow) -> EglContext {
    let display = egl::get_display(egl::DEFAULT_DISPLAY);
    if let Err(e) = egl::initialize(display) {
      panic!("Failed in egl::initialize(): {:?}", e);
    }

    // Here specify the attributes of the desired configuration.  Below, we select an EGLConfig with
    // at least 8 bits per color component compatible with OpenGL ES 2.0.  A very simplified
    // selection process, where we pick the first EGLConfig that matches our criteria.
    let config = {
      let attribs_config = [
        egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
        egl::BLUE_SIZE, 8,
        egl::GREEN_SIZE, 8,
        egl::RED_SIZE, 8,
        egl::NONE
      ];
      let mut configs = vec!(ptr::null());
      if let Err(e) = egl::choose_config(display, &attribs_config, &mut configs) {
        panic!("Failed in egl::choose_config(): {:?}", e);
      }
      if configs.len() == 0 {
        panic!("egl::choose_config() did not find any configurations");
      }
      configs[0]
    };

    // EGL_NATIVE_VISUAL_ID is an attribute of the EGLConfig that is guaranteed to be accepted by
    // ANativeWindow_setBuffersGeometry().  As soon as we picked a EGLConfig, we can safely
    // reconfigure the NativeWindow buffers to match, using EGL_NATIVE_VISUAL_ID.
    let format = match egl::get_config_attrib(display, config, egl::NATIVE_VISUAL_ID) {
      Ok(f) => f,
      Err(e) => panic!("egl::get_config_attrib(NATIVE_VISUAL_ID) failed: {:?}", e),
    };

    unsafe {
      android_glue::ffi::ANativeWindow_setBuffersGeometry(window, 0, 0, format);
    }

    let surface = match egl::create_window_surface(display, config, window) {
      Ok(s) => s,
      Err(e) => panic!("egl::create_window_surface() failed: {:?}", e),
    };

    let context = {
      let attribs_context = [
        egl::CONTEXT_CLIENT_VERSION, 2,
        egl::NONE
      ];
      match egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, &attribs_context) {
        Ok(c) => c,
        Err(e) => panic!("egl::create_context_with_attribs() failed: {:?}", e),
      }
    };

    if let Err(e) = egl::make_current(display, surface, surface, context) {
      panic!("Failed in egl::make_current(): {:?}", e);
    }

    let w = match egl::query_surface(display, surface, egl::WIDTH) {
      Ok(w) => w,
      Err(e) => panic!("egl::query_surface(WIDTH) failed: {:?}", e),
    };
    let h = match egl::query_surface(display, surface, egl::HEIGHT) {
      Ok(w) => w,
      Err(e) => panic!("egl::query_surface(HEIGHT) failed: {:?}", e),
    };

    EglContext {
      display: display,
      surface: surface,
      context: context,
      width: w,
      height: h,
    }
  }

  fn swap_buffers(&self) {
    let _ = egl::swap_buffers(self.display, self.surface);
  }
}

impl Drop for EglContext {
  fn drop(&mut self) {
    if self.display != egl::NO_DISPLAY {
      let _ = egl::make_current(self.display, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT);
      if self.context != egl::NO_CONTEXT {
        let _ = egl::destroy_context(self.display, self.context);
        self.context = egl::NO_CONTEXT;
      }
      if self.surface != egl::NO_SURFACE {
        let _ = egl::destroy_surface(self.display, self.surface);
        self.surface = egl::NO_SURFACE;
      }
      let _ = egl::terminate(self.display);
      self.display = egl::NO_DISPLAY;
    }
  }
}

// Shared state for our app.
// TODO: Find a way not to declare all fields public.
pub struct Engine {
  pub animating: bool,
  pub egl_context: Option<Box<EglContext>>,
  pub state: SavedState,
  pub program: Option<Program>,
  /// GL matrix
  pub view_projection_matrix: Matrix4<f32>,
  /// Texture atlas.
  pub texture: gl::Texture,
}

impl Engine {
  /// Initialize the engine.
  pub fn init(&mut self, egl_context: Box<EglContext>) {
    self.egl_context = Some(egl_context);

    // Set the background clear color to sky blue.
    gl::clear_color(0.5, 0.69, 1.0, 1.0);

    // Enable reverse face culling and depth test.
    gl::enable(gl::CULL_FACE);
    gl::enable(gl::DEPTH_TEST);

    match Program::new() {
      Ok(p) => self.program = Some(p),
      Err(e) => panic!("Program failed: {:?}", e),
    }

    // Set up textures.
    self.texture = self.load_texture_atlas();
    gl::active_texture(gl::TEXTURE0);
    gl::bind_texture_2d(self.texture);
    match self.program {
      Some(ref p) => gl::uniform_int(p.texture_unit, 0),
      None => panic!("Missing program, should never happen"),
    }
    ;

    match self.egl_context {
      Some(ref ec) => {
        gl::viewport(0, 0, ec.width, ec.height);
        self.view_projection_matrix = view_projection_matrix(ec.width, ec.height);
      },
      None => panic!("self.egl_context should be present"),
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
        panic!("Loading atlas.png failed: {}", mess)
      },
    };

    let image = png::load_png_from_memory(&vec)
      .unwrap_or_else(|s| panic!("load_png_from_memory() failed: {}", s));

    let pixels = match image.pixels {
      png::PixelsByColorType::RGBA8(v) => v,
      _ => {
        let color_type = match image.pixels {
            png::PixelsByColorType::K8(_) => "K8",
            png::PixelsByColorType::KA8(_) => "KA8",
            png::PixelsByColorType::RGB8(_) => "RGB8",
            png::PixelsByColorType::RGBA8(_) => panic!("Should not happen"),
        };
        panic!("Only RGBA8 image format supported, was: {}", color_type);
      }
    };

    let texture = gl::gen_texture();
    gl::bind_texture_2d(texture);
    gl::texture_2d_param(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    gl::texture_2d_param(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    gl::texture_2d_param(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
    gl::texture_2d_param(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);
    gl::texture_2d_image_rgba(image.width as i32, image.height as i32, &pixels);
    gl::generate_mipmap_2d();
    gl::bind_texture_2d(0);

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
        gl::clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);

        match self.program {
          Some(ref p) => {
            // Create the model matrix based on the angle.
            let model_matrix = from_angle_y(self.state.angle);
            // Compute the composite mvp_matrix and send it to program.
            let mvp_matrix = self.view_projection_matrix * model_matrix;
            gl::uniform_matrix4_f32(p.mvp_matrix, &mvp_matrix);
          },
          None => panic!("Missing program, should never happen"),
        }

        // Finally, draw the triangle.
        gl::draw_arrays_triangles(mesh::triangle_count());

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
    self.program = None;  // Drop the program.
    self.egl_context = None;  // Drop the context.
    println!("Renderer terminated");
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
