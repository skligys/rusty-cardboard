extern crate android_glue;

use libc::{c_int, c_void, int32_t};
use std::mem;
use std::ptr;

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

/// Get an unsafe pointer to the sensor manager.  Manager is a singleton.
pub fn get_instance() -> &'static mut android_glue::ffi::ASensorManager {
  let manager_ptr = unsafe {
    android_glue::ffi::ASensorManager_getInstance()
  };
  assert!(!manager_ptr.is_null());
  unsafe { &mut *manager_ptr }
}

/// Returns the default sensor for the given type, or None if no sensor of that type exist.
pub fn get_default_sensor(sensor_type: i32) -> Option<&'static android_glue::ffi::ASensor> {
  let manager = get_instance();
  let sensor_ptr = unsafe {
    android_glue::ffi::ASensorManager_getDefaultSensor(manager, sensor_type)
  };
  if sensor_ptr.is_null() {
    None
  } else {
    Some(unsafe { &*sensor_ptr })
  }
}

/// Creates a new sensor event queue and associates it with a looper.
pub fn create_event_queue(looper: *mut android_glue::ffi::ALooper, ident: i32) ->
  &'static mut android_glue::ffi::ASensorEventQueue {

  let manager = get_instance();
  let queue_ptr = unsafe {
    android_glue::ffi::ASensorManager_createEventQueue(manager, looper, ident, None, ptr::null_mut())
  };
  assert!(!queue_ptr.is_null());
  unsafe { &mut *queue_ptr }
}

/**
 * Creates a new sensor event queue and associates it with a looper.  This is a version with
 * event callback.
 */
#[allow(dead_code)]
pub fn create_event_queue_with_callback(looper: *mut android_glue::ffi::ALooper, ident: i32,
  callback: android_glue::ffi::ALooper_callbackFunc, data: *mut c_void) ->
  &'static mut android_glue::ffi::ASensorEventQueue {

  let manager = get_instance();
  let queue_ptr = unsafe {
    android_glue::ffi::ASensorManager_createEventQueue(manager, looper, ident, Some(callback), data)
  };
  assert!(!queue_ptr.is_null());
  unsafe { &mut *queue_ptr }
}

/// Enable the selected sensor. Returns a negative error code on failure.
pub fn enable_sensor(queue: &mut android_glue::ffi::ASensorEventQueue, sensor: &android_glue::ffi::ASensor) ->
  Result<(), i32> {

  let res = unsafe {
    android_glue::ffi::ASensorEventQueue_enableSensor(queue, sensor)
  };
  if res >= 0 { Ok(()) } else { Err(res) }
}

/// Disable the selected sensor. Returns a negative error code on failure.
pub fn disable_sensor(queue: &mut android_glue::ffi::ASensorEventQueue, sensor: &android_glue::ffi::ASensor) ->
  Result<(), i32> {

  let res = unsafe {
    android_glue::ffi::ASensorEventQueue_disableSensor(queue, sensor)
  };
  if res >= 0 { Ok(()) } else { Err(res) }
}

/**
 * Sets the delivery rate of events in microseconds for the given sensor.  Note that this is
 * a hint only, generally events will arrive at a higher rate. It is an error to set a rate below
 * the value returned by ASensor_getMinDelay().  Returns a negative error code on failure.
 */
pub fn set_event_rate(queue: &mut android_glue::ffi::ASensorEventQueue,
  sensor: &android_glue::ffi::ASensor, usec: i32) -> Result<(), i32> {

  let res = unsafe {
    android_glue::ffi::ASensorEventQueue_setEventRate(queue, sensor, usec)
  };
  if res >= 0 { Ok(()) } else { Err(res) }
}

/*
 * Returns the next available event from the queue.  Returns a zero error value if no events are
 * available and a negative error value when an error has occurred.
*/
pub fn get_event(queue: &mut android_glue::ffi::ASensorEventQueue) -> Result<android_glue::ffi::ASensorEvent, c_int> {
  let mut event: android_glue::ffi::ASensorEvent = android_glue::ffi::ASensorEvent {
    version: mem::size_of::<android_glue::ffi::ASensorEvent>() as int32_t,
    sensor: 0,
    xtype: 0,
    reserved0: 0,
    timestamp: 0,
    data: [0.0; 16],
    flags: 0,
    reserved1: [0; 2],
  };
  let res = unsafe {
    android_glue::ffi::ASensorEventQueue_getEvents(queue, &mut event as *mut android_glue::ffi::ASensorEvent, 1)
  };
  match res {
    1 => Ok(event),
    err if err <= 0 => Err(err),
    n => a_fail!("ASensorEventQueue_getEvents returned a positive result but not 1: {}", n),
  }
}

struct PollResult {
  pub id: c_int,
  pub fd: c_int,
  pub events: c_int,
  pub data: *const c_void,
}

#[allow(dead_code)]
enum PollError {
  PollWake,
  PollCallback,
  PollTimeout,
  PollError,
}

/**
 * Waits for events to be available, with optional timeout in milliseconds.  Invokes callbacks for
 * all file descriptors on which an event occurred.  Performs all pending callbacks until all
 * data has been consumed or a file descriptor is available with no callback.
 *
 * If the timeout is zero, returns immediately without blocking.  If the timeout is negative, waits
 * indefinitely until an event appears.
 *
 * Returns PollWake if the poll was awoken using wake() before the timeout expired and
 * no callbacks were invoked and no other file descriptors were ready.
 *
 * Never returns PollCallback.
 *
 * Returns PollTimeout if there was no data before the given timeout expired.
 *
 * Returns PollError if an error occurred.
 *
 * Returns a value >= 0 containing an identifier if its file descriptor has data and it has
 * no callback function (requiring the caller here to handle it).  In this (and only this) case
 * out_fd, out_events and out_data will contain the poll events and data associated with the fd,
 * otherwise they will be set to NULL.
 *
 * This method does not return until it has finished invoking the appropriate callbacks for all
 * file descriptors that were signalled.
 */
pub fn poll_all(timeout_millis: i32) -> Result<PollResult, PollError> {
  let mut fd: c_int = 0;
  let mut events: c_int = 0;
  let mut data: *mut c_void = ptr::null_mut();
  let res = unsafe {
    android_glue::ffi::ALooper_pollAll(timeout_millis, &mut fd as *mut c_int, &mut events as *mut c_int,
      &mut data as *mut *mut c_void)
  };
  match res {
    android_glue::ffi::ALOOPER_POLL_WAKE => Err(PollError::PollWake),
    android_glue::ffi::ALOOPER_POLL_TIMEOUT => Err(PollError::PollTimeout),
    android_glue::ffi::ALOOPER_POLL_ERROR => Err(PollError::PollError),
    id if id >= 0 => Ok(PollResult { id: id, fd: fd, events: events, data: data }),
    err => a_fail!("Unknown error from ALooper_pollAll(): {}", err),
  }
}
