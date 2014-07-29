use libc::{c_char, c_int, c_void, int32_t, size_t};

use asset_manager;
use input;
use jni;
use native_window;
use sensor;

/// Opaque structure representing callbacks from the framework makes into a native application.
struct NativeActivityCallbacks;

/**
 * This structure defines the native side of an android.app.NativeActivity.  It is created by
 * the framework, and handed to the application's native code as it is being launched.
 */
pub struct NativeActivity {
  /**
   * Pointer to the callback function table of the native application.  You can set the functions
   * here to your own callbacks.  The callbacks pointer itself here should not be changed; it is
   * allocated and managed for you by the framework.
   */
  #[allow(dead_code)]
  callbacks: *const NativeActivityCallbacks,

  /// The global handle on the process's Java VM.
  pub vm: *const jni::JavaVm,

  /**
   * JNI context for the main thread of the app.  Note that this field can ONLY be used from
   * the main thread of the process; that is, the thread that calls into
   * the NativeActivityCallbacks.
   */
  #[allow(dead_code)]
  env: *const jni::JniEnv,

  /// The NativeActivity object handle.
  #[allow(dead_code)]
  activity: *const jni::Jobject,

  /// Path to this application's internal data directory.
  #[allow(dead_code)]
  pub internal_data_path: *const c_char,

  /// Path to this application's external (removable/mountable) data directory.
  #[allow(dead_code)]
  pub external_data_path: *const c_char,

  /// The platform's SDK version code.
  #[allow(dead_code)]
  pub sdk_version: int32_t,

  /**
   * This is the native instance of the application.  It is not used by the framework, but can be
   * set by the application to its own instance state.
   */
  #[allow(dead_code)]
  instance: *mut c_void,

  /**
   * Pointer to the Asset Manager instance for the application.  The application uses this to access
   * binary assets bundled inside its own .apk file.
   */
  pub asset_manager: *const asset_manager::AssetManager,

  /**
   * Available starting with Honeycomb: path to the directory containing the application's OBB files
   * (if any).  If the app doesn't have any OBB files, this directory may not exist.
   */
  #[allow(dead_code)]
  pub obb_path: *const c_char,
}


/// Opaque structure representing Android configuration.
struct Configuration;

struct Rect {
  #[allow(dead_code)]
  left: i32,
  #[allow(dead_code)]
  top: i32,
  #[allow(dead_code)]
  right: i32,
  #[allow(dead_code)]
  bottom: i32,
}

// This is the interface for the standard glue code of a threaded application.  In this model, the
// application's code is running in its own thread separate from the main thread of the process.
// It is not required that this thread be associated with the Java VM, although it will need to be
// in order to make JNI calls to any Java objects.  Compatible with C.
pub struct AndroidApp {
  // The application can place a pointer to its own state object here if it likes.
  pub user_data: *const c_void,
  // Fill this in with the function to process main app commands (APP_CMD_*)
  pub on_app_cmd: *const c_void,
  // Fill this in with the function to process input events.  At this point the event has already
  // been pre-dispatched, and it will be finished upon return.  Return 1 if you have handled
  // the event, 0 for any default dispatching.
  pub on_input_event: *const c_void,
  // The NativeActivity object instance that this app is running in.
  pub activity: *const NativeActivity,
  // The current configuration the app is running in.
  #[allow(dead_code)]
  config: *const Configuration,
  // This is the last instance's saved state, as provided at creation time.  It is NULL if there
  // was no state.  You can use this as you need; the memory will remain around until you call
  // android_app_exec_cmd() for APP_CMD_RESUME, at which point it will be freed and savedState
  // set to NULL.  These variables should only be changed when processing a APP_CMD_SAVE_STATE,
  // at which point they will be initialized to NULL and you can malloc your state and place
  // the information here.  In that case the memory will be freed for you later.
  pub saved_state: *mut c_void,
  pub saved_state_size: size_t,
  // The looper associated with the app's thread.
  pub looper: *const sensor::Looper,
  // When non-NULL, this is the input queue from which the app will receive user input events.
  #[allow(dead_code)]
  input_queue: *const input::Queue,
  // When non-NULL, this is the window surface that the app can draw in.
  pub window: *const native_window::NativeWindow,
  // Current content rectangle of the window; this is the area where the window's content should be
  // placed to be seen by the user.
  #[allow(dead_code)]
  content_rect: Rect,
  // Current state of the app's activity.  May be either APP_CMD_START, APP_CMD_RESUME,
  // APP_CMD_PAUSE, or APP_CMD_STOP; see below.
  #[allow(dead_code)]
  activity_state: c_int,
  // This is non-zero when the application's NativeActivity is being destroyed and waiting for
  // the app thread to complete.
  pub destroy_requested: c_int,
  // Plus some private implementation details.
}

// Native app glue command enums:
pub static CMD_INIT_WINDOW: int32_t = 1;
pub static CMD_TERM_WINDOW: int32_t = 2;
pub static CMD_GAINED_FOCUS: int32_t = 6;
pub static CMD_LOST_FOCUS: int32_t = 7;
pub static CMD_SAVE_STATE: int32_t = 12;

/**
 * Data associated with an Looper fd that will be returned as the "data" when that source has
 * data ready.
 */
pub struct AndroidPollSource {
  /// The identifier of this source.  May be LOOPER_ID_MAIN or LOOPER_ID_INPUT.
  #[allow(dead_code)]
  id: int32_t,
  /// The android_app this ident is associated with.
  #[allow(dead_code)]
  app: *const AndroidApp,
  /// Function to call to perform the standard processing of data from this source.
  pub process: extern "C" fn (app: *mut AndroidApp, source: *const AndroidPollSource),
}
