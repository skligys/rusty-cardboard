#include <android/log.h>
#include <android_native_app_glue.h>

void c_log_string(int priority, const char *message) {
  __android_log_write(priority, "native-activity", message);
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
