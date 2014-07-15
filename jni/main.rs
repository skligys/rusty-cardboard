extern crate libc;

#[no_mangle]
pub extern fn rusty_android() -> libc::c_int {
  43 as libc::c_int
}
