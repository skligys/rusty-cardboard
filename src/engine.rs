extern crate cgmath;
extern crate png;

use std::default::Default;
use time;

use cgmath::{Matrix4, Point3, Vector3};

#[cfg(target_os = "android")]
use egl_context::EglContext;
use gl;
use gl::Texture;
use mesh;
use program::{Buffers, Program};
use world::{Chunk, Point2, World};
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

pub struct Engine {
  engine_impl: EngineImpl,
  animating: bool,
  angle: f32,  // in degrees.
  /// GL projection matrix
  projection_matrix: Matrix4<f32>,
  /// Texture atlas.
  texture: Texture,
  world: World,
  buffers: Option<Buffers>,
}

impl Engine {
  #[cfg(target_os = "android")]
  pub fn new() -> Engine {
    Engine {
      engine_impl: Default::default(),
      animating: false,
      angle: 0.0,
      projection_matrix: Matrix4::identity(),
      texture: Default::default(),
      world: World::new(&Point2::new(0.0, 0.0), FAR_PLANE),
      buffers: None,
    }
  }

  #[cfg(target_os = "linux")]
  pub fn new(window: XWindow, program: Program) -> Engine {
    Engine {
      engine_impl: EngineImpl {
        window: window,
        program: program,
      },
      animating: false,
      angle: 0.0,
      projection_matrix: Matrix4::identity(),
      texture: Default::default(),
      world: World::new(&Point2::new(0.0, 0.0), FAR_PLANE),
      buffers: None,
    }
  }

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
      p.set_texture_unit(0);
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
    self.engine_impl.program.set_texture_unit(0);
  }

  fn common_init(&mut self, texture_atlas_bytes: &[u8]) {
    // Set the background clear color to sky blue.
    gl::clear_color(0.5, 0.69, 1.0, 1.0);

    // Enable reverse face culling.
    gl::enable(gl::CULL_FACE);
    // Enable depth test.
    gl::enable(gl::DEPTH_TEST);
    gl::depth_func(gl::LEQUAL);

    // Set up textures.
    self.texture = Engine::load_texture_atlas(texture_atlas_bytes);
    gl::active_texture(gl::TEXTURE0);
    gl::bind_texture_2d(self.texture);

    self.load_mesh();
  }

  #[cfg(target_os = "android")]
  fn load_mesh(&mut self) {
    let start_s = time::precise_time_s();
    log!("*** Loading mesh...");

    if let Some(ref mut p) = self.engine_impl.program {
      let vertices = mesh::create_mesh_vertices(
        self.world.chunk_blocks(&Chunk::new(0, 0, 0)).unwrap(),
        &self.world);
      let buffers = p.upload_vertices(&vertices);
      self.buffers = Some(buffers);
    }

    let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
    log!("*** Loaded mesh: {:.3}ms, 1 chunk", spent_ms);
  }

  #[cfg(target_os = "linux")]
  fn load_mesh(&mut self) {
    let start_s = time::precise_time_s();
    log!("*** Loading mesh...");

    let vertices = mesh::create_mesh_vertices(
        self.world.chunk_blocks(&Chunk::new(0, 0, 0)).unwrap(),
        &self.world);
    let p = &self.engine_impl.program;
    let buffers = p.upload_vertices(&vertices);
    self.buffers = Some(buffers);

    let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
    log!("*** Loaded mesh: {:.3}ms, 1 chunk", spent_ms);
  }

  pub fn set_viewport(&mut self, w: i32, h: i32) {
    gl::viewport(0, 0, w, h);
    self.projection_matrix = projection_matrix(w, h);
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
    gl::texture_2d_param(gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR);
    gl::texture_2d_param(gl::TEXTURE_MAG_FILTER, gl::NEAREST);
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
          Some(ref p) => {
            if let Some(e) = self.world.eye() {
              // Compute the composite mvp_matrix and send it to program.  Model matrix
              // is always identity so instead of MVP = P * V * M just do MVP = P * V.
              let mvp_matrix = self.projection_matrix * view_matrix(&e, self.angle);
              p.set_mvp_matrix(mvp_matrix);

              // Finally, draw the cube mesh.
              if let Some(ref bs) = self.buffers {
                gl::bind_array_buffer(bs.vertex_buffer);
                gl::bind_index_buffer(bs.index_buffer);
                gl::draw_elements_triangles_u16(bs.index_count);
              }
            }
          },
          None => panic!("Missing program, should never happen"),
        }

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

    let p = &self.engine_impl.program;

    if let Some(e) = self.world.eye() {
      // Compute the composite mvp_matrix and send it to program.  Model matrix
      // is always identity so instead of MVP = P * V * M just do MVP = P * V.
      let mvp_matrix = self.projection_matrix * view_matrix(&e, self.angle);
      p.set_mvp_matrix(mvp_matrix);

      // Finally, draw the cube mesh.
      if let Some(ref bs) = self.buffers {
        gl::bind_array_buffer(bs.vertex_buffer);
        gl::bind_index_buffer(bs.index_buffer);
        gl::draw_elements_triangles_u16(bs.index_count);
      }
    }

    self.engine_impl.window.swap_buffers();
    self.engine_impl.window.flush();
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
    log!("*** Renderer terminated");
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

/// A view matrix, eye is at (p.x, p.y + 2.12, p.z), rotating in horizontal plane counter-clockwise
/// and looking at (p.x + sin α, p.y + 2.12, p.z + cos α).
fn view_matrix(p: &Point3<i32>, angle: f32) -> Matrix4<f32> {
  let y = p.y as f32 + 2.12;  // 0.5 for half block under feet + 1.62 up to eye height.
  let (s, c) = angle.to_radians().sin_cos();

  let eye = Point3::new(p.x as f32, y, p.z as f32);
  let center = Point3::new(p.x as f32 + s, y, p.x as f32 + c);
  let up = Vector3::new(0.0, 1.0, 0.0);
  Matrix4::look_at(&eye, &center, &up)
}

const NEAR_PLANE: f32 = 0.1;
const FAR_PLANE: f32 = 60.0;
const FIELD_OF_VIEW_DEGREES: f32 = 70.0;

/// Perspective projection matrix as frustum matrix.
fn projection_matrix(width: i32, height: i32) -> Matrix4<f32> {
  let inverse_aspect = height as f32 / width as f32;
  let field_of_view = FIELD_OF_VIEW_DEGREES.to_radians();

  let right = NEAR_PLANE * (field_of_view / 2.0).tan();
  let left = -right;
  let top = right * inverse_aspect;
  let bottom = -top;
  let near = NEAR_PLANE;
  let far = FAR_PLANE;
  cgmath::frustum(left, right, bottom, top, near, far)
}
