.PHONY: clean deploy_debug deploy_release rust apk

include config.mk
ADB=$(ANDROID_SDK_HOME)/platform-tools/adb

deploy_debug: bin/RustyCardboard-debug.apk
	$(ADB) install -r $<
	$(ADB) shell am start -n com.example.native_activity/android.app.NativeActivity
	$(ADB) logcat | grep native-activity

deploy_release: bin/RustyCardboard-release.apk
	$(ADB) install -r $<
	$(ADB) shell am start -n com.example.native_activity/android.app.NativeActivity

apk: bin/RustyCardboard-debug.apk

bin/RustyCardboard-debug.apk: build.xml jni/main.c jni/Android.mk jni/Application.mk jni/librust.a
	$(ANDROID_NDK_HOME)/ndk-build
	ant debug

bin/RustyCardboard-release.apk: build.xml jni/main.c jni/Android.mk jni/Application.mk jni/librust.a
	$(ANDROID_NDK_HOME)/ndk-build
	ant release

rust: jni/librust.a

jni/librust.a: jni/*.rs jni/libcgmath.rlib jni/libpng.rlib
	$(PRE_RUSTC) $(RUSTC) --target=arm-linux-androideabi jni/main.rs -C linker=$(ANDROID_NDK_STANDALONE_HOME)/bin/arm-linux-androideabi-gcc --crate-type=staticlib --opt-level=3 -o jni/librust.a -L jni
  # WTH, r-compiler-rt-divsi3.o and r-compiler-rt-aeabi_idivmod.o in librust.a conflict with
  # _divsi3.o and aeabi_idivmod.o in ligcc.a!
	$(ANDROID_NDK_HOME)/toolchains/arm-linux-androideabi-4.8/prebuilt/linux-x86_64/bin/arm-linux-androideabi-ar d jni/librust.a r-compiler-rt-divsi3.o r-compiler-rt-aeabi_idivmod.o

jni/libcgmath.rlib: external/cgmath-rs/src/*.rs
	$(PRE_RUSTC) $(RUSTC) --target=arm-linux-androideabi external/cgmath-rs/src/cgmath.rs -C linker=$(ANDROID_NDK_STANDALONE_HOME)/bin/arm-linux-androideabi-gcc --crate-type=rlib --opt-level=3 -o jni/libcgmath.rlib

jni/libpng.rlib: external/rust-png/*.rs jni/libpng.a jni/libshim.a
	$(PRE_RUSTC) $(RUSTC) --target=arm-linux-androideabi external/rust-png/lib.rs -C linker=$(ANDROID_NDK_STANDALONE_HOME)/bin/arm-linux-androideabi-gcc --crate-type=rlib --opt-level=3 -o jni/libpng.rlib -L jni

jni/libpng.a: external/libpng-android/jni/*.c external/libpng-android/jni/*.h
	$(ANDROID_NDK_HOME)/ndk-build APP_BUILD_SCRIPT=external/libpng-android/jni/Android.mk
	cp obj/local/armeabi/libpng.a jni/

jni/libshim.a: external/rust-png/shim.c external/libpng-android/jni/png.h
	$(ANDROID_NDK_HOME)/ndk-build APP_BUILD_SCRIPT=external/Android.mk
	cp obj/local/armeabi/libshim.a jni/

clean:
	rm -rf obj libs jni/*.a jni/*.rlib bin
