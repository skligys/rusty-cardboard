#![feature(macro_rules)]

extern crate libc;
use libc::{c_float, c_int, c_void, int32_t, malloc, size_t};
use native_window::ANativeWindow;
use std::default::Default;
use std::mem;
use std::ptr;
pub use input::Event;

mod egl;
mod gl;
mod input;
mod native_window;
mod log;
mod sensor;

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

/**
 * This structure defines the native side of an android.app.NativeActivity.
 * It is created by the framework, and handed to the application's native
 * code as it is being launched.
 */
struct ANativeActivity;

/// Opaque structure representing Android configuration.
struct AConfiguration;

struct ARect {
  #[allow(dead_code)]
  left: i32,
  #[allow(dead_code)]
  top: i32,
  #[allow(dead_code)]
  right: i32,
  #[allow(dead_code)]
  bottom: i32,
}

// This is the interface for the standard glue code of a threaded application.  In this model, the
// application's code is running in its own thread separate from the main thread of the process.
// It is not required that this thread be associated with the Java VM, although it will need to be
// in order to make JNI calls to any Java objects.  Compatible with C.
pub struct AndroidApp {
  // The application can place a pointer to its own state object here if it likes.
  user_data: *const c_void,
  // Fill this in with the function to process main app commands (APP_CMD_*)
  // TODO: implement.
  on_app_cmd: *const c_void,
  // Fill this in with the function to process input events.  At this point the event has already
  // been pre-dispatched, and it will be finished upon return.  Return 1 if you have handled
  // the event, 0 for any default dispatching.
  on_input_event: *const c_void,
  // The ANativeActivity object instance that this app is running in.
  #[allow(dead_code)]
  activity: *const ANativeActivity,
  // The current configuration the app is running in.
  #[allow(dead_code)]
  config: *const AConfiguration,
  // This is the last instance's saved state, as provided at creation time.  It is NULL if there
  // was no state.  You can use this as you need; the memory will remain around until you call
  // android_app_exec_cmd() for APP_CMD_RESUME, at which point it will be freed and savedState
  // set to NULL.  These variables should only be changed when processing a APP_CMD_SAVE_STATE,
  // at which point they will be initialized to NULL and you can malloc your state and place
  // the information here.  In that case the memory will be freed for you later.
  saved_state: *mut c_void,
  saved_state_size: size_t,
  // The looper associated with the app's thread.
  looper: *const sensor::ALooper,
  // When non-NULL, this is the input queue from which the app will receive user input events.
  #[allow(dead_code)]
  input_queue: *const input::Queue,
  // When non-NULL, this is the window surface that the app can draw in.
  window: *const ANativeWindow,
  // Current content rectangle of the window; this is the area where the window's content should be
  // placed to be seen by the user.
  #[allow(dead_code)]
  content_rect: ARect,
  // Current state of the app's activity.  May be either APP_CMD_START, APP_CMD_RESUME,
  // APP_CMD_PAUSE, or APP_CMD_STOP; see below.
  #[allow(dead_code)]
  activity_state: c_int,
  // This is non-zero when the application's NativeActivity is being destroyed and waiting for
  // the app thread to complete.
  destroy_requested: c_int,
  // Plus some private implementation details.
}

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
  accelerometer_sensor: Option<&'static sensor::Sensor>,
  sensor_event_queue: Option<&'static sensor::EventQueue>,
  animating: bool,
  egl_context: Option<Box<EglContext>>,
  state: SavedState,
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

trait Renderer {
  /// Initialize the renderer.
  fn init(&mut self, egl_context: Box<EglContext>);
  /// Draw a frame.
  fn draw(&mut self);
  /// Update for time passed and draw a frame.
  fn update_draw(&mut self);
  /// Terminate the renderer.
  fn term(&mut self);
  /// Handle touch and key input.  Return true if you handled event, false for any default handling.
  fn handle_input(&mut self, event: &input::Event) -> bool;
  /// Loop and handle all sensor events if any.
  fn handle_sensor_events(&self);
  /// Handle sensor input.
  fn handle_sensor(&self, event: &sensor::Event);
  /// Called when window gains input focus.
  fn gained_focus(&mut self);
  /// Active when initialized and has focus.
  fn is_active(&self) -> bool;
  /// Called when window loses input focus.
  fn lost_focus(&mut self);
  /// Called to save application state.  malloc some memory, copy your data into it and return
  /// together with its size.  Native activity code will free it for you later.
  fn save_state(&self) -> (size_t, *mut c_void);
}

impl Renderer for Engine {
  /// Initialize the engine.
  fn init(&mut self, egl_context: Box<EglContext>) {
    self.egl_context = Some(egl_context);
    self.state.angle = 0.0;

    gl_try!("gl::enable(gl::CULL_FACE)", gl::enable(gl::CULL_FACE));
    gl_try!("gl::disable(gl::DEPTH_TEST)", gl::disable(gl::DEPTH_TEST));
  }

  /// Draw a frame.
  fn draw(&mut self) {
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
  fn update_draw(&mut self) {
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
  fn term(&mut self) {
    self.animating = false;
    self.egl_context = None;  // This closes the existing context via Drop.
    log::i("Terminated");
  }

  /// Handle touch and key input.  Return true if you handled event, false for any default handling.
  fn handle_input(&mut self, event: &input::Event) -> bool {
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
  fn handle_sensor_events(&self) {
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
  fn gained_focus(&mut self) {
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
  fn is_active(&self) -> bool {
    self.animating
  }

  /// Called when window loses input focus.
  fn lost_focus(&mut self) {
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
  fn save_state(&self) -> (size_t, *mut c_void) {
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

fn create_egl_context(window: *const ANativeWindow) -> Box<EglContext> {
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

/// Initialize EGL context for the current display.
fn init_display(app_ptr: *mut AndroidApp, engine: &mut Engine) {
  let window = unsafe { (*app_ptr).window };
  let egl_context = create_egl_context(window);
  engine.init(egl_context);
}

/// Process the next input event.
#[no_mangle]
pub extern fn handle_input(app: *mut AndroidApp, event_ptr: *const input::Event) -> int32_t {
  let engine_ptr = unsafe { (*app).user_data as *mut Engine };
  if engine_ptr.is_null() {
    fail!("Engine pointer is null");
  }
  let engine: &mut Engine = unsafe { &mut *engine_ptr };
  let event: &input::Event = unsafe { &*event_ptr };
  match engine.handle_input(event) {
    true => 1,
    false => 0,
  }
}

// Native app glue command enums:
static APP_CMD_INIT_WINDOW: int32_t = 1;
static APP_CMD_TERM_WINDOW: int32_t = 2;
static APP_CMD_GAINED_FOCUS: int32_t = 6;
static APP_CMD_LOST_FOCUS: int32_t = 7;
static APP_CMD_SAVE_STATE: int32_t = 12;

/// Process the next main command.
// Application lifecycle: APP_CMD_START, APP_CMD_RESUME, APP_CMD_INPUT_CHANGED,
// APP_CMD_INIT_WINDOW, APP_CMD_GAINED_FOCUS, ...,
// APP_CMD_SAVE_STATE, APP_CMD_PAUSE, APP_CMD_LOST_FOCUS, APP_CMD_TERM_WINDOW,
// APP_CMD_STOP.
#[no_mangle]
pub extern fn handle_cmd(app_ptr: *mut AndroidApp, command: int32_t) {
  let engine_ptr = unsafe { (*app_ptr).user_data as *mut Engine };
  if engine_ptr.is_null() {
    fail!("Engine pointer is null");
  }
  let engine: &mut Engine = unsafe { &mut *engine_ptr };

  match command {
    APP_CMD_INIT_WINDOW => {
      // The window is being shown, get it ready.
      if unsafe { !(*app_ptr).window.is_null() } {
        init_display(app_ptr, engine);
        engine.draw();
      }
    },
    APP_CMD_TERM_WINDOW => {
      // The window is being hidden or closed, clean it up.
      engine.term();
    },
    APP_CMD_GAINED_FOCUS => {
      engine.gained_focus();
    },
    APP_CMD_LOST_FOCUS => {
      engine.lost_focus();
    },
    APP_CMD_SAVE_STATE => {
      // The system has asked us to save our current state.  Do so.
      let app: &mut AndroidApp = unsafe { &mut *app_ptr };
      let (size, saved_state) = engine.save_state();
      app.saved_state = saved_state;
      app.saved_state_size = size;
    },
    _ => (),
  }
}

/**
 * Data associated with an ALooper fd that will be returned as the "data" when that source has
 * data ready.
 */
struct AndroidPollSource {
  /// The identifier of this source.  May be LOOPER_ID_MAIN or LOOPER_ID_INPUT.
  #[allow(dead_code)]
  id: int32_t,
  /// The android_app this ident is associated with.
  #[allow(dead_code)]
  app: *const AndroidApp,
  /// Function to call to perform the standard processing of data from this source.
  process: extern "C" fn (app: *mut AndroidApp, source: *const AndroidPollSource),
}

fn rust_event_loop(app_ptr: *mut AndroidApp, engine_ptr: *mut Engine) {
  let app: &mut AndroidApp = unsafe { &mut *app_ptr };
  let engine: &mut Engine = unsafe { &mut *engine_ptr };

  // Loop waiting for stuff to do.
  loop {
    'inner: loop {
      // Block polling when not animating.
      let poll_timeout = if engine.is_active() { 0 } else { -1 };
      match sensor::poll_all(poll_timeout) {
        Err(_) => break 'inner,
        Ok(poll_result) => {
          // Process this event.
          if !poll_result.data.is_null() {
            let source: &AndroidPollSource = unsafe {
              &*(poll_result.data as *const AndroidPollSource)
            };
            let process = source.process;
            process(app_ptr, source as *const AndroidPollSource);
          }

          // If the sensor has data, process it now.
          if poll_result.id == sensor::LOOPER_ID_USER {
            engine.handle_sensor_events();
          }

          // Check if should exit.
          if app.destroy_requested != 0 {
            engine.term();
            return;
          }
        }
      }
    }
    engine.update_draw();
  }
}

/**
 * This is the main entry point of a native application that is using android_native_app_glue.
 * It runs in its own thread, with its own event loop for receiving input events and doing other
 * things.
 */
#[no_mangle]
pub extern fn rust_android_main(app_ptr: *mut AndroidApp) {
  log::i("-------------------------------------------------------------------");

  let app: &mut AndroidApp = unsafe { &mut *app_ptr };
  let mut engine = Engine {
    accelerometer_sensor: sensor::get_default_sensor(sensor::TYPE_ACCELEROMETER),
    sensor_event_queue: Some(sensor::create_event_queue(app.looper, sensor::LOOPER_ID_USER)),
    state: if app.saved_state.is_null() {
      Default::default()
    } else {
      // We are starting with a previous saved state; restore from it.
      let app_saved_state: &SavedState = unsafe { &*(app.saved_state as *const SavedState) };
      SavedState { angle: app_saved_state.angle, x: app_saved_state.x, y: app_saved_state.y }
    },
    ..Default::default()};

  // Notify the system about our custom data and callbacks.
  app.user_data = &engine as *const Engine as *const c_void;
  app.on_app_cmd = handle_cmd as *const c_void;
  app.on_input_event = handle_input as *const c_void;

  rust_event_loop(app_ptr, &mut engine as *mut Engine);
}
