#![feature(macro_rules)]

extern crate libc;
extern crate time;

use libc::{c_int, c_void, int32_t, size_t};
use std::default::Default;
pub use input::Event;

mod egl;
mod engine;
mod gl;
mod input;
mod native_window;
mod log;
mod sensor;

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

/**
 * This structure defines the native side of an android.app.NativeActivity.  It is created by
 * the framework, and handed to the application's native code as it is being launched.
 */
struct NativeActivity;

/// Opaque structure representing Android configuration.
struct Configuration;

struct Rect {
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
  // The NativeActivity object instance that this app is running in.
  #[allow(dead_code)]
  activity: *const NativeActivity,
  // The current configuration the app is running in.
  #[allow(dead_code)]
  config: *const Configuration,
  // This is the last instance's saved state, as provided at creation time.  It is NULL if there
  // was no state.  You can use this as you need; the memory will remain around until you call
  // android_app_exec_cmd() for APP_CMD_RESUME, at which point it will be freed and savedState
  // set to NULL.  These variables should only be changed when processing a APP_CMD_SAVE_STATE,
  // at which point they will be initialized to NULL and you can malloc your state and place
  // the information here.  In that case the memory will be freed for you later.
  saved_state: *mut c_void,
  saved_state_size: size_t,
  // The looper associated with the app's thread.
  looper: *const sensor::Looper,
  // When non-NULL, this is the input queue from which the app will receive user input events.
  #[allow(dead_code)]
  input_queue: *const input::Queue,
  // When non-NULL, this is the window surface that the app can draw in.
  window: *const native_window::NativeWindow,
  // Current content rectangle of the window; this is the area where the window's content should be
  // placed to be seen by the user.
  #[allow(dead_code)]
  content_rect: Rect,
  // Current state of the app's activity.  May be either APP_CMD_START, APP_CMD_RESUME,
  // APP_CMD_PAUSE, or APP_CMD_STOP; see below.
  #[allow(dead_code)]
  activity_state: c_int,
  // This is non-zero when the application's NativeActivity is being destroyed and waiting for
  // the app thread to complete.
  destroy_requested: c_int,
  // Plus some private implementation details.
}

/// Initialize EGL context for the current display.
fn init_display(app_ptr: *mut AndroidApp, engine: &mut engine::Engine) {
  a_info!("Renderer initializing...");
  let start_ns = time::precise_time_ns();
  let window = unsafe { (*app_ptr).window };
  let egl_context = engine::create_egl_context(window);
  engine.init(egl_context);
  let elapsed_ms = (time::precise_time_ns() - start_ns) as f32 / 1000000.0;
  a_info!("Renderer initialized, {:.3f}ms", elapsed_ms);
}

/// Process the next input event.
#[no_mangle]
pub extern fn handle_input(app: *mut AndroidApp, event_ptr: *const input::Event) -> int32_t {
  let engine_ptr = unsafe { (*app).user_data as *mut engine::Engine };
  if engine_ptr.is_null() {
    a_fail!("Engine pointer is null");
  }
  let engine: &mut engine::Engine = unsafe { &mut *engine_ptr };
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
  let engine_ptr = unsafe { (*app_ptr).user_data as *mut engine::Engine };
  if engine_ptr.is_null() {
    a_fail!("Engine pointer is null");
  }
  let engine: &mut engine::Engine = unsafe { &mut *engine_ptr };

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
 * Data associated with an Looper fd that will be returned as the "data" when that source has
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

fn rust_event_loop(app_ptr: *mut AndroidApp, engine_ptr: *mut engine::Engine) {
  let app: &mut AndroidApp = unsafe { &mut *app_ptr };
  let engine: &mut engine::Engine = unsafe { &mut *engine_ptr };

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
  a_info!("-------------------------------------------------------------------");

  let app: &mut AndroidApp = unsafe { &mut *app_ptr };
  let mut engine = engine::Engine {
    accelerometer_sensor: sensor::get_default_sensor(sensor::TYPE_ACCELEROMETER),
    sensor_event_queue: Some(sensor::create_event_queue(app.looper, sensor::LOOPER_ID_USER)),
    state: if app.saved_state.is_null() {
      Default::default()
    } else {
      // We are starting with a previous saved state; restore from it.
      engine::restore_saved_state(app.saved_state, app.saved_state_size)
    },
    ..Default::default()};

  // Notify the system about our custom data and callbacks.
  app.user_data = &engine as *const engine::Engine as *const c_void;
  app.on_app_cmd = handle_cmd as *const c_void;
  app.on_input_event = handle_input as *const c_void;

  rust_event_loop(app_ptr, &mut engine as *mut engine::Engine);
}
