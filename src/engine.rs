extern crate cgmath;
extern crate png;

use std::default::Default;
use std::f64;
use time;

use cgmath::{Matrix4, Point, Point3, Vector3};
use noise;
use noise::{Brownian3, Seed};

#[cfg(target_os = "android")]
use egl_context::EglContext;
use gl;
use gl::Texture;
use mesh;
use program::{Program, VertexArray};
use world::{Block, World};
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
      world: generate_chunk_of_perlin(),
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
      world: generate_chunk_of_perlin(),
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

  fn create_mesh_vertices(world: &World) -> (Vec<f32>, Vec<f32>) {
    // If the world nas N cubes in it, the mesh may have up to 12 * N triangles
    // and up to 9 * 12 * N vertices.  Set capacity to half of that.
    let mut vertex_coords: Vec<f32> = Vec::with_capacity(54 * world.len());
    // Up to 6 * 12 * N texture coordinated, also halve it.
    let mut texture_coords: Vec<f32> = Vec::with_capacity(36 * world.len());

    for block in world.iter() {
      // Eliminate definitely invisible faces, i.e. those between two neighboring cubes.
      for face in mesh::CUBE_FACES.iter() {
        if !world.contains(&block.add_v(&face.direction)) {
          vertex_coords.push_all(&translate(&face.vertices, block));
          texture_coords.push_all(&face.texture_coords);
        }
      }
    }

    (vertex_coords, texture_coords)
  }

  #[cfg(target_os = "android")]
  fn load_mesh(&mut self) {
    if let Some(ref mut p) = self.engine_impl.program {
      let (vertex_coords, texture_coords) = Engine::create_mesh_vertices(&self.world);
      assert!(vertex_coords.len() / 3 == texture_coords.len() / 2);
      let vcs = VertexArray {
        data: &vertex_coords[0..],
        components: 3,
        stride: 12,
      };
      let tcs = VertexArray {
        data: &texture_coords[0..],
        components: 2,
        stride: 8,
      };
      p.set_vertices(&vcs, &tcs);

      // Debug:
      log!("*** Triangle count: {}, vertex count: {}, xyz count: {}, bytes: {}, st count: {}, bytes: {}",
        p.triangle_count(), p.vertex_count(), vertex_coords.len(), vertex_coords.len() * 4,
        texture_coords.len(), texture_coords.len() * 4);
    }
  }

  #[cfg(target_os = "linux")]
  fn load_mesh(&mut self) {
    let (vertex_coords, texture_coords) = Engine::create_mesh_vertices(&self.world);
    assert!(vertex_coords.len() / 3 == texture_coords.len() / 2);
    let vcs = VertexArray {
      data: &vertex_coords[0..],
      components: 3,
      stride: 12,
    };
    let tcs = VertexArray {
      data: &texture_coords[0..],
      components: 2,
      stride: 8,
    };
    let p = &mut self.engine_impl.program;
    p.set_vertices(&vcs, &tcs);

    // Debug:
    log!("*** Triangle count: {}, vertex count: {}, xyz count: {}, bytes: {}, st count: {}, bytes: {}",
      p.triangle_count(), p.vertex_count(), vertex_coords.len(), vertex_coords.len() * 4,
      texture_coords.len(), texture_coords.len() * 4);
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
        gl::draw_arrays_triangles(self.vertex_count as i32);

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
    self.set_mvp_matrix(&self.engine_impl.program);

    // Finally, draw the cube mesh.
    gl::draw_arrays_triangles(p.vertex_count());

    self.engine_impl.window.swap_buffers();
    self.engine_impl.window.flush();
  }

  fn set_mvp_matrix(&self, program: &Program) {
    // Compute the composite mvp_matrix and send it to program.  Model matrix
    // is always identity so instead of MVP = P * V * M just do MVP = P * V.
    let mvp_matrix = self.projection_matrix * view_matrix(self.angle);
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

fn generate_chunk_of_perlin() -> World {
  let start_s = time::precise_time_s();

  let seed = Seed::new(1);
  let noise = Brownian3::new(noise::perlin3, 4).wavelength(16.0);

  let y_min = -3;
  let y_max = 2;
  let y_range = y_max - y_min;

  let mut min = f64::MAX;
  let mut max = f64::MIN;
  let mut world: World = Default::default();

  for y in y_min..(y_max + 1) {
    // Normalize into [0, 1].
    let normalized_y = (y as f64 - y_min as f64) / y_range as f64;
    for x in -3..3 {
      for z in -3..4 {
        let p = [x as f64, y as f64, z as f64];
        let val = noise.apply(&seed, &p);

        if val < min {
          min = val;
        }
        if val > max {
          max = val;
        }

        // Probablility to have a block added linearly increases from 0.0 at y_max to 1.0 at y_min.
        if 0.5 * (val + 1.0) >= normalized_y {
          world.add(Block::new(x, y, z));
        }
      }
    }
  }

  let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
  log!("*** Generating a chunk of perlin: {:.3}ms, {} blocks", spent_ms, world.len());
  log!("***   min = {}, max = {}", min, max);
  log!("***   world = {:?}", world);

  world
}

/// Accepts vertex coordinates as a flat list: x, y, z, x, y, z, ...
/// Translates them along the vector corresponding to the block.
fn translate(coords: &[f32; 18], block: &Block) -> [f32; 18] {
  let x = block.x as f32;
  let y = block.y as f32;
  let z = block.z as f32;

  [
    coords[0] + x, coords[1] + y, coords[2] + z,
    coords[3] + x, coords[4] + y, coords[5] + z,
    coords[6] + x, coords[7] + y, coords[8] + z,
    coords[9] + x, coords[10] + y, coords[11] + z,
    coords[12] + x, coords[13] + y, coords[14] + z,
    coords[15] + x, coords[16] + y, coords[17] + z,
  ]
}

/// A view matrix, eye is on a 10.0 radius circle starting at (0.0, 2.1, 10.0),
/// rotating around (0, 1, 0) counter-clockwise and looking at (0, 2.1, 0).
fn view_matrix(angle: f32) -> Matrix4<f32> {
  let r = 13.0;
  let y = 2.1;  // 0.5 for half block under feet + 1.6 up to eye height.
  let (s, c) = angle.to_radians().sin_cos();
  let eye = Point3::new(r * s, y, r * c);
  let center = Point3::new(0.0, y, 0.0);
  let up = Vector3::new(0.0, 1.0, 0.0);
  Matrix4::look_at(&eye, &center, &up)
}

/// Perspective projection matrix as frustum matrix.
fn projection_matrix(width: i32, height: i32) -> Matrix4<f32> {
  let ratio = width as f32 / height as f32;
  let left = -ratio;
  let right = ratio;
  let bottom = -1.0;
  let top = 1.0;
  let near = 1.0;
  let far = 50.0;
  cgmath::frustum(left, right, bottom, top, near, far)
}
