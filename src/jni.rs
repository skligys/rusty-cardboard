extern crate android_glue;

use std::ptr;

pub fn attach_current_thread_to_jvm(jvm: &mut android_glue::ffi::JavaVM) -> i32 {
  let mut env: *mut android_glue::ffi::JNIEnv = ptr::null_mut();
  let attach_current_thread = unsafe {
    (*(*jvm).functions).AttachCurrentThread
  };
  attach_current_thread(jvm, &mut env, ptr::null_mut())
}

pub fn detach_current_thread_from_jvm(jvm: &mut android_glue::ffi::JavaVM) -> i32 {
  let detach_current_thread = unsafe {
    (*(*jvm).functions).DetachCurrentThread
  };
  detach_current_thread(jvm)
}
