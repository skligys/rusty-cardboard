// Opaque struct for Android native window.
pub struct ANativeWindow;

pub fn set_buffers_geometry(window: *const ANativeWindow, width: i32, height: i32, format: i32) -> i32 {
  unsafe {
    ANativeWindow_setBuffersGeometry(window, width, height, format)
  }
}

extern {
  fn ANativeWindow_setBuffersGeometry(window: *const ANativeWindow, width: i32, height: i32, format: i32) -> i32;
}
