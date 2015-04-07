use libc::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::sync::{Mutex, Once, ONCE_INIT};

type Atom = c_ulong;
type Bool = c_int;
type Status = c_int;

type XID = c_ulong;
type Colormap = XID;
type Cursor = XID;
type Pixmap = XID;
type Window = XID;

type XIM = *mut ();
type XrmDatabase = *const ();
type XIC = *mut ();

#[repr(C)]
struct Display;

#[repr(C)]
struct XErrorEvent {
  type_: c_int,
  display: *mut Display,
  resourceid: XID,
  serial: c_ulong,
  error_code: c_char,
  request_code: c_char,
  minor_code: c_char,
}

static THREAD_INIT: Once = ONCE_INIT;

fn ensure_thread_init() {
  THREAD_INIT.call_once(|| {
    init_threads();

    extern "C" fn error_callback(_: *mut Display, event: *mut XErrorEvent) -> c_int {
      let error_code = unsafe { (*event).error_code };
      let major = unsafe { (*event).request_code };
      let minor = unsafe { (*event).minor_code };
      println!("X11 error code={}, major={}, minor={}", error_code, major, minor);
      0
    }

    set_error_handler(error_callback);
  });
}

// XOpenIM doesn't seem to be thread-safe
lazy_static! {
  static ref GLOBAL_XOPENIM_LOCK: Mutex<()> = Mutex::new(());
}

pub struct XWindow {
  display: *mut Display,
  screen_id: c_int,
  window: Window,
  wm_delete_window: Atom,
  input_method: XIM,
  input_context: XIC,
  context: GLXContext,
}

impl Drop for XWindow {
  fn drop(&mut self) {
    destroy_context(self.display, self.context);
    destroy_ic(self.input_context);
    close_im(self.input_method);
    destroy_window(self.display, self.window);
    close_display(self.display);
  }
}

impl XWindow {
  pub fn new(title: &str) -> XWindow {
    ensure_thread_init();

    let display = open_display();
    let screen_id = default_screen(display);

    let configs = {
      let visual_attributes = vec![
        GLX_X_RENDERABLE, 1,
        GLX_DRAWABLE_TYPE, GLX_WINDOW_BIT,
        GLX_RENDER_TYPE, GLX_RGBA_BIT,
        GLX_X_VISUAL_TYPE,GLX_TRUE_COLOR,
        GLX_RED_SIZE, 8,
        GLX_GREEN_SIZE, 8,
        GLX_BLUE_SIZE, 8,
        GLX_ALPHA_SIZE, 8,
        GLX_DEPTH_SIZE, 24,
        GLX_STENCIL_SIZE, 8,
        GLX_DOUBLEBUFFER, 1,
        0,
      ];
      choose_fb_config(display, screen_id, &visual_attributes)
    };
    let config = configs[0];
    println!("Frame buffer configuration: {}", dump_fb_config(display, config));

    let visual_info = visual_info_from_fb_config(display, config);
    println!("Visual info: id 0x{:x}, screen {}, depth {}, class {}, red mask 0x{:x}, green mask 0x{:x}, blue mask 0x{:x}, colormap size {}, bits per RGB {}", visual_info.visual_id, visual_info.screen, visual_info.depth, visual_info.class, visual_info.red_mask, visual_info.green_mask, visual_info.blue_mask, visual_info.colormap_size, visual_info.bits_per_rgb);

    let root_window = default_root_window(display);
    let color_map = create_color_map(display, root_window, visual_info.visual, ALLOC_NONE);

    let window = {
      let mut set_window_attr = {
        let mut swa: XSetWindowAttributes = unsafe { mem::zeroed() };
        swa.colormap = color_map;
        swa.event_mask = EXPOSURE_MASK | STRUCTURE_NOTIFY_MASK | VISIBILITY_CHANGE_MASK | KEY_PRESS_MASK |
          POINTER_MOTION_MASK | KEY_RELEASE_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK | KEYMAP_STATE_MASK;
        swa.border_pixel = 0;
        swa.override_redirect = 0;
        swa
      };
      let window_attr = CW_BORDER_PIXEL | CW_COLORMAP | CW_EVENT_MASK;
      let dimensions = (800, 600);

      create_window(display, root_window, 0, 0, dimensions.0 as c_uint, dimensions.1 as c_uint,
        visual_info.depth, visual_info.visual, window_attr, &mut set_window_attr)
    };
    println!("Created window: 0x{:x}", window);

    store_name(display, window, title);
    map_raised(display, window);
    let wm_delete_window = {
      let wm_delete_window = intern_atom(display, "WM_DELETE_WINDOW", false);
      set_wm_protocol(display, window, wm_delete_window);
      wm_delete_window
    };
    flush(display);

    let input_method = {
      let _lock = GLOBAL_XOPENIM_LOCK.lock().unwrap();
      open_im(display)
    };

    let input_context = {
      let ic = create_ic(input_method, XIM_PREEDIT_NOTHING | XIM_STATUS_NOTHING, window);
      set_ic_focus(ic);
      ic
    };

    // Make keyboard input repeat detectable.
    set_detectable_auto_repeat(display);

    // Create GL context.
    let context = {
      let context_attributes = vec![
        GLX_CONTEXT_MAJOR_VERSION, 1,
        GLX_CONTEXT_MINOR_VERSION, 4,
        // TODO: Only for debug build:
        GLX_EXTRA_CONTEXT_FLAGS_ARB, GLX_EXTRA_CONTEXT_DEBUG_BIT_ARB,
        0,
      ];
      create_context_attribs_arb(display, config, ptr::null(), true, &context_attributes)
    };

    XWindow {
      display: display,
      screen_id: screen_id,
      window: window,
      wm_delete_window: wm_delete_window,
      input_method: input_method,
      input_context: input_context,
      context: context,
    }
  }

  pub fn make_current(&self) {
    let rc = unsafe { glXMakeCurrent(self.display, self.window, self.context) };
    if rc == 0 {
      panic!("glXMakeCurrent() failed");
    }
  }

  pub fn swap_buffers(&self) {
    unsafe { glXSwapBuffers(self.display, self.window); }
  }

  pub fn flush(&self) {
    flush(self.display);
  }
}

// Wrapper functions for FFI.
fn init_threads() {
  unsafe { XInitThreads(); }
}

fn set_error_handler(callback: extern "C" fn(display: *mut Display, event: *mut XErrorEvent) -> c_int) {
  unsafe { XSetErrorHandler(callback); }
}

fn open_display() -> *mut Display {
  let display = unsafe { XOpenDisplay(ptr::null()) };
  if display.is_null() {
    panic!("XOpenDisplay() failed");
  }
  display
}

fn close_display(display: *mut Display) {
  unsafe { XCloseDisplay(display); }
}

fn destroy_window(display: *mut Display, window: Window) {
  unsafe { XDestroyWindow(display, window) };
}

fn default_screen(display: *mut Display) -> c_int {
  unsafe { XDefaultScreen(display) }
}

const GLX_WINDOW_BIT: c_int = 0x00000001;
const GLX_RGBA_BIT: c_int = 0x00000001;

const GLX_DOUBLEBUFFER: c_int = 5;
const GLX_RED_SIZE: c_int = 8;
const GLX_GREEN_SIZE: c_int = 9;
const GLX_BLUE_SIZE: c_int = 10;
const GLX_ALPHA_SIZE: c_int = 11;
const GLX_DEPTH_SIZE: c_int = 12;
const GLX_STENCIL_SIZE: c_int = 13;

const GLX_X_VISUAL_TYPE: c_int = 0x22;
const GLX_TRUE_COLOR: c_int = 0x8002;
const GLX_DRAWABLE_TYPE: c_int = 0x8010;
const GLX_RENDER_TYPE: c_int = 0x8011;
const GLX_X_RENDERABLE: c_int = 0x8012;

fn choose_fb_config(display: *mut Display, screen_id: c_int, visual_attributes: &[c_int]) -> Vec<GLXFBConfig> {
  let mut num_fb: c_int = unsafe { mem::uninitialized() };
  let fb = unsafe {
    glXChooseFBConfig(display, screen_id, visual_attributes.as_ptr(), &mut num_fb)
  };
  if fb.is_null() {
    panic!("glXChooseFBConfig() failed");
  }
  if num_fb == 0 {
    panic!("glXChooseFBConfig() returned no frame buffer configurations");
  }

  let configs = unsafe {
    Vec::from_raw_buf(fb, num_fb as usize)
  };
  unsafe { XFree(fb as *const c_void) };
  configs
}

fn dump_fb_config(display: *mut Display, config: GLXFBConfig) -> String {
  let config_id = config_attribute_int(display, config, GLX_FBCONFIG_ID);

  let color_bits = config_attribute_int(display, config, GLX_BUFFER_SIZE);
  let red_bits = config_attribute_int(display, config, GLX_RED_SIZE);
  let green_bits = config_attribute_int(display, config, GLX_GREEN_SIZE);
  let blue_bits = config_attribute_int(display, config, GLX_BLUE_SIZE);
  let alpha_bits = config_attribute_int(display, config, GLX_ALPHA_SIZE);
  let depth_bits = config_attribute_int(display, config, GLX_DEPTH_SIZE);
  let stencil_bits = config_attribute_int(display, config, GLX_STENCIL_SIZE);

  let double_buffer =  config_attribute_bool(display, config, GLX_DOUBLEBUFFER);
  let stereo = config_attribute_bool(display, config, GLX_STEREO);

  let visual_id = config_attribute_int(display, config, GLX_VISUAL_ID);

  format!("id: 0x{:x}, {} bits, {}/{}/{}/{} RGBA bits, {}/{} depth/stencil bits, double buffer: {}, stereo: {}, visual id: 0x{:x}",
    config_id, color_bits, red_bits, green_bits, blue_bits, alpha_bits, depth_bits, stencil_bits,
    double_buffer, stereo, visual_id)
}

const GLX_VISUAL_ID: c_int = 0x800B;
const GLX_FBCONFIG_ID: c_int = 0x8013;
const GLX_BUFFER_SIZE: c_int = 2;
const GLX_STEREO: c_int = 6;

fn config_attribute_int(display: *mut Display, config: GLXFBConfig, attribute: c_int) -> c_int {
  let mut value: c_int = unsafe { mem::uninitialized() };
  let rc = unsafe {
    glXGetFBConfigAttrib(display, config, attribute, &mut value)
  };
  if rc != 0 {
    panic!("glXGetFBConfigAttrib() failed, return code {}", rc);
  }
  value
}

fn config_attribute_bool(display: *mut Display, config: GLXFBConfig, attribute: c_int) -> bool {
  config_attribute_int(display, config, attribute) != 0
}

fn visual_info_from_fb_config(display: *mut Display, config: GLXFBConfig) -> XVisualInfo {
  let vi = unsafe { glXGetVisualFromFBConfig(display, config) };
  if vi.is_null() {
    panic!("glx::glXGetVisualFromFBConfig() failed");
  }
  let vi_copy = unsafe { ptr::read(vi as *const _) };
  unsafe { XFree(vi as *const c_void) };
  vi_copy
}

fn default_root_window(display: *mut Display) -> Window {
  unsafe { XDefaultRootWindow(display) }
}

const ALLOC_NONE: c_int = 0;

fn create_color_map(display: *mut Display, w: Window, visual: *mut Visual, alloc: c_int) -> Colormap {
  unsafe {
    XCreateColormap(display, w, visual, alloc)
  }
}

fn create_window(display: *mut Display, parent: Window, x: c_int, y: c_int, width: c_uint, height: c_uint,
  depth: c_int, visual: *mut Visual, value_mask: c_ulong, attributes: *mut XSetWindowAttributes) -> Window {

  unsafe {
    XCreateWindow(display, parent, x, y, width, height, 0, depth, INPUT_OUTPUT, visual, value_mask, attributes)
  }
}

fn map_raised(display: *mut Display, w: Window) {
  unsafe { XMapRaised(display, w); };
}

fn flush(display: *mut Display) {
  unsafe { XFlush(display); };
}

fn store_name(display: *mut Display, w: Window, name: &str) {
  let c_name = CString::new(name).unwrap();
  unsafe {
    XStoreName(display, w, c_name.as_ptr());
  }
}

fn intern_atom(display: *mut Display, atom_name: &str, only_if_exists: bool) -> Atom {
  let c_atom_name = CString::new(atom_name).unwrap();
  let c_only_if_exists = if only_if_exists { 1 } else { 0 };
  let atom = unsafe {
    XInternAtom(display, c_atom_name.as_ptr(), c_only_if_exists)
  };
  if atom == 0 {
    panic!("XInternAtom({}) failed", atom_name);
  }
  atom
}

fn set_wm_protocol(display: *mut Display, w: Window, protocol: Atom) {
  let mut protocol_mut = protocol;
  let status = unsafe {
    XSetWMProtocols(display, w, &mut protocol_mut, 1)
  };
  if status == 0 {
    panic!("XSetWMProtocols() failed");
  }
}

fn open_im(display: *mut Display) -> XIM {
  let im = unsafe {
    XOpenIM(display, ptr::null(), ptr::null_mut(), ptr::null_mut())
  };
  if im.is_null() {
    panic!("XOpenIM() failed");
  }
  im
}

fn close_im(input_method: XIM) {
  unsafe {
    XCloseIM(input_method);
  }
}

// Bit flags for create_ic() parameter input_style:
const XIM_PREEDIT_NOTHING: c_long = 0x0008;
const XIM_STATUS_NOTHING: c_long = 0x0400;

fn create_ic(im: XIM, input_style: c_long, client_window: Window) -> XIC {
  let c_input_style_name = CString::new("inputStyle").unwrap();
  let c_client_window_name = CString::new("clientWindow").unwrap();
  let ic = unsafe {
    XCreateIC(im, c_input_style_name.as_ptr(), input_style, c_client_window_name.as_ptr(),
      client_window, ptr::null())
  };
  if ic.is_null() {
    panic!("XCreateIC() failed");
  }
  ic
}

fn set_ic_focus(ic: XIC) {
  unsafe {
    XSetICFocus(ic);
  }
}

fn destroy_ic(ic: XIC) {
  unsafe {
    XDestroyIC(ic);
  }
}

fn set_detectable_auto_repeat(display: *mut Display) {
  let mut supported_ptr = 0;
  unsafe {
    XkbSetDetectableAutoRepeat(display, 1, &mut supported_ptr);
  }
  if supported_ptr == 0 {
    panic!("XkbSetDetectableAutoRepeat() failed");
  }
}

fn get_proc_address(proc_name: &str) -> GLXextFuncPtr {
  let c_proc_name = CString::new(proc_name).unwrap();
  let maybe_address = unsafe {
    glXGetProcAddress(c_proc_name.as_ptr() as *const u8)
  };
  match maybe_address {
    Some(p) => p,
    None => panic!("glXGetProcAddress({}) failed", proc_name),
  }
}

fn create_context_attribs_arb(display: *mut Display, config: GLXFBConfig, share_context: GLXContext,
  direct: bool, attrib_list: &[c_int]) -> GLXContext {

  // NOTE: No caching, OK to use if only called once.
  let create_context_attribs_arb_fn = get_proc_address("glXCreateContextAttribsARB");
  type CreateContextAttribsArbFn =
    extern "system" fn(*mut Display, GLXFBConfig, GLXContext, Bool, *const c_int) -> GLXContext;
  let create_context_attribs_arb_fn = unsafe {
    mem::transmute::<_, CreateContextAttribsArbFn>(create_context_attribs_arb_fn)
  };

  let c_direct = if direct { 1 } else { 0 };
  let context =
    create_context_attribs_arb_fn(display, config, share_context, c_direct, attrib_list.as_ptr());
  if context.is_null() {
    panic!("glXCreateContextAttribsARB() failed");
  }
  context
}

fn destroy_context(display: *mut Display, context: GLXContext) {
  unsafe { glXDestroyContext(display, context); }
}


const GLX_CONTEXT_MAJOR_VERSION: c_int = 0x2091;
const GLX_CONTEXT_MINOR_VERSION: c_int = 0x2092;
const GLX_EXTRA_CONTEXT_FLAGS_ARB: c_int = 0x2094;
const GLX_EXTRA_CONTEXT_DEBUG_BIT_ARB: c_int = 0x00000001;

// FFI functions.
#[repr(C)]
struct XSetWindowAttributes {
  background_pixmap: Pixmap,
  background_pixel: c_ulong,
  border_pixmap: Pixmap,
  border_pixel: c_ulong,
  bit_gravity: c_int,
  win_gravity: c_int,
  backing_store: c_int,
  backing_planes: c_ulong,
  backing_pixel: c_long,
  save_under: Bool,
  event_mask: c_long,
  do_not_propagate_mask: c_long,
  override_redirect: Bool,
  colormap: Colormap,
  cursor: Cursor,
}

// For XSetWindowAttributes.event_mask:
const KEY_PRESS_MASK: c_long = (1<<0);
const KEY_RELEASE_MASK: c_long = (1<<1);
const BUTTON_PRESS_MASK: c_long = (1<<2);
const BUTTON_RELEASE_MASK: c_long = (1<<3);
const POINTER_MOTION_MASK: c_long = (1<<6);
const KEYMAP_STATE_MASK: c_long = (1<<14);
const EXPOSURE_MASK: c_long = (1<<15);
const VISIBILITY_CHANGE_MASK: c_long = (1<<16);
const STRUCTURE_NOTIFY_MASK: c_long = (1<<17);

// For XCreateWindow() parameter valuemask:
const CW_BORDER_PIXEL: c_ulong = (1<<3);
const CW_EVENT_MASK: c_ulong = (1<<11);
const CW_COLORMAP: c_ulong = (1<<13);

// For XCreateWindow() parameter class:
const INPUT_OUTPUT: c_uint = 1;

#[link(name = "X11")]
extern "C" {
  fn XCloseDisplay(display: *mut Display);
  fn XCloseIM(im: XIM) -> Status;
  fn XCreateColormap(display: *mut Display, w: Window, visual: *mut Visual, alloc: c_int) -> Colormap;
  // This is a vararg function.
  fn XCreateIC(im: XIM, a: *const c_char, b: c_long, c: *const c_char, d: Window, e: *const ()) -> XIC;
  fn XCreateWindow(display: *mut Display, parent: Window, x: c_int, y: c_int, width: c_uint, height: c_uint,
    border_width: c_uint, depth: c_int, class: c_uint, visual: *mut Visual, valuemask: c_ulong,
    attributes: *mut XSetWindowAttributes) -> Window;
  fn XDefaultRootWindow(display: *mut Display) -> Window;
  fn XDefaultScreen(display: *mut Display) -> c_int;
  fn XDestroyIC(ic: XIC);
  fn XDestroyWindow(display: *mut Display, w: Window);
  fn XFlush(display: *mut Display);
  fn XFree(data: *const c_void);
  fn XInitThreads() -> Status;
  fn XInternAtom(display: *mut Display, atom_name: *const c_char, only_if_exists: Bool) -> Atom;
  fn XkbSetDetectableAutoRepeat(display: *mut Display, detectable: Bool, supported_rtm: *mut Bool) -> Bool;
  fn XMapRaised(display: *mut Display, w: Window);
  fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
  fn XOpenIM(display: *mut Display, db: XrmDatabase, res_name: *mut c_char, res_class: *mut c_char) -> XIM;
  fn XSetErrorHandler(callback: extern "C" fn(display: *mut Display, event: *mut XErrorEvent) -> c_int) -> c_int;
  fn XSetICFocus(ic: XIC);
  fn XSetWMProtocols(display: *mut Display, w: Window, protocols: *mut Atom, count: c_int) -> Status;
  fn XStoreName(display: *mut Display, w: Window, window_name: *const c_char);
}


type Visual = ();
type VisualID = c_ulong;

#[repr(C)]
struct XVisualInfo {
  visual: *mut Visual,
  visual_id: VisualID,
  screen: c_int,
  depth: c_int,
  class: c_int,
  red_mask: c_ulong,
  green_mask: c_ulong,
  blue_mask: c_ulong,
  colormap_size: c_int,
  bits_per_rgb: c_int,
}

type GLubyte = c_uchar;
type GLXFBConfig = *const c_void;
type GLXContext = *const c_void;
type GLXextFuncPtr = extern "system" fn();
type GLXDrawable = XID;

#[link(name = "GL")]
extern "C" {
  fn glXChooseFBConfig(display: *mut Display, screen_id: c_int, attrib_list: *const c_int, num_elements: *mut c_int) -> *mut GLXFBConfig;
  fn glXDestroyContext(display: *mut Display, context: GLXContext);
  fn glXGetFBConfigAttrib(display: *mut Display, config: GLXFBConfig, attribute: c_int, value: *mut c_int) -> c_int;
  fn glXGetProcAddress(proc_name: *const GLubyte) -> Option<GLXextFuncPtr>;
  fn glXGetVisualFromFBConfig(display: *mut Display, config: GLXFBConfig) -> *mut XVisualInfo;
  fn glXMakeCurrent(display: *mut Display, drawable: GLXDrawable, context: GLXContext) -> Bool;
  fn glXSwapBuffers(display: *mut Display, drawable: GLXDrawable);
}
