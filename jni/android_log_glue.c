#include <android/log.h>

void c_log_string(int priority, const char *message) {
  __android_log_write(priority, "native-activity", message);
}
