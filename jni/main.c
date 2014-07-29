#include <android/asset_manager.h>
#include <android/log.h>
#include <android_native_app_glue.h>
#include <inttypes.h>
#include <jni.h>

#define LOGI(...) ((void)__android_log_print(ANDROID_LOG_INFO, "native-activity", __VA_ARGS__))

void c_log_string(int priority, const char *message) {
  __android_log_write(priority, "native-activity", message);
}

int32_t c_attach_current_thread_to_jvm(JavaVM *jvm) {
  JNIEnv *env = NULL;
  return (*jvm)->AttachCurrentThread(jvm, &env, NULL);
}

int32_t c_detach_current_thread_from_jvm(JavaVM *jvm) {
  return (*jvm)->DetachCurrentThread(jvm);
}

int32_t c_load_asset(AAssetManager *assetManager, const char *filename, off_t *outLength, unsigned char **outBuff) {
  AAsset *asset = AAssetManager_open(assetManager, filename, AASSET_MODE_STREAMING);
  if (!asset) {
    return -1;
  }

  off_t len = AAsset_getLength(asset);

  const unsigned char *buff = NULL;
  buff = AAsset_getBuffer(asset);
  if (!buff) {
    AAsset_close(asset);
    return -2;
  }

  // Closing the asset invalidates the buffer, copy it over.
  *outBuff = malloc(len);
  if (!outBuff) {
    AAsset_close(asset);
    return -3;
  }

  memcpy(*outBuff, buff, len);
  *outLength = len;

  AAsset_close(asset);
  return 0;
}

/* Functions implemented in Rust. */
extern void rust_android_main(struct android_app* app);

/**
 * This is the main entry point of a native application that is using android_native_app_glue.
 * It runs in its own thread, with its own event loop for receiving input events and doing other
 * things.
 *
 * TODO: Figure out why skipping this and implementing anroid_main in Rust leads to much smaller
 * libnative-activity.so, which cannot be loaded from Java.
 */
void android_main(struct android_app* app) {
  // Make sure glue isn't stripped.
  app_dummy();

  rust_android_main(app);
}
