use libc::{c_char, c_int, c_void, int32_t, off_t};
use std::vec::raw;

/// Opaque structure representing asset manager.
pub struct AssetManager;

/// Opaque structure representing an asset.
pub struct Asset;

// Modes for opening assets:
#[allow(dead_code)]
static MODE_UNKNOWN: c_int = 0;
#[allow(dead_code)]
static MODE_RANDOM: c_int = 1;
static MODE_STREAMING: c_int = 2;
#[allow(dead_code)]
static MODE_BUFFER: c_int = 3;

struct AssetCloser {
  asset: *const Asset,
}

impl Drop for AssetCloser {
  fn drop(&mut self) {
    unsafe { AAsset_close(self.asset) };
  }
}

pub fn load_asset(manager: &AssetManager, filename: &str) -> Result<Vec<u8>, int32_t> {
  let filename_c_str = filename.to_c_str();
  let asset = unsafe {
    AAssetManager_open(manager, filename_c_str.as_ptr(), MODE_STREAMING)
  };
  if asset.is_null() {
    return Err(-1);
  }
  let _asset_closer = AssetCloser { asset: asset };

  let len = unsafe { AAsset_getLength(asset) };
  let buff = unsafe { AAsset_getBuffer(asset) };
  if buff.is_null() {
    return Err(-2);
  }
  let vec = unsafe {
    raw::from_buf(buff as *const u8, len as uint)
  };
  Ok(vec)
}

extern {
  fn AAssetManager_open(manager: *const AssetManager, filename: *const c_char, mode: c_int) -> *const Asset;
  fn AAsset_getLength(asset: *const Asset) -> off_t;
  fn AAsset_getBuffer(asset: *const Asset) -> *const c_void;
  fn AAsset_close(asset: *const Asset);
}
