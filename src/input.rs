use libc::{c_float, int32_t, size_t};

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

/// Input event is an opaque structure.
pub struct Event;
/// Input queue is for retrieving input events.
pub struct Queue;

// Input event types:
const EVENT_TYPE_KEY: int32_t = 1;
const EVENT_TYPE_MOTION: int32_t = 2;
pub enum EventType {
  Key,
  Motion,
}

/// Get the input event type.
pub fn get_event_type(event: *const Event) -> EventType {
  let res = unsafe {
    AInputEvent_getType(event)
  };
  match res {
    EVENT_TYPE_KEY => EventType::Key,
    EVENT_TYPE_MOTION => EventType::Motion,
    _ => a_fail!("Unknown event type: {}", res),
  }
}

/** Get the current X coordinate of this event for the given pointer index.
 * Whole numbers are pixels; the value may have a fraction for input devices
 * that are sub-pixel precise. */
pub fn get_motion_event_x(event: *const Event, pointer_index: u32) -> f32 {
  unsafe {
    AMotionEvent_getX(event, pointer_index)
  }
}

/* Get the current Y coordinate of this event for the given pointer index.
 * Whole numbers are pixels; the value may have a fraction for input devices
 * that are sub-pixel precise. */
pub fn get_motion_event_y(event: *const Event, pointer_index: u32) -> f32 {
  unsafe {
    AMotionEvent_getY(event, pointer_index)
  }
}

extern {
 fn AInputEvent_getType(event: *const Event) -> int32_t;
 fn AMotionEvent_getX(event: *const Event, pointer_index: size_t) -> c_float;
 fn AMotionEvent_getY(event: *const Event, pointer_index: size_t) -> c_float;
}
