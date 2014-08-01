LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)

# Already compiled static library containing Rust code.
LOCAL_MODULE := rust-prebuilt
LOCAL_SRC_FILES := librust.a
include $(PREBUILT_STATIC_LIBRARY)
include $(CLEAR_VARS)

# A workaround for missing symbols: _Unwind_GetIP, _Unwind_SetIP, _Unwind_SetGR.
LOCAL_MODULE := unwind
LOCAL_C_INCLUDES := $(LOCAL_PATH)/unwind/include
LOCAL_SRC_FILES := unwind/unwind.c
include $(BUILD_STATIC_LIBRARY)
include $(CLEAR_VARS)

# Main
LOCAL_MODULE := native-activity
LOCAL_SRC_FILES := main.c
LOCAL_LDLIBS := -llog -landroid -lEGL -lGLESv2 -lz
LOCAL_STATIC_LIBRARIES := rust-prebuilt android_native_app_glue unwind pngshim png
include $(BUILD_SHARED_LIBRARY)

$(call import-module,android/native_app_glue)
