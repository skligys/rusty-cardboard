use libc::{c_float, c_void, malloc, size_t};
use std::mem;
use std::ptr;
use std::default::Default;

use native_window;
use native_window::ANativeWindow;

use egl;
use gl;
use input;
use log;
use sensor;

// Macro that logs an Android error on error and terminates.
macro_rules! gl_try( ($n: expr, $e: expr) => (
  match $e {
    Ok(e) => e,
    Err(e) => {
      log::e_f(format!("{} failed: {}", $n, e));
      fail!();
    }
  }
))

// Saved state data.  Compatible with C.
struct SavedState {
  angle: c_float,
  x: i32,
  y: i32,
}

impl Default for SavedState {
  fn default() -> SavedState {
    SavedState { angle: 0.0, x: 0, y: 0 }
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
    gl_try!("egl::swap_buffers", egl::swap_buffers(self.display, self.surface));
  }
}

impl Drop for EglContext {
  fn drop(&mut self) {
    if self.display != egl::NO_DISPLAY {
      gl_try!("egl::make_current",
        egl::make_current(self.display, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT));
      if self.context != egl::NO_CONTEXT {
        gl_try!("egl::destroy_context", egl::destroy_context(self.display, self.context));
        self.context = egl::NO_CONTEXT;
      }
      if self.surface != egl::NO_SURFACE {
        gl_try!("egl::destroy_surface", egl::destroy_surface(self.display, self.surface));
        self.surface = egl::NO_SURFACE;
      }
      gl_try!("egl::terminate", egl::terminate(self.display));
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
}

impl Default for Engine {
  fn default() -> Engine {
    Engine {
      accelerometer_sensor: None,
      sensor_event_queue: None,
      egl_context: None,
      animating: false,
      state: Default::default(),
    }
  }
}

impl Engine {
  /// Initialize the engine.
  pub fn init(&mut self, egl_context: Box<EglContext>) {
    self.egl_context = Some(egl_context);
    self.state.angle = 0.0;

    gl_try!("gl::enable(gl::CULL_FACE)", gl::enable(gl::CULL_FACE));
    gl_try!("gl::disable(gl::DEPTH_TEST)", gl::disable(gl::DEPTH_TEST));
  }

  /// Draw a frame.
  pub fn draw(&mut self) {
    match self.egl_context {
      None => return,  // No display.
      Some(ref egl_context) => {
        // Just fill the screen with a color.
        let r = (self.state.x as f32) / (egl_context.width as f32);
        let g = self.state.angle;
        let b = (self.state.y as f32) / (egl_context.height as f32);
        gl::clear_color(r, g, b, 1.0);

        gl_try!("gl::clear(gl::COLOR_BUFFER_BIT)", gl::clear(gl::COLOR_BUFFER_BIT));
        egl_context.swap_buffers();
      }
    }
  }

  /// Update for time passed and draw a frame.
  pub fn update_draw(&mut self) {
    if self.animating {
      // Done processing events; draw next animation frame.
      self.state.angle += 0.01;
      if self.state.angle > 1.0 {
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
    log::i("Renderer terminated");
  }

  /// Handle touch and key input.  Return true if you handled event, false for any default handling.
  pub fn handle_input(&mut self, event: &input::Event) -> bool {
    match input::get_event_type(event) {
      input::Key => false,
      input::Motion => {
        self.state.x = input::get_motion_event_x(event, 0) as i32;
        self.state.y = input::get_motion_event_y(event, 0) as i32;
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
    saved_state.x = self.state.x;
    saved_state.y = self.state.y;

    (size, result as *mut c_void)
  }
}

fn enable_sensor(event_queue: &sensor::EventQueue, sensor: &sensor::Sensor) {
  match sensor::enable_sensor(event_queue, sensor) {
    Ok(_) => (),
    Err(e) => {
      log::e_f(format!("enable_sensor failed: {}", e));
      fail!();
    }
  };
}

fn sensor_event_rate(event_queue: &sensor::EventQueue, sensor: &sensor::Sensor, events_per_second: i32) {
  match sensor::set_event_rate(event_queue, sensor, 1000 * 1000 / events_per_second) {
    Ok(_) => (),
    Err(e) => {
      log::e_f(format!("set_event_rate failed: {}", e));
      fail!();
    }
  };
}

fn disable_sensor(event_queue: &sensor::EventQueue, sensor: &sensor::Sensor) {
  match sensor::disable_sensor(event_queue, sensor) {
    Ok(_) => (),
    Err(e) => {
      log::e_f(format!("disable_sensor failed: {}", e));
      fail!();
    }
  };
}

pub fn create_egl_context(window: *const ANativeWindow) -> Box<EglContext> {
  let display = egl::get_display(egl::DEFAULT_DISPLAY);

  gl_try!("egl::initialize", egl::initialize(display));

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
  gl_try!("egl::choose_config", egl::choose_config(display, attribs_config, &mut configs));
  if configs.len() == 0 {
    log::e("choose_config() did not find any configurations");
    fail!();
  }
  let config = *configs.get(0);

  // EGL_NATIVE_VISUAL_ID is an attribute of the EGLConfig that is guaranteed to be accepted by
  // ANativeWindow_setBuffersGeometry().  As soon as we picked a EGLConfig, we can safely
  // reconfigure the ANativeWindow buffers to match, using EGL_NATIVE_VISUAL_ID.
  let format = gl_try!("egl::get_config_attrib",
    egl::get_config_attrib(display, config, egl::NATIVE_VISUAL_ID));

  native_window::set_buffers_geometry(window, 0, 0, format);

  let surface = gl_try!("egl::create_window_surface", egl::create_window_surface(display, config, window));

  let attribs_context = [
    egl::CONTEXT_CLIENT_VERSION, 2,
    egl::NONE
  ];
  let context = gl_try!("egl::create_context_with_attribs",
    egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, attribs_context));

  gl_try!("egl::make_current", egl::make_current(display, surface, surface, context));

  let w = gl_try!("egl::query_surface(egl::WIDTH)", egl::query_surface(display, surface, egl::WIDTH));
  let h = gl_try!("egl::query_surface(egl::HEIGHT)", egl::query_surface(display, surface, egl::HEIGHT));

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
    SavedState { angle: saved_state.angle, x: saved_state.x, y: saved_state.y }
  } else {
    // Incompatible size, don't even try to restore.
    Default::default()
  }
}
