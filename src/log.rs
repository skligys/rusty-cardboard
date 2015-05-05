// tomaka/android-rs-glue uses io redirection to redirect stdout into Android logs.  Yet, recently
// it started printing each argument as a new log item.  This is an ugly workaround: on Android only,
// replace log! with print!(format!()); on linux with prinln!().

#[macro_escape]
#[cfg(target_os = "android")]
macro_rules! log {
  ($fmt:expr) => (println!($fmt));
  ($fmt:expr, $($arg:tt)*) => (print!("{}", format!($fmt, $($arg)*)));
}

#[macro_escape]
#[cfg(target_os = "linux")]
macro_rules! log {
  ($fmt:expr) => (println!($fmt));
  ($fmt:expr, $($arg:tt)*) => (println!($fmt, $($arg)*));
}
