#![feature(start, slice_patterns)]

#[macro_use]
#[cfg(target_os = "android")]
extern crate android_glue;

#[macro_use]
extern crate lazy_static;

extern crate cgmath;
extern crate collision;
extern crate libc;
extern crate noise;
#[cfg(target_os = "linux")]
extern crate png;
extern crate time;

#[cfg(target_os = "android")]
use std::sync::mpsc;
#[cfg(target_os = "android")]
use std::sync::mpsc::{Receiver, TryRecvError};

#[cfg(target_os = "android")]
use android_glue::{AssetError, Event};
#[cfg(target_os = "android")]
use egl_context::EglContext;
use engine::Engine;
#[cfg(target_os = "linux")]
use program::Program;
#[cfg(target_os = "linux")]
use x11::{Event, XWindow};

#[macro_use]
mod log;

#[cfg(target_os = "android")]
mod egl;
#[cfg(target_os = "android")]
mod egl_context;
mod engine;
mod fov;
mod gl;
mod mesh;
mod perlin;
mod program;
mod world;
#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "android")]
android_start!(main);

/**
 * This is the main entry point of a native application that is using android_native_app_glue.
 * It runs in its own thread, with its own event loop for receiving input events and doing other
 * things.
 */
#[cfg(target_os = "android")]
pub fn main() {
  log!("-------------------------------------------------------------------");

  // TODO: Implement restoring / saving state in android-rust-glue.
  let mut engine = Engine::new();

  let (event_tx, event_rx) = mpsc::channel::<Event>();
  android_glue::add_sender_missing(event_tx);
  'event: loop {
    handle_event(&event_rx, &mut engine);
    engine.update_draw();

    let app = android_glue::get_app();
    if app.destroyRequested != 0 {
      engine.term();  // Double-termination is fine.
      break 'event;
    }
  }
}

/// Handle a single event if present.
#[cfg(target_os = "android")]
fn handle_event(rx: &Receiver<Event>, engine: &mut Engine) {
  match rx.try_recv() {
    Ok(ev) => {
      log!("----- Event: {:?}", ev);
      match ev {
        Event::InitWindow => init_display(engine),
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
}

/// Initialize EGL context for the current display.
#[cfg(target_os = "android")]
fn init_display(engine: &mut Engine) {
  log!("*** Renderer initializing...");
  let start_ns = time::precise_time_ns();
  let app = android_glue::get_app();
  let window = app.window as egl::NativeWindowType;
  let egl_context = Box::new(EglContext::new(window));
  let texture_atlas_bytes = load_asset("atlas.png");
  engine.init(egl_context, &texture_atlas_bytes);
  let elapsed_ms = (time::precise_time_ns() - start_ns) as f32 / 1000000.0;
  log!("*** Renderer initialized, {:.3}ms", elapsed_ms);
}

#[cfg(target_os = "android")]
fn load_asset(filename: &str) -> Vec<u8> {
  match android_glue::load_asset(filename) {
    Ok(v) => v,
    Err(e) => {
      let mess = match e {
        AssetError::AssetMissing => "asset missing",
        AssetError::EmptyBuffer => "couldn't read asset",
      };
      panic!("Loading atlas.png failed: {}", mess)
    },
  }
}

#[cfg(target_os = "linux")]
pub fn main() {
  log!("-------------------------------------------------------------------");
  let window = XWindow::new("Rusty Cardboard");

  // Compile and link vertex and fragment shaders.
  let program = match Program::new() {
    Ok(p) => p,
    Err(e) => panic!("Program failed: {:?}", e),
  };

  let mut engine = Engine::new(window, program);
  engine.init(TEXTURE_ATLAS);

  while !engine.is_closed() {
    engine.update_draw();
    handle_events(&mut engine);
   }
}

#[cfg(target_os = "linux")]
static TEXTURE_ATLAS: &'static [u8] = include_bytes!("../assets/atlas.png");

/// Process X11 events while there are any queued.
#[cfg(target_os = "linux")]
fn handle_events(engine: &mut Engine) {
 enum FocusChange {
    Gained,
    Lost,
  }

  let mut focus_change: Option<FocusChange> = None;
  let mut resized_to: Option<(u32, u32)> = None;
  for e in engine.poll_events() {
    match e {
      Event::MouseMoved(_) => (),  // too much spam
      Event::Resized(w, h) => {
        log!("{:?}", Event::Resized(w, h));
        resized_to = Some((w, h));
      },
      Event::Focused(f) => {
        log!("{:?}", Event::Focused(f));
        if f {
          focus_change = Some(FocusChange::Gained);
        } else {
          focus_change = Some(FocusChange::Lost);
        }
      },
      e => log!("{:?}", e),
    }
  }

  match focus_change {
    Some(FocusChange::Gained) => engine.gained_focus(),
    Some(FocusChange::Lost) => engine.lost_focus(),
    None => (),
  }
  if let Some((w, h)) = resized_to {
    engine.set_viewport(w as i32, h as i32);
  }
}
