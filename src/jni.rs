use libc::{c_void, int32_t};
use std::ptr;

/// JNI invocation interface.
pub struct JavaVm {
  functions: *const JniInvokeInterface,
}

pub struct JniInvokeInterface {
  #[allow(dead_code)]
  reserved0: *const c_void,
  #[allow(dead_code)]
  reserved1: *const c_void,
  #[allow(dead_code)]
  reserved2: *const c_void,
  #[allow(dead_code)]
  destroy_java_vm: extern fn(*const JavaVm) -> int32_t,
  attach_current_thread: extern fn(*const JavaVm, *mut *const JniEnv, *const c_void) -> int32_t,
  detach_current_thread: extern fn(*const JavaVm) -> int32_t,
  #[allow(dead_code)]
  get_env: extern fn(*const JavaVm, *mut *const c_void, int32_t) -> int32_t,
  #[allow(dead_code)]
  attach_current_thread_as_daemon: extern fn(*const JavaVm, *mut *const JniEnv, *const c_void) -> int32_t,
}

/// Opaque structure for the JNI context.
pub struct JniEnv;

/// Opaque Java object handle.
pub struct Jobject;

pub fn attach_current_thread_to_jvm(jvm: *const JavaVm) -> int32_t {
  let mut env: *const JniEnv = ptr::null();
  let attach_current_thread = unsafe {
    (*(*jvm).functions).attach_current_thread
  };
  attach_current_thread(jvm, &mut env, ptr::null())
}

pub fn detach_current_thread_from_jvm(jvm: *const JavaVm) -> int32_t {
  let detach_current_thread = unsafe {
    (*(*jvm).functions).detach_current_thread
  };
  detach_current_thread(jvm)
}
