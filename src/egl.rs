extern crate libc;

use libc::{c_uint, c_void};
use std::ptr;
use std::result::Result;
use std::vec::Vec;

pub type Display = *const c_void;
pub const NO_DISPLAY: Display = 0 as Display;

pub type NativeDisplayType = *const c_void;
pub const DEFAULT_DISPLAY: NativeDisplayType = 0 as NativeDisplayType;

pub enum ANativeWindow {}
pub type NativeWindowType = *const ANativeWindow;

pub type Surface = *const c_void;
pub const NO_SURFACE: Surface = 0 as Surface;
pub type Context = *const c_void;
pub const NO_CONTEXT: Context = 0 as Context;

pub type Config = *const c_void;

// Config attributes.
pub const ALPHA_SIZE: Int = 0x3021;
pub const BLUE_SIZE: Int = 0x3022;
pub const GREEN_SIZE: Int = 0x3023;
pub const RED_SIZE: Int = 0x3024;
pub const DEPTH_SIZE: Int = 0x3025;
pub const STENCIL_SIZE: Int = 0x3026;
pub const CONFIG_ID: Int = 0x3028;
pub const NONE: Int =  0x3038;  /* Attrib list terminator */
pub const RENDERABLE_TYPE: Int = 0x3040;
pub const OPENGL_ES2_BIT: Int = 0x0004;  /* EGL_RENDERABLE_TYPE mask bits */
pub const NATIVE_VISUAL_ID: Int = 0x302E;

// Context attributes.
pub const CONTEXT_CLIENT_VERSION: Int = 0x3098;

// QuerySurface targets
pub const HEIGHT: Int = 0x3056;
pub const WIDTH: Int = 0x3057;

pub type Int = i32;

// Error codes.
type Boolean = c_uint;
const FALSE: Boolean = 0;
const TRUE: Boolean = 1;
const NOT_INITIALIZED: Boolean = 0x3001;
const BAD_ACCESS: Boolean = 0x3002;
const BAD_ALLOC: Boolean = 0x3003;
const BAD_ATTRIBUTE: Boolean = 0x3004;
const BAD_CONFIG: Boolean = 0x3005;
const BAD_CONTEXT: Boolean = 0x3006;
const BAD_CURRENT_SURFACE: Boolean = 0x3007;
const BAD_DISPLAY: Boolean = 0x3008;
const BAD_MATCH: Boolean = 0x3009;
const BAD_NATIVE_PIXMAP: Boolean = 0x300A;
const BAD_NATIVE_WINDOW: Boolean = 0x300B;
const BAD_PARAMETER: Boolean = 0x300C;
const BAD_SURFACE: Boolean = 0x300D;
const CONTEXT_LOST: Boolean = 0x300E;  // EGL 1.1 - IMG_power_management


pub fn get_display(display_id: NativeDisplayType) -> Display {
  unsafe {
    eglGetDisplay(display_id)
  }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
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
    eglInitialize(display, ptr::null_mut(), ptr::null_mut())
  };
  match res {
    TRUE => Ok(()),
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_DISPLAY => Err(Error::BadDisplay),
        _ => panic!("Unknown error from eglInitialize(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglInitialize(): {}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_DISPLAY => Err(Error::BadDisplay),
        _ => panic!("Unknown error from eglInitialize(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglInitialize(): {}", res),
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
      configs.truncate(num_config as usize);
      Ok(())
    },
    FALSE => {
      let err = unsafe { eglGetError() } as Boolean;
      match err {
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_ATTRIBUTE => Err(Error::BadAttribute),
        BAD_DISPLAY => Err(Error::BadDisplay),
        BAD_PARAMETER => Err(Error::BadParameter),
        _ => panic!("Unknown error from eglChooseConfig(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglChooseConfig(): {}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_ATTRIBUTE => Err(Error::BadAttribute),
        BAD_CONFIG => Err(Error::BadConfig),
        BAD_DISPLAY => Err(Error::BadDisplay),
        _ => panic!("Unknown error from eglGetConfigAttrib(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglGetConfigAttrib(): {}", res),
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
      NOT_INITIALIZED => Err(Error::NotInitialized),
      BAD_ALLOC => Err(Error::BadAlloc),
      BAD_ATTRIBUTE => Err(Error::BadAttribute),
      BAD_CONFIG => Err(Error::BadConfig),
      BAD_DISPLAY => Err(Error::BadDisplay),
      BAD_MATCH => Err(Error::BadMatch),
      BAD_NATIVE_WINDOW => Err(Error::BadNativeWindow),
      _ => panic!("Unknown error from eglCreateWindowSurface(): {:?}", res),
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
      NOT_INITIALIZED => Err(Error::NotInitialized),
      BAD_ALLOC => Err(Error::BadAlloc),
      BAD_ATTRIBUTE => Err(Error::BadAttribute),
      BAD_CONFIG => Err(Error::BadConfig),
      BAD_DISPLAY => Err(Error::BadDisplay),
      BAD_MATCH => Err(Error::BadMatch),
      BAD_NATIVE_WINDOW => Err(Error::BadNativeWindow),
      _ => panic!("Unknown error from eglCreateWindowSurface(): {:?}", res),
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
      NOT_INITIALIZED => Err(Error::NotInitialized),
      BAD_ALLOC => Err(Error::BadAlloc),
      BAD_ATTRIBUTE => Err(Error::BadAttribute),
      BAD_CONFIG => Err(Error::BadConfig),
      BAD_CONTEXT => Err(Error::BadContext),
      BAD_DISPLAY => Err(Error::BadDisplay),
      BAD_MATCH => Err(Error::BadMatch),
      _ => panic!("Unknown error from eglCreateContext(): {:?}", res),
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
      NOT_INITIALIZED => Err(Error::NotInitialized),
      BAD_ALLOC => Err(Error::BadAlloc),
      BAD_ATTRIBUTE => Err(Error::BadAttribute),
      BAD_CONFIG => Err(Error::BadConfig),
      BAD_CONTEXT => Err(Error::BadContext),
      BAD_DISPLAY => Err(Error::BadDisplay),
      BAD_MATCH => Err(Error::BadMatch),
      _ => panic!("Unknown error from eglCreateContext(): {:?}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_ACCESS => Err(Error::BadAccess),
        BAD_ALLOC => Err(Error::BadAlloc),
        BAD_CONTEXT => Err(Error::BadContext),
        BAD_CURRENT_SURFACE => Err(Error::BadCurrentSurface),
        BAD_DISPLAY => Err(Error::BadDisplay),
        BAD_MATCH => Err(Error::BadMatch),
        BAD_NATIVE_PIXMAP => Err(Error::BadNativePixmap),
        BAD_NATIVE_WINDOW => Err(Error::BadNativeWindow),
        BAD_SURFACE => Err(Error::BadSurface),
        CONTEXT_LOST => Err(Error::ContextLost),
        _ => panic!("Unknown error from eglMakeCurrent(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglMakeCurrent(): {}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_ATTRIBUTE => Err(Error::BadAttribute),
        BAD_DISPLAY => Err(Error::BadDisplay),
        BAD_SURFACE => Err(Error::BadSurface),
        _ => panic!("Unknown error from eglQuerySurface(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglQuerySurface(): {}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_DISPLAY => Err(Error::BadDisplay),
        BAD_SURFACE => Err(Error::BadSurface),
        CONTEXT_LOST => Err(Error::ContextLost),
        _ => panic!("Unknown error from eglSwapBuffers(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglSwapBuffers(): {}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_DISPLAY => Err(Error::BadDisplay),
        BAD_CONTEXT => Err(Error::BadContext),
        _ => panic!("Unknown error from eglDestroyContext(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglDestroyContext(): {}", res),
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
        NOT_INITIALIZED => Err(Error::NotInitialized),
        BAD_DISPLAY => Err(Error::BadDisplay),
        BAD_SURFACE => Err(Error::BadSurface),
        _ => panic!("Unknown error from eglDestroySurface(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglDestroySurface(): {}", res),
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
        BAD_DISPLAY => Err(Error::BadDisplay),
        _ => panic!("Unknown error from eglTerminate(): {}", err),
      }
    },
    _ => panic!("Unknown return value from eglTerminate(): {}", res),
  }
}

#[link(name = "EGL")]
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
