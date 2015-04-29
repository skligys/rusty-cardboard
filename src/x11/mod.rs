use libc::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};
use std::cell::Cell;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;
use std::str;
use std::sync::{Mutex, Once, ONCE_INIT};
use std::sync::atomic::{AtomicBool, Ordering};

use gl;
use x11::key::{ElementState, ScanCode, VirtualKeyCode};

mod key;

type Atom = c_ulong;
type Bool = c_int;
type Status = c_int;

type XID = c_ulong;
type Colormap = XID;
type Cursor = XID;
type KeySym = XID;
type Pixmap = XID;
type Window = XID;

type KeyCode = c_ulong;
type Time = c_ulong;

type XIM = *mut ();
type XrmDatabase = *const ();
type XIC = *mut ();

type Enum = c_uint;
type UInt = c_uint;
type SizeI = c_int;
type Char = c_char;

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
  window: Window,
  wm_delete_window: Atom,
  input_method: XIM,
  input_context: XIC,
  context: GLXContext,
  is_closed: AtomicBool,
  current_size: Cell<(c_int, c_int)>,
  /// Events that have been retreived from XLib but not dispatched via iterators yet.
  pending_events: Mutex<VecDeque<Event>>,
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

#[cfg(debug_assertions)]
lazy_static! {
  static ref CONTEXT_ATTRIBUTES: Vec<c_int> = vec![
    GLX_CONTEXT_MAJOR_VERSION, 1,
    GLX_CONTEXT_MINOR_VERSION, 4,
    GLX_EXTRA_CONTEXT_FLAGS_ARB, GLX_EXTRA_CONTEXT_DEBUG_BIT_ARB,
    0,
  ];
}

#[cfg(not(debug_assertions))]
lazy_static! {
  // Unused with debug assertions off.
  static ref CONTEXT_ATTRIBUTES: Vec<c_int> = Vec::new();
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
        GLX_X_VISUAL_TYPE, GLX_TRUE_COLOR,
        GLX_RED_SIZE, 8,
        GLX_GREEN_SIZE, 8,
        GLX_BLUE_SIZE, 8,
        GLX_DEPTH_SIZE, 24,
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
          POINTER_MOTION_MASK | KEY_RELEASE_MASK | BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK |
          KEYMAP_STATE_MASK | FOCUS_CHANGE_MASK;
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
      create_context_attribs(display, config, ptr::null(), true, &CONTEXT_ATTRIBUTES)
    };

    set_debug_message_callback();

    let xwindow = XWindow {
      display: display,
      window: window,
      wm_delete_window: wm_delete_window,
      input_method: input_method,
      input_context: input_context,
      context: context,
      is_closed: AtomicBool::new(false),
      current_size: Cell::new((0, 0)),
      pending_events: Mutex::new(VecDeque::new()),
    };

    xwindow.make_current();
    xwindow
  }

  pub fn make_current(&self) {
    let rc = unsafe { glXMakeCurrent(self.display, self.window, self.context) };
    if rc == 0 {
      panic!("glXMakeCurrent() failed");
    }
  }

  pub fn is_closed(&self) -> bool {
    self.is_closed.load(Ordering::Relaxed)
  }

  pub fn flush(&self) {
    flush(self.display);
  }

  pub fn swap_buffers(&self) {
    unsafe { glXSwapBuffers(self.display, self.window); }
  }

  #[allow(dead_code)]
  pub fn wait_events(&self) -> WaitEventsIterator {
    WaitEventsIterator {
      window: self
    }
  }

  pub fn poll_events(&self) -> PollEventsIterator {
    PollEventsIterator {
      window: self
    }
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

fn create_color_map(display: *mut Display, window: Window, visual: *mut Visual, alloc: c_int) -> Colormap {
  unsafe {
    XCreateColormap(display, window, visual, alloc)
  }
}

fn create_window(display: *mut Display, parent: Window, x: c_int, y: c_int, width: c_uint, height: c_uint,
  depth: c_int, visual: *mut Visual, value_mask: c_ulong, attributes: *mut XSetWindowAttributes) -> Window {

  unsafe {
    XCreateWindow(display, parent, x, y, width, height, 0, depth, INPUT_OUTPUT, visual, value_mask, attributes)
  }
}

fn map_raised(display: *mut Display, window: Window) {
  unsafe { XMapRaised(display, window); };
}

fn flush(display: *mut Display) {
  unsafe { XFlush(display); };
}

fn store_name(display: *mut Display, window: Window, name: &str) {
  let c_name = CString::new(name).unwrap();
  unsafe {
    XStoreName(display, window, c_name.as_ptr());
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

fn set_wm_protocol(display: *mut Display, window: Window, protocol: Atom) {
  let mut protocol_mut = protocol;
  let status = unsafe {
    XSetWMProtocols(display, window, &mut protocol_mut, 1)
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

#[cfg(debug_assertions)]
fn create_context_attribs(display: *mut Display, config: GLXFBConfig, share_context: GLXContext,
  direct: bool, attrib_list: &[c_int]) -> GLXContext {

  // NOTE: No caching, OK to use since only called once.
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

#[cfg(not(debug_assertions))]
fn create_context_attribs(display: *mut Display, config: GLXFBConfig, share_context: GLXContext,
  direct: bool, _attrib_list: &[c_int]) -> GLXContext {

  let c_direct = if direct { 1 } else { 0 };
  let context = unsafe {
    glXCreateNewContext(display, config, GLX_RGBA_BIT, share_context, c_direct)
  };
  if context.is_null() {
    panic!("glXCreateNewContext() failed");
  }
  context
}

// Gl capability to enable:
const DEBUG_OUTPUT_SYNCHRONOUS_ARB: Enum = 0x8242;

#[cfg(debug_assertions)]
fn set_debug_message_callback() {
  // NOTE: No caching, OK to use since only called once.
  let debug_message_callback_arb_fn = get_proc_address("glDebugMessageCallbackARB");
  type DebugProcArb = extern "system" fn(Enum, Enum, UInt, Enum, SizeI, *const Char, *mut c_void);
  type DebugMessageCallbackArbFn = extern "system" fn(callback: DebugProcArb, user_param: *const c_void);
  let debug_message_callback_arb_fn = unsafe {
    mem::transmute::<_, DebugMessageCallbackArbFn>(debug_message_callback_arb_fn)
  };

  debug_message_callback_arb_fn(debug_message_callback, ptr::null());

  // Enable debug output.
  gl::enable(DEBUG_OUTPUT_SYNCHRONOUS_ARB);
}

// Values for source:
const DEBUG_SOURCE_API_ARB: Enum = 0x8246;
const DEBUG_SOURCE_WINDOW_SYSTEM_ARB: Enum = 0x8247;
const DEBUG_SOURCE_SHADER_COMPILER_ARB: Enum = 0x8248;
const DEBUG_SOURCE_THIRD_PARTY_ARB: Enum = 0x8249;
const DEBUG_SOURCE_APPLICATION_ARB: Enum = 0x824A;
const DEBUG_SOURCE_OTHER_ARB: Enum = 0x824B;

// Values for gl_type:
const DEBUG_TYPE_ERROR_ARB: Enum = 0x824C;
const DEBUG_TYPE_DEPRECATED_BEHAVIOR_ARB: Enum = 0x824D;
const DEBUG_TYPE_UNDEFINED_BEHAVIOR_ARB: Enum = 0x824E;
const DEBUG_TYPE_PORTABILITY_ARB: Enum = 0x824F;
const DEBUG_TYPE_PERFORMANCE_ARB: Enum = 0x8250;
const DEBUG_TYPE_OTHER_ARB: Enum = 0x8251;

// Values for severity:
const DEBUG_SEVERITY_HIGH_ARB: Enum = 0x9146;
const DEBUG_SEVERITY_MEDIUM_ARB: Enum = 0x9147;
const DEBUG_SEVERITY_LOW_ARB: Enum = 0x9148;

#[cfg(debug_assertions)]
extern "system" fn debug_message_callback(source: Enum, message_type: Enum, id: UInt, severity: Enum, _length: SizeI, message: *const Char, _user_param: *mut c_void) {

  let source = match source {
    DEBUG_SOURCE_API_ARB => "API".to_string(),
    DEBUG_SOURCE_WINDOW_SYSTEM_ARB => "Window system".to_string(),
    DEBUG_SOURCE_SHADER_COMPILER_ARB => "Shader compiler".to_string(),
    DEBUG_SOURCE_THIRD_PARTY_ARB => "Third party".to_string(),
    DEBUG_SOURCE_APPLICATION_ARB => "Application".to_string(),
    DEBUG_SOURCE_OTHER_ARB => "Other".to_string(),
    e => format!("Unknown source: {}", e),
  };

  let message_type = match message_type {
    DEBUG_TYPE_ERROR_ARB => "Error".to_string(),
    DEBUG_TYPE_DEPRECATED_BEHAVIOR_ARB => "Deprecated behavior".to_string(),
    DEBUG_TYPE_UNDEFINED_BEHAVIOR_ARB => "Undefined behavior".to_string(),
    DEBUG_TYPE_PORTABILITY_ARB => "Portability".to_string(),
    DEBUG_TYPE_PERFORMANCE_ARB => "Performance".to_string(),
    DEBUG_TYPE_OTHER_ARB => "Other".to_string(),
    e => format!("Unknown message type: {}", e),
  };

  let severity = match severity {
    DEBUG_SEVERITY_HIGH_ARB => "High severity".to_string(),
    DEBUG_SEVERITY_MEDIUM_ARB => "Medium severity".to_string(),
    DEBUG_SEVERITY_LOW_ARB => "Low severity".to_string(),
    e => format!("Unknown severity: {}", e),
  };

  let message = unsafe {
    CStr::from_ptr(message).to_bytes()
  };
  let message = str::from_utf8(message).unwrap_or("???");
  println!("----- {}: {} {} {}: {}", id, severity, source, message_type, message);
}

#[cfg(not(debug_assertions))]
fn set_debug_message_callback() {
  // Do nothing.
}

fn destroy_context(display: *mut Display, context: GLXContext) {
  unsafe { glXDestroyContext(display, context); }
}

fn peek_event(display: *mut Display, event: *mut XEvent) {
  unsafe { XPeekEvent(display, event) }
}

#[derive(Clone, Debug, Copy)]
pub enum Event {
  /// The size of the window has changed.
  Resized(u32, u32),

  /// The position of the window has changed.
  Moved(i32, i32),

  /// The window has been closed.
  Closed,

  /// The window received a unicode character.
  ReceivedCharacter(char),

  /// The window gained or lost focus.
  ///
  /// The parameter is true if the window has gained focus, and false if it has lost focus.
  Focused(bool),

  /// An event from the keyboard has been received.
  KeyboardInput(ElementState, ScanCode, Option<VirtualKeyCode>),

  /// The cursor has moved on the window.
  ///
  /// The parameter are the (x,y) coords in pixels relative to the top-left corner of the window.
  MouseMoved((i32, i32)),

  /// A positive value indicates that the wheel was rotated forward, away from the user;
  ///  a negative value indicates that the wheel was rotated backward, toward the user.
  MouseWheel(i32),

  /// An event from the mouse has been received.
  MouseInput(ElementState, MouseButton),

  /// The event loop was woken up by another thread.
  Awakened,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
  Left,
  Right,
  Middle,
  Other(u8),
}

#[allow(dead_code)]
pub struct WaitEventsIterator<'a> {
  window: &'a XWindow,
}

impl<'a> Iterator for WaitEventsIterator<'a> {
  type Item = Event;

  fn next(&mut self) -> Option<Event> {
    loop {
      if self.window.is_closed() {
        return None;
      }

      // This will block until an event arrives, but will not remove it from the queue.
      let mut x_event = unsafe { mem::uninitialized() };
      peek_event(self.window.display, &mut x_event);

      // Call poll_events() to get the event from the queue.
      if let Some(ev) = self.window.poll_events().next() {
        return Some(ev);
      }
    }
  }
}

pub struct PollEventsIterator<'a> {
  window: &'a XWindow,
}

impl<'a> Iterator for PollEventsIterator<'a> {
  type Item = Event;

  fn next(&mut self) -> Option<Event> {
    if let Some(ev) = self.window.pending_events.lock().unwrap().pop_front() {
      return Some(ev);
    }

    'event: loop {
      let x_event = match get_x_event(self.window.display) {
        Some(e) => e,
        None => return None,
      };

      match x_event.type_ {
        KEYMAP_NOTIFY => {
          println!("X11 event: KEYMAP_NOTIFY");
          refresh_keyboard_mapping(&x_event);
        },

        CLIENT_MESSAGE => {
          println!("X11 event: CLIENT_MESSAGE");
          let client_msg: &XClientMessageEvent = unsafe { mem::transmute(&x_event) };

          if client_msg.l[0] == self.window.wm_delete_window as c_long {
            self.window.is_closed.store(true, Ordering::Relaxed);
            return Some(Event::Closed);
          } else {
            return Some(Event::Awakened);
          }
        },

        CONFIGURE_NOTIFY => {
          println!("X11 event: CONFIGURE_NOTIFY");
          let cfg_event: &XConfigureEvent = unsafe { mem::transmute(&x_event) };
          let (current_width, current_height) = self.window.current_size.get();
          if current_width != cfg_event.width || current_height != cfg_event.height {
            self.window.current_size.set((cfg_event.width, cfg_event.height));
            return Some(Event::Resized(cfg_event.width as u32, cfg_event.height as u32));
          }
        },

        MOTION_NOTIFY => {
//          println!("X11 event: MOTION_NOTIFY");
          let event: &XMotionEvent = unsafe { mem::transmute(&x_event) };
          return Some(Event::MouseMoved((event.x as i32, event.y as i32)));
        },

        FOCUS_IN | FOCUS_OUT => {
          println!("X11 event: FOCUS_IN or FOCUS_OUT");
          let focused = x_event.type_ == FOCUS_IN;
          return Some(Event::Focused(focused));
        }

        KEY_PRESS | KEY_RELEASE => {
          println!("X11 event: KEY_PRESS or KEY_RELEASE");
          let event: &mut XKeyEvent = unsafe { mem::transmute(&x_event) };

          if event.type_ == KEY_PRESS {
            if filter_event(&x_event, self.window.window) {
              continue 'event;
            }
          }

          let state = if x_event.type_ == KEY_PRESS {
            ElementState::Pressed
          } else {
            ElementState::Released
          };

          let written = utf8_lookup_string(self.window.input_context, &event);
          {
            let mut pending = self.window.pending_events.lock().unwrap();
            for chr in written.chars() {
              pending.push_back(Event::ReceivedCharacter(chr));
            }
          }

          let keysym = keycode_to_keysym(self.window.display, event.keycode as KeyCode, 0);
          let vkey = key::keycode_to_element(keysym as c_uint);

          return Some(Event::KeyboardInput(state, event.keycode as u8, vkey));
        },

        BUTTON_PRESS | BUTTON_RELEASE => {
          println!("X11 event: BUTTON_PRESS or BUTTON_RELEASE");
          let event: &XButtonEvent = unsafe { mem::transmute(&x_event) };

          let state = if x_event.type_ == BUTTON_PRESS {
            ElementState::Pressed
          } else {
            ElementState::Released
          };

          let button = match event.button {
            BUTTON1 => Some(MouseButton::Left),
            BUTTON2 => Some(MouseButton::Middle),
            BUTTON3 => Some(MouseButton::Right),
            BUTTON4 => {
              self.window.pending_events.lock().unwrap().push_back(Event::MouseWheel(1));
              None
            }
            BUTTON5 => {
              self.window.pending_events.lock().unwrap().push_back(Event::MouseWheel(-1));
              None
            }
            _ => None
          };

          match button {
            Some(button) => return Some(Event::MouseInput(state, button)),
            None => ()
          };
        },

        _ => {
          println!("X11 event of unhandled type {}", x_event.type_);
          ()
        }
      };
    }
  }
}

fn get_x_event(display: *mut Display) -> Option<XEvent> {
  let mut x_event = unsafe { mem::uninitialized() };
  let rc = unsafe { XCheckMaskEvent(display, -1, &mut x_event) };
  if rc != 0 {
    return Some(x_event);
  }

  // Functions with mask arguments don't return non-maskable events (MappingNotify, Selection
  // events, CLIENT_MESSAGE), have to query them by type.
  let rc = unsafe { XCheckTypedEvent(display, CLIENT_MESSAGE, &mut x_event) };
  if rc != 0 {
    return Some(x_event);
  } else {
    return None;
  }
}

fn refresh_keyboard_mapping(event_map: &XEvent) {
  unsafe { XRefreshKeyboardMapping(event_map); }
}

fn filter_event(event: &XEvent, window: Window) -> bool {
  let rc = unsafe { XFilterEvent(mem::transmute(event as *const XEvent), window) };
  rc != 0
}

fn utf8_lookup_string(ic: XIC, event: &XKeyEvent) -> String {
  let mut buffer: [u8; 16] = unsafe { [mem::uninitialized(); 16] };
  let count = unsafe {
    Xutf8LookupString(ic, mem::transmute(event as *const XKeyEvent),
      mem::transmute(buffer.as_mut_ptr()), buffer.len() as c_int, ptr::null_mut(), ptr::null_mut())
  };
  str::from_utf8(&buffer[..count as usize]).unwrap_or("").to_string()
}

fn keycode_to_keysym(display: *mut Display, keycode: KeyCode, index: usize) -> KeySym {
  unsafe { XKeycodeToKeysym(display, keycode, index as i32) }
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
const FOCUS_CHANGE_MASK: c_long = (1<<21);

// For XCreateWindow() parameter valuemask:
const CW_BORDER_PIXEL: c_ulong = (1<<3);
const CW_EVENT_MASK: c_ulong = (1<<11);
const CW_COLORMAP: c_ulong = (1<<13);

// For XCreateWindow() parameter class:
const INPUT_OUTPUT: c_uint = 1;

// Values for XEvent.type_:
const KEY_PRESS: c_int = 2;
const KEY_RELEASE: c_int = 3;
const BUTTON_PRESS: c_int = 4;
const BUTTON_RELEASE: c_int = 5;
const MOTION_NOTIFY: c_int = 6;
const FOCUS_IN: c_int = 9;
const FOCUS_OUT: c_int = 10;
const KEYMAP_NOTIFY: c_int = 11;
const CONFIGURE_NOTIFY: c_int = 22;
const CLIENT_MESSAGE: c_int = 33;

#[repr(C)]
struct XEvent {
  type_: c_int,
  pad: [c_long; 24],
}

#[repr(C)]
struct XClientMessageEvent {
  type_: c_int,
  serial: c_ulong,
  send_event: Bool,
  display: *mut Display,
  window: Window,
  message_type: Atom,
  format: c_int,
  l: [c_long; 5],
}

#[repr(C)]
struct XConfigureEvent {
  type_: c_int,
  serial: c_ulong,
  send_event: Bool,
  display: *mut Display,
  event: Window,
  window: Window,
  x: c_int,
  y: c_int,
  width: c_int,
  height: c_int,
  border_width: c_int,
  above: Window,
  override_redirect: Bool,
}

#[repr(C)]
struct XMotionEvent {
  type_: c_int,
  serial: c_ulong,
  send_event: Bool,
  display: *mut Display,
  window: Window,
  root: Window,
  subwindow: Window,
  time: Time,
  x: c_int,
  y: c_int,
  x_root: c_int,
  y_root: c_int,
  state: c_uint,
  is_hint: c_char,
  same_screen: Bool,
}

#[repr(C)]
struct XKeyEvent {
  type_: c_int,
  serial: c_ulong,
  send_event: Bool,
  display: *mut Display,
  window: Window,
  root: Window,
  subwindow: Window,
  time: Time,
  x: c_int,
  y: c_int,
  x_root: c_int,
  y_root: c_int,
  state: c_uint,
  keycode: c_uint,
  same_screen: Bool,
}

// Values for XButtonEvent.button:
const BUTTON1: c_uint = 1;
const BUTTON2: c_uint = 2;
const BUTTON3: c_uint = 3;
const BUTTON4: c_uint = 4;
const BUTTON5: c_uint = 5;

#[repr(C)]
struct XButtonEvent {
  type_: c_int,
  serial: c_ulong,
  send_event: Bool,
  display: *mut Display,
  window: Window,
  root: Window,
  subwindow: Window,
  time: Time,
  x: c_int,
  y: c_int,
  x_root: c_int,
  y_root: c_int,
  state: c_uint,
  button: c_uint,
  same_screen: Bool,
}

#[link(name = "X11")]
extern "C" {
  fn XCheckMaskEvent(display: *mut Display, event_mask: c_long, event_return: *mut XEvent) -> Bool;
  fn XCheckTypedEvent(display: *mut Display, event_type: c_int, event_return: *mut XEvent) -> Bool;
  fn XCloseDisplay(display: *mut Display);
  fn XCloseIM(im: XIM) -> Status;
  fn XCreateColormap(display: *mut Display, window: Window, visual: *mut Visual, alloc: c_int) -> Colormap;
  // This is a vararg function.
  fn XCreateIC(im: XIM, a: *const c_char, b: c_long, c: *const c_char, d: Window, e: *const ()) -> XIC;
  fn XCreateWindow(display: *mut Display, parent: Window, x: c_int, y: c_int, width: c_uint, height: c_uint,
    border_width: c_uint, depth: c_int, class: c_uint, visual: *mut Visual, valuemask: c_ulong,
    attributes: *mut XSetWindowAttributes) -> Window;
  fn XDefaultRootWindow(display: *mut Display) -> Window;
  fn XDefaultScreen(display: *mut Display) -> c_int;
  fn XDestroyIC(ic: XIC);
  fn XDestroyWindow(display: *mut Display, window: Window);
  fn XFilterEvent(event: *mut XEvent, window: Window) -> Bool;
  fn XFlush(display: *mut Display);
  fn XFree(data: *const c_void);
  fn XInitThreads() -> Status;
  fn XInternAtom(display: *mut Display, atom_name: *const c_char, only_if_exists: Bool) -> Atom;
  fn XkbSetDetectableAutoRepeat(display: *mut Display, detectable: Bool, supported_rtm: *mut Bool) -> Bool;
  fn XKeycodeToKeysym(display: *mut Display, keycode: KeyCode, index: c_int) -> KeySym;
  fn XMapRaised(display: *mut Display, window: Window);
  fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
  fn XOpenIM(display: *mut Display, db: XrmDatabase, res_name: *mut c_char, res_class: *mut c_char) -> XIM;
  fn XPeekEvent(display: *mut Display, event: *mut XEvent);
  fn XRefreshKeyboardMapping(event_map: *const XEvent);
  fn XSetErrorHandler(callback: extern "C" fn(display: *mut Display, event: *mut XErrorEvent) -> c_int) -> c_int;
  fn XSetICFocus(ic: XIC);
  fn XSetWMProtocols(display: *mut Display, window: Window, protocols: *mut Atom, count: c_int) -> Status;
  fn XStoreName(display: *mut Display, window: Window, window_name: *const c_char);
  fn Xutf8LookupString(ic: XIC, event: *mut XKeyEvent, buffer_return: *mut c_char, bytes_buffer: c_int,
    keysym_return: *mut KeySym, status_return: *mut Status) -> c_int;
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
  #[cfg(not(debug_assertions))]
  fn glXCreateNewContext(display: *mut Display, config: GLXFBConfig, render_type: c_int, share_list: GLXContext, direct: Bool) -> GLXContext;
  fn glXDestroyContext(display: *mut Display, context: GLXContext);
  fn glXGetFBConfigAttrib(display: *mut Display, config: GLXFBConfig, attribute: c_int, value: *mut c_int) -> c_int;
  fn glXGetProcAddress(proc_name: *const GLubyte) -> Option<GLXextFuncPtr>;
  fn glXGetVisualFromFBConfig(display: *mut Display, config: GLXFBConfig) -> *mut XVisualInfo;
  fn glXMakeCurrent(display: *mut Display, drawable: GLXDrawable, context: GLXContext) -> Bool;
  fn glXSwapBuffers(display: *mut Display, drawable: GLXDrawable);
}
