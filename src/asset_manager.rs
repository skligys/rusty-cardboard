extern crate android_glue;

use std::ffi::CString;
use libc::int32_t;

struct AssetCloser {
  asset: *mut android_glue::ffi::AAsset,
}

impl Drop for AssetCloser {
  fn drop(&mut self) {
    unsafe { android_glue::ffi::AAsset_close(self.asset) };
  }
}

pub fn load_asset(manager: &mut android_glue::ffi::AAssetManager, filename: &str) -> Result<Vec<u8>, int32_t> {
  let filename_c_str = CString::new(filename).unwrap();
  let asset = unsafe {
    android_glue::ffi::AAssetManager_open(manager, filename_c_str.as_ptr(), android_glue::ffi::AASSET_MODE_STREAMING)
  };
  if asset.is_null() {
    return Err(-1);
  }
  let _asset_closer = AssetCloser { asset: asset };

  let len = unsafe { android_glue::ffi::AAsset_getLength(asset) };
  let buff = unsafe { android_glue::ffi::AAsset_getBuffer(asset) };
  if buff.is_null() {
    return Err(-2);
  }
  let vec = unsafe {
    Vec::from_raw_buf(buff as *const u8, len as usize)
  };
  Ok(vec)
}
