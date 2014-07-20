use libc::{c_int, int32_t};

// Opaque structure.
pub struct Manager;
// Opaque structure.
pub struct Sensor;
// Opaque structure.
pub struct EventQueue;

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

extern {
  fn ASensorEventQueue_enableSensor(queue: *mut EventQueue, sensor: *const Sensor) -> c_int;
  fn ASensorEventQueue_disableSensor(queue: *mut EventQueue, sensor: *const Sensor) -> c_int;
  fn ASensorEventQueue_setEventRate(queue: *mut EventQueue, sensor: *const Sensor, usec: int32_t) -> c_int;
}
