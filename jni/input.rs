use libc::{c_float, int32_t, size_t};

/// Input event is an opaque structure.
pub struct Event;
/// Input queue is for retrieving input events.
pub struct Queue;

// Input event types:
static EVENT_TYPE_KEY: int32_t = 1;
static EVENT_TYPE_MOTION: int32_t = 2;
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
    EVENT_TYPE_KEY => Key,
    EVENT_TYPE_MOTION => Motion,
    _ => fail!("Unknown event type: {}", res),
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
