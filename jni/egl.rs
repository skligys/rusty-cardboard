extern crate libc;
use libc::{c_uint, c_void};
use std::ptr;
use std::result::Result;
use std::vec::Vec;

use log;
use native_window;

// TODO: Figure out how to put macros in a separate module and import when needed.

/// Logs the error to Android error logging and fails.
macro_rules! a_fail(
  ($msg: expr) => ({
    log::e($msg);
    fail!();
  });
  ($fmt: expr, $($arg:tt)*) => ({
    log::e_f(format!($fmt, $($arg)*));
    fail!();
  });
)

pub type Display = *const c_void;
pub static NO_DISPLAY: Display = 0 as Display;

type NativeDisplayType = *const c_void;
pub static DEFAULT_DISPLAY: NativeDisplayType = 0 as NativeDisplayType;

pub type Surface = *const c_void;
pub static NO_SURFACE: Surface = 0 as Surface;
pub type Context = *const c_void;
pub static NO_CONTEXT: Context = 0 as Context;

pub type Config = *const c_void;

// Config attributes.
pub static BLUE_SIZE: Int = 0x3022;
pub static GREEN_SIZE: Int = 0x3023;
pub static RED_SIZE: Int = 0x3024;
pub static NONE: Int =  0x3038;  /* Attrib list terminator */
pub static RENDERABLE_TYPE: Int = 0x3040;
pub static OPENGL_ES2_BIT: Int = 0x0004;  /* EGL_RENDERABLE_TYPE mask bits */
pub static NATIVE_VISUAL_ID: Int = 0x302E;

// Context attributes.
pub static CONTEXT_CLIENT_VERSION: Int = 0x3098;

// QuerySurface targets
pub static HEIGHT: Int = 0x3056;
pub static WIDTH: Int = 0x3057;

type NativeWindowType = *const native_window::NativeWindow;

type Int = i32;

// Error codes.
type Boolean = c_uint;
static FALSE: Boolean = 0;
static TRUE: Boolean = 1;
static NOT_INITIALIZED: Boolean = 0x3001;
static BAD_ACCESS: Boolean = 0x3002;
static BAD_ALLOC: Boolean = 0x3003;
static BAD_ATTRIBUTE: Boolean = 0x3004;
static BAD_CONFIG: Boolean = 0x3005;
static BAD_CONTEXT: Boolean = 0x3006;
static BAD_CURRENT_SURFACE: Boolean = 0x3007;
static BAD_DISPLAY: Boolean = 0x3008;
static BAD_MATCH: Boolean = 0x3009;
static BAD_NATIVE_PIXMAP: Boolean = 0x300A;
static BAD_NATIVE_WINDOW: Boolean = 0x300B;
static BAD_PARAMETER: Boolean = 0x300C;
static BAD_SURFACE: Boolean = 0x300D;
static CONTEXT_LOST: Boolean = 0x300E;  // EGL 1.1 - IMG_power_management


pub fn get_display(display_id: NativeDisplayType) -> Display {
  unsafe {
    eglGetDisplay(display_id)
  }
}

#[deriving(Show)]
enum Error {
  NoSurface,
  NotInitialized,
  BadAccess,
  BadAlloc,
  BadAttribute,
  BadConfig,
  BadContext,
  BadCurrentSurface,
  BadDisplay,
  BadMatch,
  BadNativePixmap,
  BadNativeWindow,
  BadParameter,
  BadSurface,
  ContextLost,
}

pub fn initialize(display: Display) -> Result<(), Error> {
  let res = unsafe {
    eglInitialize(display, ptr::mut_null(), ptr::mut_null())
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_DISPLAY => Err(BadDisplay),
        _ => a_fail!("Unknown error from eglInitialize(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglInitialize(): {}", res),
  }
}

#[allow(dead_code)]
pub fn initialize_with_version(display: Display) -> Result<(Int, Int), Error> {
  let mut major: Int = 0;
  let mut minor: Int = 0;
  let res = unsafe {
    eglInitialize(display, &mut major, &mut minor)
  };
  match res {
    TRUE => Ok((major, minor)),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_DISPLAY => Err(BadDisplay),
        _ => a_fail!("Unknown error from eglInitialize(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglInitialize(): {}", res),
  }
}

pub fn choose_config(display: Display, attribs: &[Int], configs: &mut Vec<Config>) ->
  Result<(), Error> {
  let mut num_config: Int = 0;
  let res = unsafe {
    eglChooseConfig(display, attribs.as_ptr(), configs.as_mut_ptr(), configs.len() as Int, &mut num_config)
  };
  match res {
    TRUE => {
      configs.truncate(num_config as uint);
      Ok(())
    },
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_ATTRIBUTE => Err(BadAttribute),
        BAD_DISPLAY => Err(BadDisplay),
        BAD_PARAMETER => Err(BadParameter),
        _ => a_fail!("Unknown error from eglChooseConfig(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglChooseConfig(): {}", res),
  }
}

pub fn get_config_attrib(display: Display, config: Config, attribute: Int) -> Result<Int, Error> {
  let mut result: Int = 0;
  let res = unsafe {
    eglGetConfigAttrib(display, config, attribute, &mut result)
  };
  match res {
    TRUE => Ok(result),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_ATTRIBUTE => Err(BadAttribute),
        BAD_CONFIG => Err(BadConfig),
        BAD_DISPLAY => Err(BadDisplay),
        _ => a_fail!("Unknown error from eglGetConfigAttrib(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglGetConfigAttrib(): {}", res),
  }
}

pub fn create_window_surface(display: Display, config: Config, window: NativeWindowType) ->
  Result<Surface, Error> {
  let res = unsafe {
    eglCreateWindowSurface(display, config, window, ptr::null())
  };
  if res != NO_SURFACE {
    Ok(res)
  } else {
    let err = unsafe { eglGetError() } as Boolean;
    match err {
      NOT_INITIALIZED => Err(NotInitialized),
      BAD_ALLOC => Err(BadAlloc),
      BAD_ATTRIBUTE => Err(BadAttribute),
      BAD_CONFIG => Err(BadConfig),
      BAD_DISPLAY => Err(BadDisplay),
      BAD_MATCH => Err(BadMatch),
      BAD_NATIVE_WINDOW => Err(BadNativeWindow),
      _ => a_fail!("Unknown error from eglCreateWindowSurface(): {}", res),
    }
  }
}

#[allow(dead_code)]
pub fn create_window_surface_with_attribs(display: Display, config: Config, window: NativeWindowType,
  attribs: &[Int]) -> Result<Surface, Error> {
  let res = unsafe {
    eglCreateWindowSurface(display, config, window, attribs.as_ptr())
  };
  if res != NO_SURFACE {
    Ok(res)
  } else {
    let err = unsafe { eglGetError() } as Boolean;
    match err {
      NOT_INITIALIZED => Err(NotInitialized),
      BAD_ALLOC => Err(BadAlloc),
      BAD_ATTRIBUTE => Err(BadAttribute),
      BAD_CONFIG => Err(BadConfig),
      BAD_DISPLAY => Err(BadDisplay),
      BAD_MATCH => Err(BadMatch),
      BAD_NATIVE_WINDOW => Err(BadNativeWindow),
      _ => a_fail!("Unknown error from eglCreateWindowSurface(): {}", res),
    }
  }
}

#[allow(dead_code)]
pub fn create_context(display: Display, config: Config, share_context: Context) ->
  Result<Context, Error> {
  let res = unsafe {
    eglCreateContext(display, config, share_context, ptr::null())
  };
  if res != ptr::null() {
    Ok(res)
  } else {
    let err = unsafe { eglGetError() } as Boolean;
    match err {
      NOT_INITIALIZED => Err(NotInitialized),
      BAD_ALLOC => Err(BadAlloc),
      BAD_ATTRIBUTE => Err(BadAttribute),
      BAD_CONFIG => Err(BadConfig),
      BAD_CONTEXT => Err(BadContext),
      BAD_DISPLAY => Err(BadDisplay),
      BAD_MATCH => Err(BadMatch),
      _ => a_fail!("Unknown error from eglCreateContext(): {}", res),
    }
  }
}

pub fn create_context_with_attribs(display: Display, config: Config, share_context: Context,
  attribs: &[Int]) -> Result<Context, Error> {
  let res = unsafe {
    eglCreateContext(display, config, share_context, attribs.as_ptr())
  };
  if res != ptr::null() {
    Ok(res)
  } else {
    let err = unsafe { eglGetError() } as Boolean;
    match err {
      NOT_INITIALIZED => Err(NotInitialized),
      BAD_ALLOC => Err(BadAlloc),
      BAD_ATTRIBUTE => Err(BadAttribute),
      BAD_CONFIG => Err(BadConfig),
      BAD_CONTEXT => Err(BadContext),
      BAD_DISPLAY => Err(BadDisplay),
      BAD_MATCH => Err(BadMatch),
      _ => a_fail!("Unknown error from eglCreateContext(): {}", res),
    }
  }
}

pub fn make_current(display: Display, draw: Surface, read: Surface, context: Context) -> Result<(), Error> {
  let res = unsafe {
    eglMakeCurrent(display, draw, read, context)
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_ACCESS => Err(BadAccess),
        BAD_ALLOC => Err(BadAlloc),
        BAD_CONTEXT => Err(BadContext),
        BAD_CURRENT_SURFACE => Err(BadCurrentSurface),
        BAD_DISPLAY => Err(BadDisplay),
        BAD_MATCH => Err(BadMatch),
        BAD_NATIVE_PIXMAP => Err(BadNativePixmap),
        BAD_NATIVE_WINDOW => Err(BadNativeWindow),
        BAD_SURFACE => Err(BadSurface),
        CONTEXT_LOST => Err(ContextLost),
        _ => a_fail!("Unknown error from eglMakeCurrent(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglMakeCurrent(): {}", res),
  }
}

pub fn query_surface(display: Display, surface: Surface, attribute: Int) -> Result<Int, Error> {
  let mut value: Int = 0;
  let res = unsafe {
    eglQuerySurface(display, surface, attribute, &mut value)
  };
  match res {
    TRUE => Ok(value),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_ATTRIBUTE => Err(BadAttribute),
        BAD_DISPLAY => Err(BadDisplay),
        BAD_SURFACE => Err(BadSurface),
        _ => a_fail!("Unknown error from eglQuerySurface(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglQuerySurface(): {}", res),
  }
}

pub fn swap_buffers(display: Display, surface: Surface) -> Result<(), Error> {
  let res = unsafe {
    eglSwapBuffers(display, surface)
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_DISPLAY => Err(BadDisplay),
        BAD_SURFACE => Err(BadSurface),
        CONTEXT_LOST => Err(ContextLost),
        _ => a_fail!("Unknown error from eglSwapBuffers(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglSwapBuffers(): {}", res),
  }
}

pub fn destroy_context(display: Display, context: Context) -> Result<(), Error> {
  let res = unsafe {
    eglDestroyContext(display, context)
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_DISPLAY => Err(BadDisplay),
        BAD_CONTEXT => Err(BadContext),
        _ => a_fail!("Unknown error from eglDestroyContext(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglDestroyContext(): {}", res),
  }
}

pub fn destroy_surface(display: Display, surface: Surface) -> Result<(), Error> {
  let res = unsafe {
    eglDestroySurface(display, surface)
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(NotInitialized),
        BAD_DISPLAY => Err(BadDisplay),
        BAD_SURFACE => Err(BadSurface),
        _ => a_fail!("Unknown error from eglDestroySurface(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglDestroySurface(): {}", res),
  }
}

pub fn terminate(display: Display) -> Result<(), Error> {
  let res = unsafe {
    eglTerminate(display)
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        BAD_DISPLAY => Err(BadDisplay),
        _ => a_fail!("Unknown error from eglTerminate(): {}", err),
      }
    },
    _ => a_fail!("Unknown return value from eglTerminate(): {}", res),
  }
}

extern {
  fn eglGetDisplay(display_id: NativeDisplayType) -> Display;
  fn eglInitialize(display: Display, major: *mut Int, minor: *mut Int) -> Boolean;
  fn eglChooseConfig(display: Display, attrib_list: *const Int, configs: *mut Config,
    config_size: Int, num_config: *mut Int) -> Boolean;
  fn eglGetConfigAttrib(display: Display, config: Config, attribute: Int, value: *mut Int) -> Boolean;
  fn eglCreateWindowSurface(display: Display, config: Config, window: NativeWindowType, attrib_list: *const Int) -> Surface;
  fn eglGetError() -> Int;
  fn eglCreateContext(display: Display, config: Config, share_context: Context, attrib_list: *const Int) -> Context;
  fn eglMakeCurrent(display: Display, draw: Surface, read: Surface, context: Context) -> Boolean;
  fn eglQuerySurface(display: Display, surface: Surface, attribute: Int, value: *mut Int) -> Boolean;
  fn eglSwapBuffers(display: Display, surface: Surface) -> Boolean;
  fn eglDestroyContext(display: Display, context: Context) -> Boolean;
  fn eglDestroySurface(display: Display, surface: Surface) -> Boolean;
  fn eglTerminate(display: Display) -> Boolean;
}
