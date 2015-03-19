#![feature(collections, start, std_misc, unsafe_destructor)]

#[macro_use]
extern crate android_glue;

extern crate cgmath;
extern crate libc;
extern crate time;

use libc::{c_void, int32_t};
use std::default::Default;
use std::mem;

use cgmath::Matrix4;

mod asset_manager;
mod egl;
mod engine;
mod gl;
mod input;
mod jni;
mod sensor;

// TODO: Figure out how to put macros in a separate module and import when needed.

/// Logs the error to Android error logging and fails.
macro_rules! a_panic(
  ($msg: expr) => (
    panic!($msg);
  );
  ($fmt: expr, $($arg:tt)*) => (
    panic!($fmt, $($arg)*);
  );
);

/// Logs to Android info logging.
macro_rules! a_info(
  ($msg: expr) => (
    println!($msg);
  );
  ($fmt: expr, $($arg:tt)*) => (
    println!($fmt, $($arg)*);
  );
);

/// Initialize EGL context for the current display.
fn init_display(app_ptr: *mut android_glue::ffi::android_app, engine: &mut engine::Engine) {
  a_info!("Renderer initializing...");
  let start_ns = time::precise_time_ns();
  let window = unsafe { (*app_ptr).window as *mut android_glue::ffi::ANativeWindow};
  let egl_context = Box::new(engine::create_egl_context(window));
  engine.init(egl_context);
  let elapsed_ms = (time::precise_time_ns() - start_ns) as f32 / 1000000.0;
  a_info!("Renderer initialized, {:.3}ms", elapsed_ms);
}

/// Process the next input event.
#[no_mangle]
pub extern fn handle_input(app: *mut android_glue::ffi::android_app,
  event_ptr: *const android_glue::ffi::AInputEvent) -> int32_t {

  let engine_ptr = unsafe { (*app).userData as *mut engine::Engine };
  if engine_ptr.is_null() {
    a_panic!("Engine pointer is null");
  }
  let engine: &mut engine::Engine = unsafe { &mut *engine_ptr };
  let event: &android_glue::ffi::AInputEvent = unsafe { &*event_ptr };
  match engine.handle_input(event) {
    true => 1,
    false => 0,
  }
}

/// Process the next main command.
// Application lifecycle: APP_CMD_START, APP_CMD_RESUME, APP_CMD_INPUT_CHANGED,
// APP_CMD_INIT_WINDOW, APP_CMD_GAINED_FOCUS, ...,
// APP_CMD_SAVE_STATE, APP_CMD_PAUSE, APP_CMD_LOST_FOCUS, APP_CMD_TERM_WINDOW,
// APP_CMD_STOP.
#[no_mangle]
pub extern fn handle_cmd(app_ptr: *mut android_glue::ffi::android_app, command: int32_t) {
  let engine_ptr = unsafe { (*app_ptr).userData as *mut engine::Engine };
  if engine_ptr.is_null() {
    a_panic!("Engine pointer is null");
  }
  let engine: &mut engine::Engine = unsafe { &mut *engine_ptr };

  match command {
    android_glue::ffi::APP_CMD_INIT_WINDOW => {
      // The window is being shown, get it ready.
      if unsafe { !(*app_ptr).window.is_null() } {
        init_display(app_ptr, engine);
        engine.draw();
      }
    },
    android_glue::ffi::APP_CMD_TERM_WINDOW => {
      // The window is being hidden or closed, clean it up.
      engine.term();
    },
    android_glue::ffi::APP_CMD_GAINED_FOCUS => {
      engine.gained_focus();
    },
    android_glue::ffi::APP_CMD_LOST_FOCUS => {
      engine.lost_focus();
    },
    android_glue::ffi::APP_CMD_SAVE_STATE => {
      // The system has asked us to save our current state.  Do so.
      let app: &mut android_glue::ffi::android_app = unsafe { &mut *app_ptr };
      let (size, saved_state) = engine.save_state();
      app.savedState = saved_state;
      app.savedStateSize = size;
    },
    _ => (),
  }
}

fn rust_event_loop(app: &mut android_glue::ffi::android_app, engine: &mut engine::Engine) {
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
            let source: &mut android_glue::ffi::android_poll_source = unsafe {
              &mut *(poll_result.data as *mut android_glue::ffi::android_poll_source)
            };
            let process = source.process;
            process(app as *mut android_glue::ffi::android_app, source as *mut android_glue::ffi::android_poll_source);
          }

          // If the sensor has data, process it now.
          if poll_result.id == android_glue::ffi::LOOPER_ID_USER {
            engine.handle_sensor_events();
          }

          // Check if should exit.
          if app.destroyRequested != 0 {
            engine.term();
            return;
          }
        }
      }
    }
    engine.update_draw();
  }
}

#[cfg(target_os = "android")]
android_start!(main);

/**
 * This is the main entry point of a native application that is using android_native_app_glue.
 * It runs in its own thread, with its own event loop for receiving input events and doing other
 * things.
 */
pub fn main() {
  a_info!("-------------------------------------------------------------------");

  let app = android_glue::get_app();
  let activity: &mut android_glue::ffi::ANativeActivity =
    unsafe { &mut *(app.activity as *mut android_glue::ffi::ANativeActivity) };
  let jvm: &mut android_glue::ffi::JavaVM = unsafe { &mut *activity.vm };
  let asset_manager: &mut android_glue::ffi::AAssetManager = unsafe { &mut *activity.assetManager };

  let mut engine = engine::Engine {
    jvm: jvm,
    asset_manager: asset_manager,
    accelerometer_sensor: sensor::get_default_sensor(android_glue::ffi::ASENSOR_TYPE_ACCELEROMETER),
    sensor_event_queue: sensor::create_event_queue(app.looper, android_glue::ffi::LOOPER_ID_USER),
    animating: false,
    egl_context: None,
    state: if app.savedState.is_null() {
      Default::default()
    } else {
      // We are starting with a previous saved state; restore from it.
      engine::restore_saved_state(app.savedState, app.savedStateSize)
    },
    mvp_matrix: Default::default(),
    position: Default::default(),
    texture_unit: Default::default(),
    texture_coord: Default::default(),
    view_projection_matrix: Matrix4::identity(),
    texture: Default::default(),
  };

  // Notify the system about our custom data and callbacks.
  app.userData = &mut engine as *mut engine::Engine as *mut c_void;
  app.onAppCmd = unsafe { mem::transmute(handle_cmd) };
  app.onInputEvent = unsafe { mem::transmute(handle_input) };

  rust_event_loop(app, &mut engine);
}
