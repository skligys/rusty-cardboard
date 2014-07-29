extern crate cgmath;

use libc::{c_float, c_void, malloc, size_t};
use std::mem;
use std::ptr;
use std::default::Default;

use self::cgmath::matrix::Matrix4;
use self::cgmath::point::Point3;
use self::cgmath::projection;
use self::cgmath::vector::Vector3;

use egl;
use gl;
use input;
use log;
use native_window;
use sensor;

// TODO: Figure out how to put macros in a separate module and import when needed.

/// Logs the error to Android error logging and fails.
macro_rules! a_fail(
  ($msg: expr) => ({
    log::e($msg);
    fail!();
  });
  ($fmt: expr, $($arg:tt)*) => ({
    log::e_f(format!($fmt, $($arg)*));
    fail!();
  });
)

/// Logs to Android info logging.
macro_rules! a_info(
  ($msg: expr) => ( log::i($msg); );
  ($fmt: expr, $($arg:tt)*) => (
    log::i_f(format!($fmt, $($arg)*));
  );
)

/// On error, logs the error and terminates.  On success, returns the result.
macro_rules! gl_try(
  ($e: expr) => (
    match $e {
      Ok(e) => e,
      Err(e) => a_fail!("{} failed: {}", stringify!($e), e),
    }
  )
)

// Saved state data.  Compatible with C.
struct SavedState {
  angle: c_float,  // in degrees.
}

impl Default for SavedState {
  fn default() -> SavedState {
    SavedState { angle: 0.0 }
  }
}

// RAII managed EGL pointers.  Cleaned up automatically via Drop.
struct EglContext {
  display: egl::Display,
  surface: egl::Surface,
  context: egl::Context,
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

// Shared state for our app.  Compatible with C.
pub struct Engine {
  pub accelerometer_sensor: Option<&'static sensor::Sensor>,
  pub sensor_event_queue: Option<&'static sensor::EventQueue>,
  animating: bool,
  egl_context: Option<Box<EglContext>>,
  pub state: SavedState,
  // GL bound variables.
  mvp_matrix: gl::UnifLoc,
  position: gl::AttribLoc,
  color: gl::AttribLoc,
  // GL matrix
  view_projection_matrix: Matrix4<f32>,
}

impl Default for Engine {
  fn default() -> Engine {
    Engine {
      accelerometer_sensor: None,
      sensor_event_queue: None,
      egl_context: None,
      animating: false,
      state: Default::default(),
      mvp_matrix: Default::default(),
      position: Default::default(),
      color: Default::default(),
      view_projection_matrix: Matrix4::identity(),
    }
  }
}

// Red, green, and blue triangle.
static TRIANGLE_VERTICES: [f32, ..21] = [
  // X, Y, Z,
  // R, G, B, A
  -0.5, -0.25, 0.0,
  1.0, 0.0, 0.0, 1.0,

  0.5, -0.25, 0.0,
  0.0, 0.0, 1.0, 1.0,

  0.0, 0.559016994, 0.0,
  0.0, 1.0, 0.0, 1.0,
];

static VERTEX_SHADER: &'static str = "\
  uniform mat4 u_MVPMatrix;\n\
  attribute vec4 a_Position;\n\
  attribute vec4 a_Color;\n\
  varying vec4 v_Color;\n\
  void main() {\n\
    v_Color = a_Color;\n\
    gl_Position = u_MVPMatrix * a_Position;
  }\n";

static FRAGMENT_SHADER: &'static str = "\
  precision mediump float;\n\
  varying vec4 v_Color;\n\
  void main() {\n\
    gl_FragColor = v_Color;\n\
  }\n";

impl Engine {
  /// Initialize the engine.
  pub fn init(&mut self, egl_context: Box<EglContext>) {
    self.egl_context = Some(egl_context);

    // Set the background clear color to gray.
    gl::clear_color(0.5, 0.5, 0.5, 1.0);

    let (mvp_matrix, position, color) = load_program(VERTEX_SHADER, FRAGMENT_SHADER);
    self.mvp_matrix = mvp_matrix;
    self.position = position;
    self.color = color;

    // Set the vertex attributes for position and color.
    gl_try!(gl::vertex_attrib_pointer_f32(self.position, 3, 7 * 4, TRIANGLE_VERTICES));
    gl_try!(gl::enable_vertex_attrib_array(self.position));
    gl_try!(gl::vertex_attrib_pointer_f32(self.color, 4, 7 * 4, TRIANGLE_VERTICES.slice_from(3)));
    gl_try!(gl::enable_vertex_attrib_array(self.color));

    match self.egl_context {
      Some(ref ec) => {
        gl_try!(gl::viewport(0, 0, ec.width, ec.height));
        self.view_projection_matrix = view_projection_matrix(ec.width, ec.height);
      },
      None => a_fail!("self.egl_context should be present"),
    }
  }

  /// Draw a frame.
  pub fn draw(&mut self) {
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
        gl_try!(gl::draw_arrays_triangles(3));

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

  /// Handle touch and key input.  Return true if you handled event, false for any default handling.
  pub fn handle_input(&mut self, event: &input::Event) -> bool {
    match input::get_event_type(event) {
      input::Key => false,
      input::Motion => {
        let x = input::get_motion_event_x(event, 0);
        let y = input::get_motion_event_y(event, 0);
        a_info!("Touch at ({}, {})", x, y);
        return true;
      },
    }
  }

  /// Loop and handle all sensor events if any.
  pub fn handle_sensor_events(&self) {
    match self.sensor_event_queue {
      None => (),
      Some(ref event_queue) => {
        'sensor: loop {
          match sensor::get_event(*event_queue) {
            Ok(ev) => {
              self.handle_sensor(&ev);
              ()
            },
            Err(_) => break 'sensor,
          }
        }
      }
    }
  }

  /// Handle sensor input.
  #[allow(unused_variable)]
  fn handle_sensor(&self, event: &sensor::Event) {
    // Do nothing.
  }

  /// Called when window gains input focus.
  pub fn gained_focus(&mut self) {
    self.animating = true;

    // When our app gains focus, we start monitoring the accelerometer.
    match self.sensor_event_queue {
      None => (),
      Some(ref event_queue) => {
        match self.accelerometer_sensor {
          None => (),
          Some(ref sensor) => {
            enable_sensor(*event_queue, *sensor);
            // Request 60 events per second, in micros.
            sensor_event_rate(*event_queue, *sensor, 60);
          }
        }
      }
    }
  }

  /// Active when initialized and has focus.
  pub fn is_active(&self) -> bool {
    self.animating
  }

  /// Called when window loses input focus.
  pub fn lost_focus(&mut self) {
    // When our app loses focus, we stop monitoring the accelerometer.
    // This is to avoid consuming battery while not being used.
    match self.sensor_event_queue {
      None => (),
      Some(ref event_queue) => {
        match self.accelerometer_sensor {
          None => (),
          Some(ref sensor) => {
            disable_sensor(*event_queue, *sensor);
          }
        }
      }
    };
    // Also stop animating.
    self.animating = false;
    self.draw();
  }

  /// Called to save application state.  malloc some memory, copy your data into it and return
  /// together with its size.  Native activity code will free it for you later.
  pub fn save_state(&self) -> (size_t, *mut c_void) {
    let size = mem::size_of::<SavedState>() as size_t;
    let result = unsafe {
      let p = malloc(size);
      assert!(!p.is_null());
      p
    };

    let saved_state: &mut SavedState = unsafe { &mut *(result as *mut SavedState) };
    saved_state.angle = self.state.angle;

    (size, result as *mut c_void)
  }
}

fn view_projection_matrix(width: i32, height: i32) -> Matrix4<f32> {
  // Initialize a static view matrix.
  let eye = Point3::new(0.0, 0.0, 1.5);
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
  let projection_matrix = projection::frustum(left, right, bottom, top, near, far);

  projection_matrix * view_matrix
}

/// Create a matrix from a rotation around the `y` axis (yaw).
fn from_angle_y(degrees: f32) -> Matrix4<f32> {
    // http://en.wikipedia.org/wiki/Rotation_matrix#Basic_rotations
    let (s, c) = degrees.to_radians().sin_cos();
    Matrix4::new(   c, 0.0,  -s, 0.0,
                  0.0, 1.0, 0.0, 0.0,
                    s, 0.0,   c, 0.0,
                  0.0, 0.0, 0.0, 1.0)
}

fn enable_sensor(event_queue: &sensor::EventQueue, sensor: &sensor::Sensor) {
  match sensor::enable_sensor(event_queue, sensor) {
    Ok(_) => (),
    Err(e) => a_fail!("enable_sensor failed: {}", e),
  };
}

fn sensor_event_rate(event_queue: &sensor::EventQueue, sensor: &sensor::Sensor, events_per_second: i32) {
  match sensor::set_event_rate(event_queue, sensor, 1000 * 1000 / events_per_second) {
    Ok(_) => (),
    Err(e) => a_fail!("set_event_rate failed: {}", e),
  };
}

fn disable_sensor(event_queue: &sensor::EventQueue, sensor: &sensor::Sensor) {
  match sensor::disable_sensor(event_queue, sensor) {
    Ok(_) => (),
    Err(e) => a_fail!("disable_sensor failed: {}", e),
  };
}

pub fn create_egl_context(window: *const native_window::NativeWindow) -> Box<EglContext> {
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
  gl_try!(egl::choose_config(display, attribs_config, &mut configs));
  if configs.len() == 0 {
    a_fail!("choose_config() did not find any configurations");
  }
  let config = *configs.get(0);

  // EGL_NATIVE_VISUAL_ID is an attribute of the EGLConfig that is guaranteed to be accepted by
  // ANativeWindow_setBuffersGeometry().  As soon as we picked a EGLConfig, we can safely
  // reconfigure the NativeWindow buffers to match, using EGL_NATIVE_VISUAL_ID.
  let format = gl_try!(egl::get_config_attrib(display, config, egl::NATIVE_VISUAL_ID));

  native_window::set_buffers_geometry(window, 0, 0, format);

  let surface = gl_try!(egl::create_window_surface(display, config, window));

  let attribs_context = [
    egl::CONTEXT_CLIENT_VERSION, 2,
    egl::NONE
  ];
  let context = gl_try!(egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, attribs_context));

  gl_try!(egl::make_current(display, surface, surface, context));

  let w = gl_try!(egl::query_surface(display, surface, egl::WIDTH));
  let h = gl_try!(egl::query_surface(display, surface, egl::HEIGHT));

  box EglContext {
    display: display,
    surface: surface,
    context: context,
    width: w,
    height: h,
  }
}

pub fn restore_saved_state(state: *mut c_void, state_size: size_t) -> SavedState {
  if state_size == mem::size_of::<SavedState>() as size_t {
    // Compatible size, can restore.
    let saved_state: &SavedState = unsafe { &*(state as *const SavedState) };
    SavedState { angle: saved_state.angle }
  } else {
    // Incompatible size, don't even try to restore.
    Default::default()
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
    a_fail!("Compiling shader {} failed: {}", shader_type, info_log);
  }
  shader
}

fn load_program(vertex_shader_string: &str, fragment_shader_string: &str) -> (gl::UnifLoc, gl::AttribLoc, gl::AttribLoc) {
  let vertex_shader = compile_shader(vertex_shader_string, gl::VERTEX_SHADER);
  let fragment_shader = compile_shader(fragment_shader_string, gl::FRAGMENT_SHADER);
  let program = gl_try!(gl::create_program());
  gl_try!(gl::attach_shader(program, vertex_shader));
  gl_try!(gl::attach_shader(program, fragment_shader));
  gl_try!(gl::bind_attrib_location(program, 0, "a_Position"));
  gl_try!(gl::bind_attrib_location(program, 1, "a_Color"));
  gl_try!(gl::link_program(program));
  let status = gl_try!(gl::get_link_status(program));
  if !status {
    let info_log = gl_try!(gl::get_program_info_log(program));
    gl_try!(gl::delete_program(program));
    a_fail!("Linking program failed: {}", info_log);
  }
  let mvp_matrix = gl_try!(gl::get_uniform_location(program, "u_MVPMatrix"));
  let position = gl_try!(gl::get_attrib_location(program, "a_Position"));
  let color = gl_try!(gl::get_attrib_location(program, "a_Color"));
  gl_try!(gl::use_program(program));
  (mvp_matrix, position, color)
}
