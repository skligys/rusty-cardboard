use libc::{c_float, c_int, c_void, int8_t, int32_t, int64_t, size_t, ssize_t, uint8_t};
use std::default::Default;
use std::mem;
use std::ptr;

use log;

// Opaque structure.
pub struct Manager;
// Opaque structure.
pub struct Sensor;
// Opaque structure.
pub struct EventQueue;

// C structure contains unions not representable in Rust, so this is just the
// version as it applies to accelerometer.
struct Vector {
  #[allow(dead_code)]
  x: c_float,
  #[allow(dead_code)]
  y: c_float,
  #[allow(dead_code)]
  z: c_float,
  #[allow(dead_code)]
  status: int8_t,
  #[allow(dead_code)]
  reserved: [uint8_t, ..3]
}

impl Default for Vector {
  fn default() -> Vector {
    Vector { x: 0.0, y: 0.0, z: 0.0, status: 0, reserved: [0, 0, 0] }
  }
}

// C structure contains unions not representable in Rust, so this is just the
// version as it applies to accelerometer.
pub struct Event {
  #[allow(dead_code)]
  version: int32_t,  /* size_of(Event) */
  #[allow(dead_code)]
  sensor: int32_t,
  #[allow(dead_code)]
  event_type: int32_t,
  #[allow(dead_code)]
  reserved0: int32_t,
  #[allow(dead_code)]
  timestamp: int64_t,
  #[allow(dead_code)]
  acceleration: Vector,
  #[allow(dead_code)]
  reserved1: [int32_t, ..4]
}

impl Default for Event {
  fn default() -> Event {
    Event {
      version: mem::size_of::<Event>() as int32_t,
      sensor: 0,
      event_type: 0,
      reserved0: 0,
      timestamp: 0,
      acceleration: Default::default(),
      reserved1: [0, 0, 0, 0],
    }
  }
}

// Looper id enums:
#[allow(dead_code)]
pub static LOOPER_ID_MAIN: c_int = 1;
#[allow(dead_code)]
pub static LOOPER_ID_INPUT: c_int = 2;
pub static LOOPER_ID_USER: c_int = 3;

/**
 * A looper is the state tracking an event loop for a thread.  Loopers do not define event
 * structures or other such things; rather they are a lower-level facility to attach one or more
 * discrete objects listening for an event.  An "event" here is simply data available on a file
 * descriptor: each attached object has an associated file descriptor, and waiting for "events"
 * means (internally) polling on all of these file descriptors until one or more of them have data
 * available.
 *
 * A thread can have only one ALooper associated with it.
*/
pub struct ALooper;

/**
 * For callback-based event loops, this is the prototype of the function that is called when a file
 * descriptor event occurs.  It is given the file descriptor it is associated with, a bitmask
 * of the poll events that were triggered (typically ALOOPER_EVENT_INPUT), and the data pointer
 * that was originally supplied.
 *
 * Implementations should return 1 to continue receiving callbacks, or 0 to have this file
 * descriptor and callback unregistered from the looper.
 */
// This is the right way but could not make passing null pointers work, neither with 0 as ...,
// nor with None::<..>.
// type LooperCallback = extern "C" fn (fd: c_int, events: c_int, data: *const c_void) -> c_int;
#[allow(dead_code)]
type LooperCallback = *const c_void;

// Sensor type enums:
pub static TYPE_ACCELEROMETER: c_int = 1;
#[allow(dead_code)]
pub static TYPE_MAGNETIC_FIELD: c_int = 2;

/// Get an unsafe pointer to the sensor manager.  Manager is a singleton.
pub fn get_instance() -> *const Manager {
  unsafe {
    ASensorManager_getInstance()
  }
}

/// Returns the default sensor for the given type, or null if no sensor of that type exist.
pub fn get_default_sensor(sensor_type: c_int) -> *const Sensor {
  let manager = get_instance();
  unsafe {
    ASensorManager_getDefaultSensor(manager, sensor_type)
  }
}

/// Creates a new sensor event queue and associates it with a looper.
pub fn create_event_queue(looper: *const ALooper, ident: c_int) -> *mut EventQueue {
  let manager = get_instance();
  unsafe {
    ASensorManager_createEventQueue(manager, looper, ident, ptr::null(), ptr::null())
  }
}

/**
 * Creates a new sensor event queue and associates it with a looper.  This is a version with
 * event callback.
 */
#[allow(dead_code)]
pub fn create_event_queue_with_callback(looper: *const ALooper, ident: c_int,
  callback: LooperCallback, data: *const c_void) -> *mut EventQueue {
  let manager = get_instance();
  unsafe {
    ASensorManager_createEventQueue(manager, looper, ident, callback, data)
  }
}

/// Enable the selected sensor. Returns a negative error code on failure.
pub fn enable_sensor(queue: *mut EventQueue, sensor: *const Sensor) -> Result<(), i32> {
  let res = unsafe {
    ASensorEventQueue_enableSensor(queue, sensor)
  };
  if res >= 0 { Ok(()) } else { Err(res) }
}

/// Disable the selected sensor. Returns a negative error code on failure.
pub fn disable_sensor(queue: *mut EventQueue, sensor: *const Sensor) -> Result<(), i32> {
  let res = unsafe {
    ASensorEventQueue_disableSensor(queue, sensor)
  };
  if res >= 0 { Ok(()) } else { Err(res) }
}

/**
 * Sets the delivery rate of events in microseconds for the given sensor.  Note that this is
 * a hint only, generally events will arrive at a higher rate. It is an error to set a rate below
 * the value returned by ASensor_getMinDelay().  Returns a negative error code on failure.
 */
pub fn set_event_rate(queue: *mut EventQueue, sensor: *const Sensor, usec: int32_t) -> Result<(), i32> {
  let res = unsafe {
    ASensorEventQueue_setEventRate(queue, sensor, usec)
  };
  if res >= 0 { Ok(()) } else { Err(res) }
}

/*
 * Returns the next available event from the queue.  Returns a zero error value if no events are
 * available and a negative error value when an error has occurred.
*/
pub fn get_event(queue: *mut EventQueue) -> Result<Event, c_int> {
  let mut event: Event = Default::default();
  let res = unsafe {
    ASensorEventQueue_getEvents(queue, &mut event as *mut Event, 1)
  };
  match res {
    1 => Ok(event),
    err if err <= 0 => Err(err),
    n => {
      log::e_f(format!("ASensorEventQueue_getEvents returned a positive result but not 1: {}", n));
      fail!();
    },
  }
}

// Lopper poll result enums:
/**
 * The poll was awoken using wake() before the timeout expired and no callbacks were executed and
 * no other file descriptors were ready.
 */
static ALOOPER_POLL_WAKE: c_int = -1;
/// One or more callbacks were executed.
#[allow(dead_code)]
static ALOOPER_POLL_CALLBACK: c_int = -2;
/// The timeout expired.
static ALOOPER_POLL_TIMEOUT: c_int = -3;
/// An error occurred.
static ALOOPER_POLL_ERROR: c_int = -4;

struct PollResult {
  pub id: c_int,
  pub fd: c_int,
  pub events: c_int,
  pub data: *const c_void,
}

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
 * Returns ALOOPER_POLL_WAKE if the poll was awoken using wake() before the timeout expired and
 * no callbacks were invoked and no other file descriptors were ready.
 *
 * Never returns ALOOPER_POLL_CALLBACK.
 *
 * Returns ALOOPER_POLL_TIMEOUT if there was no data before the given timeout expired.
 *
 * Returns ALOOPER_POLL_ERROR if an error occurred.
 *
 * Returns a value >= 0 containing an identifier if its file descriptor has data and it has
 * no callback function (requiring the caller here to handle it).  In this (and only this) case
 * out_fd, out_events and out_data will contain the poll events and data associated with the fd,
 * otherwise they will be set to NULL.
 *
 * This method does not return until it has finished invoking the appropriate callbacks for all
 * file descriptors that were signalled.
 */
pub fn poll_all(timeout_millis: c_int) -> Result<PollResult, PollError> {
  let mut fd: c_int = 0;
  let mut events: c_int = 0;
  let mut data: *const c_void = ptr::null();
  let res = unsafe {
    ALooper_pollAll(timeout_millis, &mut fd as *mut c_int, &mut events as *mut c_int,
      &mut data as *mut *const c_void)
  };
  match res {
    ALOOPER_POLL_WAKE => Err(PollWake),
    ALOOPER_POLL_TIMEOUT => Err(PollTimeout),
    ALOOPER_POLL_ERROR => Err(PollError),
    id if id >= 0 => Ok(PollResult { id: id, fd: fd, events: events, data: data }),
    err => fail!("Unknown error from ALooper_pollAll(): {}", err),
  }
}

extern {
  fn ASensorManager_getInstance() -> *const Manager;
  fn ASensorManager_getDefaultSensor(manager: *const Manager, sensor_type: c_int) -> *const Sensor;
  fn ASensorManager_createEventQueue(manager: *const Manager, looper: *const ALooper, ident: c_int,
    callback: LooperCallback, data: *const c_void) -> *mut EventQueue;
  fn ASensorEventQueue_enableSensor(queue: *mut EventQueue, sensor: *const Sensor) -> c_int;
  fn ASensorEventQueue_disableSensor(queue: *mut EventQueue, sensor: *const Sensor) -> c_int;
  fn ASensorEventQueue_setEventRate(queue: *mut EventQueue, sensor: *const Sensor, usec: int32_t) -> c_int;
  fn ASensorEventQueue_getEvents(queue: *mut EventQueue, events: *mut Event, count: size_t) -> ssize_t;

  fn ALooper_pollAll(timeout_millis: c_int, out_fd: *mut c_int, out_events: *mut c_int, out_data: *mut *const c_void) -> c_int;
}
