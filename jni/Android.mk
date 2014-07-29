LOCAL_PATH := $(call my-dir)
include $(CLEAR_VARS)

# libpng
LOCAL_MODULE := libpng
LOCAL_SRC_FILES :=\
  ../external/libpng-android/jni/png.c \
  ../external/libpng-android/jni/pngerror.c \
  ../external/libpng-android/jni/pngget.c \
  ../external/libpng-android/jni/pngmem.c \
  ../external/libpng-android/jni/pngpread.c \
  ../external/libpng-android/jni/pngread.c \
  ../external/libpng-android/jni/pngrio.c \
  ../external/libpng-android/jni/pngrtran.c \
  ../external/libpng-android/jni/pngrutil.c \
  ../external/libpng-android/jni/pngset.c \
  ../external/libpng-android/jni/pngtrans.c \
  ../external/libpng-android/jni/pngwio.c \
  ../external/libpng-android/jni/pngwrite.c \
  ../external/libpng-android/jni/pngwtran.c \
  ../external/libpng-android/jni/pngwutil.c
include $(BUILD_STATIC_LIBRARY)
include $(CLEAR_VARS)

# Compatibility shim for librustpng
LOCAL_MODULE := pngshim
LOCAL_C_INCLUDES := external/libpng-android/jni/
LOCAL_SRC_FILES := ../external/rust-png/shim.c
include $(BUILD_STATIC_LIBRARY)
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
