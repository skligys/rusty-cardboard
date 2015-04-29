extern crate png;

use std::default::Default;
use std::ptr;

use android_glue;
use egl;
use egl::{Context, Display, Surface};

// RAII managed EGL pointers.  Cleaned up automatically via Drop.
pub struct EglContext {
  display: Display,
  surface: Surface,
  context: Context,
  pub width: i32,
  pub height: i32,
}

impl Default for EglContext {
  fn default() -> EglContext {
    EglContext {
      display: egl::NO_DISPLAY,
      surface: egl::NO_SURFACE,
      context: egl::NO_CONTEXT,
      width: 0,
      height: 0,
    }
  }
}

impl EglContext {
  pub fn new(window: *mut android_glue::ffi::ANativeWindow) -> EglContext {
    let display = egl::get_display(egl::DEFAULT_DISPLAY);
    if let Err(e) = egl::initialize(display) {
      panic!("Failed in egl::initialize(): {:?}", e);
    }

    // Specify attributes of the desired configuration.  Select an EGLConfig with at least 8 bits
    // per color component compatible with OpenGL ES 2.0.  A very simplified selection process,
    // just pick the first EGLConfig that matches our criteria.
    let config = {
      let attribs_config = [
        egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
        egl::RED_SIZE, 8,
        egl::GREEN_SIZE, 8,
        egl::BLUE_SIZE, 8,
        egl::DEPTH_SIZE, 24,
        egl::NONE,
      ];
      let mut configs = vec!(ptr::null());
      if let Err(e) = egl::choose_config(display, &attribs_config, &mut configs) {
        panic!("Failed in egl::choose_config(): {:?}", e);
      }
      if configs.len() == 0 {
        panic!("egl::choose_config() did not find any configurations");
      }
      configs[0]
    };

    // EGL_NATIVE_VISUAL_ID is an attribute of the EGLConfig that is guaranteed to be accepted by
    // ANativeWindow_setBuffersGeometry().  As soon as we picked a EGLConfig, we can safely
    // reconfigure the NativeWindow buffers to match, using EGL_NATIVE_VISUAL_ID.
    let format = match egl::get_config_attrib(display, config, egl::NATIVE_VISUAL_ID) {
      Ok(f) => f,
      Err(e) => panic!("egl::get_config_attrib(NATIVE_VISUAL_ID) failed: {:?}", e),
    };

    unsafe {
      android_glue::ffi::ANativeWindow_setBuffersGeometry(window, 0, 0, format);
    }

    let surface = match egl::create_window_surface(display, config, window) {
      Ok(s) => s,
      Err(e) => panic!("egl::create_window_surface() failed: {:?}", e),
    };

    let context = {
      let attribs_context = [
        egl::CONTEXT_CLIENT_VERSION, 2,
        egl::NONE
      ];
      match egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, &attribs_context) {
        Ok(c) => c,
        Err(e) => panic!("egl::create_context_with_attribs() failed: {:?}", e),
      }
    };

    if let Err(e) = egl::make_current(display, surface, surface, context) {
      panic!("Failed in egl::make_current(): {:?}", e);
    }

    let w = match egl::query_surface(display, surface, egl::WIDTH) {
      Ok(w) => w,
      Err(e) => panic!("egl::query_surface(WIDTH) failed: {:?}", e),
    };
    let h = match egl::query_surface(display, surface, egl::HEIGHT) {
      Ok(w) => w,
      Err(e) => panic!("egl::query_surface(HEIGHT) failed: {:?}", e),
    };

    EglContext {
      display: display,
      surface: surface,
      context: context,
      width: w,
      height: h,
    }
  }

  pub fn swap_buffers(&self) {
    let _ = egl::swap_buffers(self.display, self.surface);
  }
}

impl Drop for EglContext {
  fn drop(&mut self) {
    if self.display != egl::NO_DISPLAY {
      let _ = egl::make_current(self.display, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT);
      if self.context != egl::NO_CONTEXT {
        let _ = egl::destroy_context(self.display, self.context);
        self.context = egl::NO_CONTEXT;
      }
      if self.surface != egl::NO_SURFACE {
        let _ = egl::destroy_surface(self.display, self.surface);
        self.surface = egl::NO_SURFACE;
      }
      let _ = egl::terminate(self.display);
      self.display = egl::NO_DISPLAY;
    }
  }
}
