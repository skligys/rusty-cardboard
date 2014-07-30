#![feature(macro_rules)]

extern crate cgmath;
extern crate libc;
extern crate time;

use libc::{c_void, int32_t};
use std::default::Default;

// To avoid warning: private type in exported type signature
pub use app::{AndroidApp, NativeActivity};
pub use asset_manager::AssetManager;
pub use input::Event;
pub use jni::JavaVm;
pub use native_window::NativeWindow;
pub use sensor::Looper;

use self::cgmath::matrix::Matrix4;

mod app;
mod asset_manager;
mod egl;
mod engine;
mod gl;
mod input;
mod jni;
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

/// Initialize EGL context for the current display.
fn init_display(app_ptr: *mut app::AndroidApp, engine: &mut engine::Engine) {
  a_info!("Renderer initializing...");
  let start_ns = time::precise_time_ns();
  let window = unsafe { (*app_ptr).window };
  let egl_context = box engine::create_egl_context(window);
  engine.init(egl_context);
  let elapsed_ms = (time::precise_time_ns() - start_ns) as f32 / 1000000.0;
  a_info!("Renderer initialized, {:.3f}ms", elapsed_ms);
}

/// Process the next input event.
#[no_mangle]
pub extern fn handle_input(app: *mut app::AndroidApp, event_ptr: *const input::Event) -> int32_t {
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

/// Process the next main command.
// Application lifecycle: APP_CMD_START, APP_CMD_RESUME, APP_CMD_INPUT_CHANGED,
// APP_CMD_INIT_WINDOW, APP_CMD_GAINED_FOCUS, ...,
// APP_CMD_SAVE_STATE, APP_CMD_PAUSE, APP_CMD_LOST_FOCUS, APP_CMD_TERM_WINDOW,
// APP_CMD_STOP.
#[no_mangle]
pub extern fn handle_cmd(app_ptr: *mut app::AndroidApp, command: int32_t) {
  let engine_ptr = unsafe { (*app_ptr).user_data as *mut engine::Engine };
  if engine_ptr.is_null() {
    a_fail!("Engine pointer is null");
  }
  let engine: &mut engine::Engine = unsafe { &mut *engine_ptr };

  match command {
    app::CMD_INIT_WINDOW => {
      // The window is being shown, get it ready.
      if unsafe { !(*app_ptr).window.is_null() } {
        init_display(app_ptr, engine);
        engine.draw();
      }
    },
    app::CMD_TERM_WINDOW => {
      // The window is being hidden or closed, clean it up.
      engine.term();
    },
    app::CMD_GAINED_FOCUS => {
      engine.gained_focus();
    },
    app::CMD_LOST_FOCUS => {
      engine.lost_focus();
    },
    app::CMD_SAVE_STATE => {
      // The system has asked us to save our current state.  Do so.
      let app: &mut app::AndroidApp = unsafe { &mut *app_ptr };
      let (size, saved_state) = engine.save_state();
      app.saved_state = saved_state;
      app.saved_state_size = size;
    },
    _ => (),
  }
}

fn rust_event_loop(app_ptr: *mut app::AndroidApp, engine_ptr: *mut engine::Engine) {
  let app: &mut app::AndroidApp = unsafe { &mut *app_ptr };
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
            let source: &app::AndroidPollSource = unsafe {
              &*(poll_result.data as *const app::AndroidPollSource)
            };
            let process = source.process;
            process(app_ptr, source as *const app::AndroidPollSource);
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
pub extern fn rust_android_main(app_ptr: *mut app::AndroidApp) {
  a_info!("-------------------------------------------------------------------");

  let app: &mut app::AndroidApp = unsafe { &mut *app_ptr };
  let activity: &app::NativeActivity = unsafe { &*app.activity };
  let jvm: &jni::JavaVm = unsafe { &*activity.vm };
  let asset_manager: &asset_manager::AssetManager = unsafe { &*activity.asset_manager };

  let mut engine = engine::Engine {
    jvm: jvm,
    asset_manager: asset_manager,
    accelerometer_sensor: sensor::get_default_sensor(sensor::TYPE_ACCELEROMETER),
    sensor_event_queue: Some(sensor::create_event_queue(app.looper, sensor::LOOPER_ID_USER)),
    animating: false,
    egl_context: None,
    state: if app.saved_state.is_null() {
      Default::default()
    } else {
      // We are starting with a previous saved state; restore from it.
      engine::restore_saved_state(app.saved_state, app.saved_state_size)
    },
    mvp_matrix: Default::default(),
    position: Default::default(),
    texture_unit: Default::default(),
    texture_coord: Default::default(),
    view_projection_matrix: Matrix4::identity(),
    texture: Default::default(),
  };

  // Notify the system about our custom data and callbacks.
  app.user_data = &engine as *const engine::Engine as *const c_void;
  app.on_app_cmd = handle_cmd as *const c_void;
  app.on_input_event = handle_input as *const c_void;

  rust_event_loop(app_ptr, &mut engine as *mut engine::Engine);
}
