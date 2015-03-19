#![feature(start, std_misc, unsafe_destructor)]

#[macro_use]
extern crate android_glue;

extern crate cgmath;
extern crate libc;
extern crate time;

use std::default::Default;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

use android_glue::Event;
use cgmath::Matrix4;

mod egl;
mod engine;
mod gl;

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

#[cfg(target_os = "android")]
android_start!(main);

/// Initialize EGL context for the current display.
fn init_display(engine: &mut engine::Engine) {
  a_info!("Renderer initializing...");
  let start_ns = time::precise_time_ns();
  let app = android_glue::get_app();
  let window = app.window as *mut android_glue::ffi::ANativeWindow;
  let egl_context = Box::new(engine::create_egl_context(window));
  engine.init(egl_context);
  let elapsed_ms = (time::precise_time_ns() - start_ns) as f32 / 1000000.0;
  a_info!("Renderer initialized, {:.3}ms", elapsed_ms);
}

/**
 * This is the main entry point of a native application that is using android_native_app_glue.
 * It runs in its own thread, with its own event loop for receiving input events and doing other
 * things.
 */
pub fn main() {
  a_info!("-------------------------------------------------------------------");

  // TODO: Implement restoring / saving state in android-rust-glue.
  let mut engine = engine::Engine {
    animating: false,
    egl_context: None,
    state: Default::default(),
    mvp_matrix: Default::default(),
    position: Default::default(),
    texture_unit: Default::default(),
    texture_coord: Default::default(),
    view_projection_matrix: Matrix4::identity(),
    texture: Default::default(),
  };

  let (event_tx, event_rx) = mpsc::channel::<Event>();
  android_glue::add_sender_missing(event_tx);
  'event: loop {
    match event_rx.try_recv() {
      Ok(ev) => {
        println!("----- Event: {:?}", ev);
        match ev {
          Event::InitWindow => init_display(&mut engine),
          Event::GainedFocus => engine.gained_focus(),
          Event::Pause => engine.lost_focus(),
          Event::TermWindow => engine.term(),
          _ => (),
        }
      },
      Err(TryRecvError::Empty) => (),
      Err(TryRecvError::Disconnected) => {
        a_panic!("----- Failed to get next event, channel disconnected")
      },
    }
    engine.update_draw();

    let app = android_glue::get_app();
    if app.destroyRequested != 0 {
      engine.term();  // Double-termination is fine.
      break 'event;
    }
  }
}
