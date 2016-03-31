extern crate cgmath;
extern crate png;

use std::collections::HashMap;
use std::default::Default;
use std::f32::consts::PI;
use time;

use cgmath::Matrix4;
use fps::{Fps, Stats};

#[cfg(target_os = "android")]
use egl_context::EglContext;
use fov::{FAR_PLANE, Fov};
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
  fov: Fov,
  /// GL projection matrix
  projection_matrix: Matrix4<f32>,
  /// Texture atlas.
  texture: Texture,
  world: World,
  buffers: HashMap<Chunk, Buffers>,
  fps: Fps,
}

#[cfg(target_os = "linux")]
impl Drop for Engine {
  fn drop(&mut self) {
    self.lost_focus();
    log!("*** Renderer terminated");
  }
}

impl Engine {
  #[cfg(target_os = "android")]
  pub fn new() -> Engine {
    use cgmath::SquareMatrix;
    Engine {
      engine_impl: Default::default(),
      animating: false,
      fov: Fov {
        vertex: Point2::new(0.0, 0.0),
        center_angle: 0f32.to_rad(),
        view_angle: 70f32.to_rad(),
      },
      projection_matrix: Matrix4::identity(),
      texture: Default::default(),
      world: World::new(&Point2::new(0.0, 0.0), FAR_PLANE),
      buffers: HashMap::new(),
      fps: Fps::stopped(),
    }
  }

  #[cfg(target_os = "linux")]
  pub fn new(window: XWindow, program: Program) -> Engine {
    use cgmath::SquareMatrix;
    Engine {
      engine_impl: EngineImpl {
        window: window,
        program: program,
      },
      animating: false,
      fov: Fov {
        vertex: Point2::new(0.0, 0.0),
        center_angle: 0f32.to_rad(),
        view_angle: 70f32.to_rad(),
      },
      projection_matrix: Matrix4::identity(),
      texture: Default::default(),
      world: World::new(&Point2::new(0.0, 0.0), FAR_PLANE),
      buffers: HashMap::new(),
      fps: Fps::stopped(),
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

    self.load_meshes();
  }

  #[cfg(target_os = "android")]
  fn load_meshes(&mut self) {
    let start_s = time::precise_time_s();
    log!("*** Loading meshes...");

    if let Some(ref mut p) = self.engine_impl.program {
      for (c, bs) in self.world.chunk_blocks() {
        let vertices = mesh::create_mesh_vertices(bs, &self.world);
        let buffers = p.upload_vertices(&vertices);
        let c = (*c).clone();
        self.buffers.insert(c, buffers);
      }
    }

    let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
    log!("*** Loaded meshes: {:.3}ms, {} chunks", spent_ms, self.buffers.len());
  }

  #[cfg(target_os = "linux")]
  fn load_meshes(&mut self) {
    let start_s = time::precise_time_s();
    log!("*** Loading meshes...");

    let p = &self.engine_impl.program;
    for (c, bs) in self.world.chunk_blocks() {
      let vertices = mesh::create_mesh_vertices(bs, &self.world);
      let buffers = p.upload_vertices(&vertices);
      let c = (*c).clone();
      self.buffers.insert(c, buffers);
    }

    let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
    log!("*** Loaded meshes: {:.3}ms, {} chunks", spent_ms, self.buffers.len());
  }

  pub fn set_viewport(&mut self, w: i32, h: i32) {
    gl::viewport(0, 0, w, h);
    self.projection_matrix = self.fov.projection_matrix(w, h);
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
              let mvp_matrix = self.projection_matrix * self.fov.view_matrix(&e);
              p.set_mvp_matrix(mvp_matrix);

              // Finally, draw the cube mesh for all visible chunks.
              for (ch, bs) in self.buffers.iter() {
                if self.fov.chunk_visible(ch) {
                  p.bind_buffers(bs);
                  gl::draw_elements_triangles_u16(bs.index_count);
                  p.unbind_buffers();
                }
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
      let mvp_matrix = self.projection_matrix * self.fov.view_matrix(&e);
      p.set_mvp_matrix(mvp_matrix);

      // Finally, draw the cube meshes for all visible chunks.
      for (ch, bs) in self.buffers.iter() {
        if self.fov.chunk_visible(ch) {
          p.bind_buffers(bs);
          gl::draw_elements_triangles_u16(bs.index_count);
          p.unbind_buffers();
        }
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
      self.fov.inc_center_angle(2.0 * PI / 600.0);

      // Drawing is throttled to the screen update rate, so there is no need to do timing here.
      self.draw();
      if let Some(fps) = self.fps.tick() {
        print_fps(fps);
      }
    }
  }

  /// Terminate the engine.
  #[cfg(target_os = "android")]
  pub fn term(&mut self) {
    self.lost_focus();
    // Drop the program and the EGL context.
    self.engine_impl = Default::default();
    log!("*** Renderer terminated");
  }

  /// Called when window gains input focus.
  pub fn gained_focus(&mut self) {
    if !self.animating {
      self.animating = true;
      self.fps.start();
    }
  }

  /// Called when window loses input focus.
  pub fn lost_focus(&mut self) {
    if self.animating {
      self.animating = false;
      if let Some(fps) = self.fps.stop() {
        print_fps(fps);
      }
    }
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

fn print_fps(fps: Stats) {
  println!("FPS: min {:.1}, avg {:.1}, max {:.1}", fps.min, fps.avg, fps.max);
}

trait ToRad {
  /// Convert a value into radians.
  fn to_rad(&self) -> Self;
}

impl ToRad for f32 {
  fn to_rad(&self) -> Self {
    self * PI / 180.0
  }
}
