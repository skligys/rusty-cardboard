extern crate libc;
use libc::{c_float, c_int, c_void, size_t};
use native_window::ANativeWindow;

mod egl;
mod gl;
mod native_window;
mod log;

// TODO: implement.
struct ANativeActivity {
  dummy: *const c_void,
}
// TODO: implement.
struct AConfiguration {
  dummy: *const c_void,
}
// TODO: implement.
struct ALooper {
  dummy: *const c_void,
}
// TODO: implement.
struct AInputQueue {
  dummy: *const c_void,
}

struct ARect {
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
}

// This is the interface for the standard glue code of a threaded application.  In this model, the
// application's code is running in its own thread separate from the main thread of the process.
// It is not required that this thread be associated with the Java VM, although it will need to be
// in order to make JNI calls to any Java objects.  Compatible with C.
struct AndroidApp {
  // The application can place a pointer to its own state object here if it likes.
  user_data: *const c_void,
  // Fill this in with the function to process main app commands (APP_CMD_*)
  // TODO: implement.
  on_app_cmd: *const c_void,
  // Fill this in with the function to process input events.  At this point the event has already
  // been pre-dispatched, and it will be finished upon return.  Return 1 if you have handled
  // the event, 0 for any default dispatching.
  on_input_event: *const c_void,
  // The ANativeActivity object instance that this app is running in.
  activity: *const ANativeActivity,
  // The current configuration the app is running in.
  config: *const AConfiguration,
  // This is the last instance's saved state, as provided at creation time.  It is NULL if there
  // was no state.  You can use this as you need; the memory will remain around until you call
  // android_app_exec_cmd() for APP_CMD_RESUME, at which point it will be freed and savedState
  // set to NULL.  These variables should only be changed when processing a APP_CMD_SAVE_STATE,
  // at which point they will be initialized to NULL and you can malloc your state and place
  // the information here.  In that case the memory will be freed for you later.
  saved_state: *const c_void,
  saved_state_size: size_t,
  // The ALooper associated with the app's thread.
  looper: *const ALooper,
  // When non-NULL, this is the input queue from which the app will receive user input events.
  input_queue: *const AInputQueue,
  // When non-NULL, this is the window surface that the app can draw in.
  window: *const ANativeWindow,
  // Current content rectangle of the window; this is the area where the window's content should be
  // placed to be seen by the user.
  content_rect: ARect,
  // Current state of the app's activity.  May be either APP_CMD_START, APP_CMD_RESUME,
  // APP_CMD_PAUSE, or APP_CMD_STOP; see below.
  activity_state: c_int,
  // This is non-zero when the application's NativeActivity is being destroyed and waiting for
  // the app thread to complete.
  destroy_requested: c_int,
  // Plus some private implementation details.
}

// TODO: implement.
struct ASensorManager {
  dummy: *const c_void,
}
// TODO: implement.
struct ASensor {
  dummy: *const c_void,
}
// TODO: implement.
struct ASensorEventQueue {
  dummy: *const c_void,
}

// Saved state data.  Compatible with C.
struct SavedState {
  angle: c_float,
  x: i32,
  y: i32,
}

// Shared state for our app.  Compatible with C.
struct Engine {
  app: *const AndroidApp,

  sensor_manager: *const ASensorManager,
  accelerometer_sensor: *const ASensor,
  sensor_event_queue: *const ASensorEventQueue,

  animating: c_int,
  display: egl::Display,
  surface: egl::Surface,
  context: egl::Context,
  width: i32,
  height: i32,
  state: SavedState,
}

/// Initialize EGL context for the current display.
#[no_mangle]
pub extern fn init_display(engine: &mut Engine) -> c_int {
  let display = egl::get_display(egl::DEFAULT_DISPLAY);

  match egl::initialize(display) {
    Ok(_) => (),
    Err(e) => {
      log::e_f(format!("egl::initialize() failed: {}", e));
      fail!();
    },
  };

  // Here specify the attributes of the desired configuration.  Below, we select an EGLConfig with
  // at least 8 bits per color component compatible with OpenGL ES 2.0.  A very simplified
  // selection process, where we pick the first EGLConfig that matches our criteria.
  let attribs_config = [
    egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
    egl::BLUE_SIZE, 8,
    egl::GREEN_SIZE, 8,
    egl::RED_SIZE, 8,
    egl::NONE
  ];
  let mut configs = vec!(0 as egl::Config);
  match egl::choose_config(display, attribs_config, &mut configs) {
    Ok(_) => (),
    Err(e) => {
      log::e_f(format!("egl::choose_config() failed: {}", e));
      fail!();
    },
  }
  if configs.len() == 0 {
    log::e("choose_config() did not find any configurations");
    fail!();
  }
  let config = *configs.get(0);

  // EGL_NATIVE_VISUAL_ID is an attribute of the EGLConfig that is guaranteed to be accepted by
  // ANativeWindow_setBuffersGeometry().  As soon as we picked a EGLConfig, we can safely
  // reconfigure the ANativeWindow buffers to match, using EGL_NATIVE_VISUAL_ID.
  let format = match egl::get_config_attrib(display, config, egl::NATIVE_VISUAL_ID) {
    Ok(vid) => vid,
    Err(e) => {
      log::e_f(format!("egl::get_config_attrib() failed: {}", e));
      fail!();
    },
  };

  let window = unsafe { (*engine.app).window };
  native_window::set_buffers_geometry(window, 0, 0, format);

  let surface = match egl::create_window_surface(display, config, window) {
    Ok(srf) => srf,
    Err(e) => {
      log::e_f(format!("egl::create_window_surface() failed: {}", e));
      fail!();
    },
  };

  let attribs_context = [
    egl::CONTEXT_CLIENT_VERSION, 2,
    egl::NONE
  ];
  let context = match egl::create_context_with_attribs(display, config, egl::NO_CONTEXT, attribs_context) {
    Ok(ctx) => ctx,
    Err(e) => {
      log::e_f(format!("egl::create_context() failed: {}", e));
      fail!();
    },
  };

  match egl::make_current(display, surface, surface, context) {
    Ok(()) => (),
    Err(_) => {
      log::w("Unable to egl::make_current");
      return -1;
    }
  }

  let version = match gl::get_string(gl::VERSION) {
    Ok(s) => s,
    Err(e) => {
      log::e_f(format!("gl::get_string(gl::VERSION) failed: {}", e));
      fail!();
    },
  };
  let vendor = match gl::get_string(gl::VENDOR) {
    Ok(s) => s,
    Err(e) => {
      log::e_f(format!("gl::get_string(gl::VENDOR) failed: {}", e));
      fail!();
    },
  };
  let renderer = match gl::get_string(gl::RENDERER) {
    Ok(s) => s,
    Err(e) => {
      log::e_f(format!("gl::get_string(gl::RENDERER) failed: {}", e));
      fail!();
    },
  };
  let sl_version = match gl::get_string(gl::SHADING_LANGUAGE_VERSION) {
    Ok(s) => s,
    Err(e) => {
      log::e_f(format!("gl::get_string(gl::SHADING_LANGUAGE_VERSION) failed: {}", e));
      fail!();
    },
  };
  log::i_f(format!("OpenGL version: \"{}\", vendor: \"{}\", renderer: \"{}\", SL version: \"{}\"",
    version.as_str().unwrap(), vendor.as_str().unwrap(), renderer.as_str().unwrap(),
    sl_version.as_str().unwrap()));

  let extensions = match gl::get_string(gl::EXTENSIONS) {
    Ok(s) => s,
    Err(e) => {
      log::e_f(format!("gl::get_string(gl::EXTENSIONS) failed: {}", e));
      fail!();
    },
  };
  log::i_f(format!("OpenGL extensions: \"{}\"", extensions.as_str().unwrap()));

  let w = match egl::query_surface(display, surface, egl::WIDTH) {
    Ok(i) => i,
    Err(e) => {
      log::e_f(format!("egl::query_surface(egl::WIDTH) failed: {}", e));
      fail!();
    },
  };
  let h = match egl::query_surface(display, surface, egl::HEIGHT) {
    Ok(i) => i,
    Err(e) => {
      log::e_f(format!("egl::query_surface(egl::HEIGHT) failed: {}", e));
      fail!();
    },
  };

  engine.display = display;
  engine.context = context;
  engine.surface = surface;
  engine.width = w;
  engine.height = h;
  engine.state.angle = 0.0;

  match gl::enable(gl::CULL_FACE) {
    Ok(()) => (),
    Err(e) => {
      log::e_f(format!("gl::enable(gl::CULL_FACE) failed: {}", e));
      fail!();
    },
  };
  match gl::disable(gl::DEPTH_TEST) {
    Ok(()) => (),
    Err(e) => {
      log::e_f(format!("gl::disable(gl::DEPTH_TEST) failed: {}", e));
      fail!();
    },
  };

  return 0;
}

/// Draw the current frame on display.
#[no_mangle]
pub extern fn draw_frame(engine: &Engine) {
  if engine.display == 0 as egl::Display {
    // No display.
    return;
  }

  // Just fill the screen with a color.
  let r = (engine.state.x as f32) / (engine.width as f32);
  let g = engine.state.angle;
  let b = (engine.state.y as f32) / (engine.height as f32);
  gl::clear_color(r, g, b, 1.0);

  match gl::clear(gl::COLOR_BUFFER_BIT) {
    Ok(()) => (),
    Err(e) => {
      log::e_f(format!("gl::clear(gl::COLOR_BUFFER_BIT) failed: {}", e));
      fail!();
    },
  };
  match egl::swap_buffers(engine.display, engine.surface) {
    Ok(()) => (),
    Err(e) => {
      log::e_f(format!("egl::swap_buffers() failed: {}", e));
      fail!();
    },
  };
}

/// Tear down the EGL context currently associated with the display.
#[no_mangle]
pub extern fn term_display(engine: &mut Engine) {
  if engine.display != egl::NO_DISPLAY {
    match egl::make_current(engine.display, egl::NO_SURFACE, egl::NO_SURFACE, egl::NO_CONTEXT) {
      Ok(()) => (),
      Err(e) => {
        log::e_f(format!("egl::make_current() failed: {}", e));
        fail!();
      },
    };
    if engine.context != egl::NO_CONTEXT {
      match egl::destroy_context(engine.display, engine.context) {
        Ok(()) => (),
        Err(e) => {
          log::e_f(format!("egl::destroy_context() failed: {}", e));
          fail!();
        },
      };
    }
    if engine.surface != egl::NO_SURFACE {
      match egl::destroy_surface(engine.display, engine.surface) {
        Ok(()) => (),
        Err(e) => {
          log::e_f(format!("egl::destroy_surface() failed: {}", e));
          fail!();
        },
      };
    }
    match egl::terminate(engine.display) {
      Ok(()) => (),
      Err(e) => {
        log::e_f(format!("egl::terminate() failed: {}", e));
        fail!();
      },
    }
  }

  engine.animating = 0;
  engine.display = egl::NO_DISPLAY;
  engine.context = egl::NO_CONTEXT;
  engine.surface = egl::NO_SURFACE;
}
