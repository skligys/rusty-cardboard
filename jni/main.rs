#![feature(macro_rules)]

extern crate libc;
use libc::{c_float, c_int, c_void, int32_t, malloc, size_t};
use native_window::ANativeWindow;
use std::mem;
use std::ptr;

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

/**
 * A looper is the state tracking an event loop for a thread.
 * Loopers do not define event structures or other such things; rather
 * they are a lower-level facility to attach one or more discrete objects
 * listening for an event.  An "event" here is simply data available on
 * a file descriptor: each attached object has an associated file descriptor,
 * and waiting for "events" means (internally) polling on all of these file
 * descriptors until one or more of them have data available.
 *
 * A thread can have only one ALooper associated with it.
*/
struct ALooper;

struct ARect {
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
}

// This is the interface for the standard glue code of a threaded application.  In this model, the
// application's code is running in its own thread separate from the main thread of the process.
// It is not required that this thread be associated with the Java VM, although it will need to be
// in order to make JNI calls to any Java objects.  Compatible with C.
struct AndroidApp {
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
  activity: *const ANativeActivity,
  // The current configuration the app is running in.
  config: *const AConfiguration,
  // This is the last instance's saved state, as provided at creation time.  It is NULL if there
  // was no state.  You can use this as you need; the memory will remain around until you call
  // android_app_exec_cmd() for APP_CMD_RESUME, at which point it will be freed and savedState
  // set to NULL.  These variables should only be changed when processing a APP_CMD_SAVE_STATE,
  // at which point they will be initialized to NULL and you can malloc your state and place
  // the information here.  In that case the memory will be freed for you later.
  saved_state: *mut c_void,
  saved_state_size: size_t,
  // The ALooper associated with the app's thread.
  looper: *const ALooper,
  // When non-NULL, this is the input queue from which the app will receive user input events.
  input_queue: *const input::Queue,
  // When non-NULL, this is the window surface that the app can draw in.
  window: *const ANativeWindow,
  // Current content rectangle of the window; this is the area where the window's content should be
  // placed to be seen by the user.
  content_rect: ARect,
  // Current state of the app's activity.  May be either APP_CMD_START, APP_CMD_RESUME,
  // APP_CMD_PAUSE, or APP_CMD_STOP; see below.
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

// Shared state for our app.  Compatible with C.
struct Engine {
  app: *mut AndroidApp,

  sensor_manager: *const sensor::Manager,
  accelerometer_sensor: *const sensor::Sensor,
  sensor_event_queue: *mut sensor::EventQueue,

  animating: c_int,
  display: egl::Display,
  surface: egl::Surface,
  context: egl::Context,
  width: i32,
  height: i32,
  state: SavedState,
}

/// Initialize EGL context for the current display.
#[no_mangle]
pub extern fn init_display(engine: &mut Engine) -> c_int {
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

  let window = unsafe { (*engine.app).window };
  native_window::set_buffers_geometry(window, 0, 0, format);

  let surface = gl_try!("egl::create_window_surface", egl::create_window_surface(display, config, window));

  let attribs_context = [
    egl::CONTEXT_CLIENT_VERSION, 2,
    egl::NONE
  ];
  let context = gl_try!("egl::create_context_with_attribs",
    egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, attribs_context));

  gl_try!("egl::make_current", egl::make_current(display, surface, surface, context));

  let version = gl_try!("gl::get_string(gl::VERSION)", gl::get_string(gl::VERSION));
  let vendor = gl_try!("gl::get_string(gl::VENDOR)", gl::get_string(gl::VENDOR));
  let renderer = gl_try!("gl::get_string(gl::RENDERER)", gl::get_string(gl::RENDERER));
  let sl_version = gl_try!("gl::get_string(gl::SHADING_LANGUAGE_VERSION)",
    gl::get_string(gl::SHADING_LANGUAGE_VERSION));
  log::i_f(format!("OpenGL version: \"{}\", vendor: \"{}\", renderer: \"{}\", SL version: \"{}\"",
    version.as_str().unwrap(), vendor.as_str().unwrap(), renderer.as_str().unwrap(),
    sl_version.as_str().unwrap()));

  let extensions = gl_try!("gl::get_string(gl::EXTENSIONS)", gl::get_string(gl::EXTENSIONS));
  log::i_f(format!("OpenGL extensions: \"{}\"", extensions.as_str().unwrap()));

  let w = gl_try!("egl::query_surface(egl::WIDTH)", egl::query_surface(display, surface, egl::WIDTH));
  let h = gl_try!("egl::query_surface(egl::HEIGHT)", egl::query_surface(display, surface, egl::HEIGHT));

  engine.display = display;
  engine.context = context;
  engine.surface = surface;
  engine.width = w;
  engine.height = h;
  engine.state.angle = 0.0;

  gl_try!("gl::enable(gl::CULL_FACE)", gl::enable(gl::CULL_FACE));
  gl_try!("gl::disable(gl::DEPTH_TEST)", gl::disable(gl::DEPTH_TEST));

  return 0;
}

/// Draw the current frame on display.
#[no_mangle]
pub extern fn draw_frame(engine: &Engine) {
  if engine.display == ptr::null() {
    // No display.
    return;
  }

  // Just fill the screen with a color.
  let r = (engine.state.x as f32) / (engine.width as f32);
  let g = engine.state.angle;
  let b = (engine.state.y as f32) / (engine.height as f32);
  gl::clear_color(r, g, b, 1.0);

  gl_try!("gl::clear(gl::COLOR_BUFFER_BIT)", gl::clear(gl::COLOR_BUFFER_BIT));
  gl_try!("egl::swap_buffers", egl::swap_buffers(engine.display, engine.surface));
}

/// Tear down the EGL context currently associated with the display.
#[no_mangle]
pub extern fn term_display(engine: &mut Engine) {
  if engine.display != egl::NO_DISPLAY {
    gl_try!("egl::make_current",
      egl::make_current(engine.display, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT));
    if engine.context != egl::NO_CONTEXT {
      gl_try!("egl::destroy_context", egl::destroy_context(engine.display, engine.context));
    }
    if engine.surface != egl::NO_SURFACE {
      gl_try!("egl::destroy_surface", egl::destroy_surface(engine.display, engine.surface));
    }
    gl_try!("egl::terminate", egl::terminate(engine.display));
  }

  engine.animating = 0;
  engine.display = egl::NO_DISPLAY;
  engine.context = egl::NO_CONTEXT;
  engine.surface = egl::NO_SURFACE;
}

/// Process the next input event.
#[no_mangle]
pub extern fn handle_input(app: *mut AndroidApp, event: *const input::Event) -> int32_t {
  let engine_ptr = unsafe { (*app).user_data as *mut Engine };
  if engine_ptr.is_null() {
    fail!("Engine pointer is null");
  }
  let engine: &mut Engine = unsafe { mem::transmute(engine_ptr) };

  match input::get_event_type(event) {
    input::Key => 0,
    input::Motion => {
      engine.animating = 1;
      engine.state.x = input::get_motion_event_x(event, 0) as i32;
      engine.state.y = input::get_motion_event_y(event, 0) as i32;
      return 1;
    },
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
pub extern fn handle_cmd(app: *mut AndroidApp, command: int32_t) {
  let engine_ptr = unsafe { (*app).user_data as *mut Engine };
  if engine_ptr.is_null() {
    fail!("Engine pointer is null");
  }
  let engine: &mut Engine = unsafe { mem::transmute(engine_ptr) };

  match command {
    APP_CMD_INIT_WINDOW => {
      // The window is being shown, get it ready.
      if unsafe { !(*engine.app).window.is_null() } {
        init_display(engine);
        draw_frame(engine);
      }
    },
    APP_CMD_TERM_WINDOW => {
      // The window is being hidden or closed, clean it up.
      term_display(engine);
    },
    APP_CMD_GAINED_FOCUS => {
      // When our app gains focus, we start monitoring the accelerometer.
      if !engine.accelerometer_sensor.is_null() {
        match sensor::enable_sensor(engine.sensor_event_queue, engine.accelerometer_sensor) {
          Ok(_) => (),
          Err(e) => {
            log::e_f(format!("enable_sensor failed: {}", e));
            fail!();
          }
        };
        // Request 60 events per second, in micros.
        match sensor::set_event_rate(engine.sensor_event_queue, engine.accelerometer_sensor,
          1000 * 1000 / 60) {
          Ok(_) => (),
          Err(e) => {
            log::e_f(format!("set_event_rate failed: {}", e));
            fail!();
          }
        };
      }
    },
    APP_CMD_LOST_FOCUS => {
      // When our app loses focus, we stop monitoring the accelerometer.
      // This is to avoid consuming battery while not being used.
      if !engine.accelerometer_sensor.is_null() {
        match sensor::disable_sensor(engine.sensor_event_queue, engine.accelerometer_sensor) {
          Ok(_) => (),
          Err(e) => {
            log::e_f(format!("disable_sensor failed: {}", e));
            fail!();
          }
        }
      }
      // Also stop animating.
      engine.animating = 0;
      draw_frame(engine);
    },
    APP_CMD_SAVE_STATE => {
      // The system has asked us to save our current state.  Do so.
      // This leaks memory every time the command is processed.
      let size = mem::size_of::<SavedState>();
      unsafe {
        (*engine.app).saved_state = malloc(size as size_t);
        ptr::copy_memory((*engine.app).saved_state, &engine.state as *const SavedState as *const c_void, size);
        (*engine.app).saved_state_size = size as size_t;
      }
    },
    _ => (),
  }
}
