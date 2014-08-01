LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)

# Compatibility shim for librustpng
LOCAL_MODULE := shim
LOCAL_C_INCLUDES := external/libpng-android/jni/
LOCAL_SRC_FILES := rust-png/shim.c
include $(BUILD_STATIC_LIBRARY)
include $(CLEAR_VARS)
