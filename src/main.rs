#![feature(collections, start, std_misc, unsafe_destructor)]

#[macro_use]
#[cfg(target_os = "android")]
extern crate android_glue;

#[macro_use]
#[cfg(not(target_os = "android"))]
extern crate lazy_static;

extern crate cgmath;
extern crate libc;
extern crate time;

#[cfg(target_os = "android")]
use std::default::Default;
#[cfg(target_os = "android")]
use std::sync::mpsc;
#[cfg(target_os = "android")]
use std::sync::mpsc::TryRecvError;

#[cfg(target_os = "android")]
use android_glue::Event;
#[cfg(target_os = "android")]
use cgmath::Matrix4;
#[cfg(target_os = "android")]
use engine::{EglContext, Engine};
#[cfg(target_os = "linux")]
use x11::{Event, XWindow};

#[cfg(target_os = "android")]
mod egl;
#[cfg(target_os = "android")]
mod engine;
mod gl;
#[cfg(target_os = "android")]
mod mesh;
#[cfg(target_os = "android")]
mod program;
#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "android")]
android_start!(main);

/// Initialize EGL context for the current display.
#[cfg(target_os = "android")]
fn init_display(engine: &mut Engine) {
  println!("Renderer initializing...");
  let start_ns = time::precise_time_ns();
  let app = android_glue::get_app();
  let window = app.window as *mut android_glue::ffi::ANativeWindow;
  let egl_context = Box::new(EglContext::new(window));
  engine.init(egl_context);
  let elapsed_ms = (time::precise_time_ns() - start_ns) as f32 / 1000000.0;
  println!("Renderer initialized, {:.3}ms", elapsed_ms);
}

/**
 * This is the main entry point of a native application that is using android_native_app_glue.
 * It runs in its own thread, with its own event loop for receiving input events and doing other
 * things.
 */
#[cfg(target_os = "android")]
pub fn main() {
  println!("-------------------------------------------------------------------");

  // TODO: Implement restoring / saving state in android-rust-glue.
  let mut engine = Engine {
    animating: false,
    egl_context: None,
    state: Default::default(),
    program: None,
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
        panic!("----- Failed to get next event, channel disconnected")
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

#[cfg(target_os = "linux")]
pub fn main() {
  println!("-------------------------------------------------------------------");
  let window = XWindow::new("Rusty Cardboard");
  window.make_current();

  while !window.is_closed() {
    // Set the background clear color to sky blue.
    gl::clear_color(0.5, 0.69, 1.0, 1.0);

    // Enable reverse face culling and depth test.
    gl::enable(gl::CULL_FACE);
    gl::enable(gl::DEPTH_TEST);
    gl::clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
    window.flush();

    window.swap_buffers();

    for e in window.poll_events() {
      match e {
        Event::MouseMoved(_) => (),  // too much spam
        e => println!("{:?}", e),
      }
    }
  }
}
