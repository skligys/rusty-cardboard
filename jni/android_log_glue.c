#include <android/log.h>

void c_logi_string(const char *msg) {
  __android_log_write(ANDROID_LOG_INFO, "native-activity", msg);
}

void c_logw_string(const char *msg) {
  __android_log_write(ANDROID_LOG_WARN, "native-activity", msg);
}
