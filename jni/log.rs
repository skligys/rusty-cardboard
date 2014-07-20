use libc::{c_char, c_int};

// Logging priorities:
static VERBOSE: c_int = 2;
static DEBUG: c_int = 3;
static INFO: c_int = 4;
static WARN: c_int = 5;
static ERROR: c_int = 6;
static FATAL: c_int = 7;

// Bridges to Android logging at various priorities.
pub fn v(msg: &str) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(VERBOSE, c_string.as_ptr());
  }
}

pub fn v_f(msg: String) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(VERBOSE, c_string.as_ptr());
  }
}

pub fn d(msg: &str) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(DEBUG, c_string.as_ptr());
  }
}

pub fn d_f(msg: String) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(DEBUG, c_string.as_ptr());
  }
}

pub fn i(msg: &str) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(INFO, c_string.as_ptr());
  }
}

pub fn i_f(msg: String) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(INFO, c_string.as_ptr());
  }
}

pub fn w(msg: &str) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(WARN, c_string.as_ptr());
  }
}

pub fn w_f(msg: String) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(WARN, c_string.as_ptr());
  }
}

pub fn e(msg: &str) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(ERROR, c_string.as_ptr());
  }
}

pub fn e_f(msg: String) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(ERROR, c_string.as_ptr());
  }
}

pub fn wtf(msg: &str) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(FATAL, c_string.as_ptr());
  }
}

pub fn wtf_f(msg: String) {
  let c_string = msg.to_c_str();
  unsafe {
    c_log_string(FATAL, c_string.as_ptr());
  }
}

extern {
  fn c_log_string(priority: c_int, message: *const c_char);
}
