extern crate cgmath;
extern crate png;

#[cfg(target_os = "android")]
use std::default::Default;

use cgmath::{Matrix4, Point3, Vector3};

#[cfg(target_os = "android")]
use egl_context::EglContext;
use gl;
use gl::Texture;
use mesh;
use program::Program;
#[cfg(target_os = "linux")]
use x11::{PollEventsIterator, XWindow};

#[cfg(target_os = "android")]
pub struct EngineImpl {
  pub egl_context: Option<Box<EglContext>>,
  pub program: Option<Program>,
}

#[cfg(target_os = "android")]
impl Default for EngineImpl {
  fn default() -> EngineImpl {
    EngineImpl {
      egl_context: None,
      program: None,
    }
  }
}

#[cfg(target_os = "linux")]
pub struct EngineImpl {
  pub window: XWindow,
  pub program: Program,
}

// TODO: Find a way not to declare all fields public.
pub struct Engine {
  pub engine_impl: EngineImpl,
  pub animating: bool,
  pub angle: f32,  // in degrees.
  /// GL matrix
  pub view_projection_matrix: Matrix4<f32>,
  /// Texture atlas.
  pub texture: Texture,
}

impl Engine {
  /// Initialize the engine.
  #[cfg(target_os = "android")]
  pub fn init(&mut self, egl_context: Box<EglContext>, texture_atlas_bytes: &[u8]) {
    self.engine_impl.egl_context = Some(egl_context);

    self.engine_impl.program = match Program::new() {
      Ok(p) => Some(p),
      Err(e) => panic!("Program failed: {:?}", e),
    };

    self.common_init(texture_atlas_bytes);

    if let Some(ref p) = self.engine_impl.program {
      gl::uniform_int(p.texture_unit, 0);
    }

    // Silly gymnastics to satisfy the borrow checker.
    let wh = if let Some(ref ec) = self.engine_impl.egl_context {
      Some((ec.width, ec.height))
    } else {
      None
    };
    if let Some((w, h)) = wh {
      self.set_viewport(w, h);
    }
  }

  /// Initialize the engine.
  #[cfg(target_os = "linux")]
  pub fn init(&mut self, texture_atlas_bytes: &[u8]) {
    self.common_init(texture_atlas_bytes);
    gl::uniform_int(self.engine_impl.program.texture_unit, 0);
  }

  fn common_init(&mut self, texture_atlas_bytes: &[u8]) {
    // Set the background clear color to sky blue.
    gl::clear_color(0.5, 0.69, 1.0, 1.0);

    // Enable reverse face culling and depth test.
    gl::enable(gl::CULL_FACE);
    gl::enable(gl::DEPTH_TEST);

    // Set up textures.
    self.texture = Engine::load_texture_atlas(texture_atlas_bytes);
    gl::active_texture(gl::TEXTURE0);
    gl::bind_texture_2d(self.texture);
  }

  pub fn set_viewport(&mut self, w: i32, h: i32) {
    gl::viewport(0, 0, w, h);
    self.view_projection_matrix = view_projection_matrix(w, h);
  }

  /// Load texture atlas from assets folder.
  fn load_texture_atlas(texture_atlas_bytes: &[u8]) -> Texture {
    let image = png::load_png_from_memory(texture_atlas_bytes)
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
  #[cfg(target_os = "android")]
  pub fn draw(&mut self) {
    if !self.animating {
      return;
    }

    match self.engine_impl.egl_context {
      None => return,  // No display.
      Some(ref egl_context) => {
        gl::clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);

        match self.engine_impl.program {
          Some(ref p) => self.set_mvp_matrix(p),
          None => panic!("Missing program, should never happen"),
        }

        // Finally, draw the cube mesh.
        gl::draw_arrays_triangles(mesh::triangle_count());

        egl_context.swap_buffers();
      }
    }
  }

  /// Draw a frame.
  #[cfg(target_os = "linux")]
  pub fn draw(&mut self) {
    if !self.animating {
      return;
    }

    gl::clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);

    self.set_mvp_matrix(&self.engine_impl.program);

    // Finally, draw the cube mesh.
    gl::draw_arrays_triangles(mesh::triangle_count());

    self.engine_impl.window.swap_buffers();
    self.engine_impl.window.flush();
  }

  fn set_mvp_matrix(&self, program: &Program) {
    // Create the model matrix based on the angle.
    let model_matrix = from_angle_y(self.angle);
    // Compute the composite mvp_matrix and send it to program.
    let mvp_matrix = self.view_projection_matrix * model_matrix;
    gl::uniform_matrix4_f32(program.mvp_matrix, &mvp_matrix);
  }

  /// Update for time passed and draw a frame.
  pub fn update_draw(&mut self) {
    if self.animating {
      // Done processing events; draw next animation frame.
      // Do a complete rotation every 10 seconds, assuming 60 FPS.
      self.angle += 360.0 / 600.0;
      if self.angle > 360.0 {
        self.angle = 0.0;
      }

      // Drawing is throttled to the screen update rate, so there is no need to do timing here.
      self.draw();
    }
  }

  /// Terminate the engine.
  #[cfg(target_os = "android")]
  pub fn term(&mut self) {
    self.animating = false;
    // Drop the program and the EGL context.
    self.engine_impl = Default::default();
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

  #[cfg(target_os = "linux")]
  pub fn is_closed(&self) -> bool {
    self.engine_impl.window.is_closed()
  }

  #[cfg(target_os = "linux")]
  pub fn poll_events(&self) -> PollEventsIterator {
    self.engine_impl.window.poll_events()
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
