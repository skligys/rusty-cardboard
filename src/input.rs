extern crate android_glue;

use log;

// TODO: Figure out how to put macros in a separate module and import when needed.

/// Logs the error to Android error logging and fails.
macro_rules! a_fail(
  ($msg: expr) => ({
    log::e($msg);
    panic!();
  });
  ($fmt: expr, $($arg:tt)*) => ({
    log::e_f(format!($fmt, $($arg)*));
    panic!();
  });
);

pub enum EventType {
  Key,
  Motion,
}

/// Get the input event type.
pub fn get_event_type(event: *const android_glue::ffi::AInputEvent) -> EventType {
  let res = unsafe {
    android_glue::ffi::AInputEvent_getType(event)
  };
  match res {
    android_glue::ffi::AINPUT_EVENT_TYPE_KEY => EventType::Key,
    android_glue::ffi::AINPUT_EVENT_TYPE_MOTION => EventType::Motion,
    _ => a_fail!("Unknown event type: {}", res),
  }
}

/** Get the current X coordinate of this event for the given pointer index.
 * Whole numbers are pixels; the value may have a fraction for input devices
 * that are sub-pixel precise. */
pub fn get_motion_event_x(event: *const android_glue::ffi::AInputEvent, pointer_index: u32) -> f32 {
  unsafe {
    android_glue::ffi::AMotionEvent_getX(event, pointer_index)
  }
}

/* Get the current Y coordinate of this event for the given pointer index.
 * Whole numbers are pixels; the value may have a fraction for input devices
 * that are sub-pixel precise. */
pub fn get_motion_event_y(event: *const android_glue::ffi::AInputEvent, pointer_index: u32) -> f32 {
  unsafe {
    android_glue::ffi::AMotionEvent_getY(event, pointer_index)
  }
}
