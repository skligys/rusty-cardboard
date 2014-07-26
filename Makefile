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

jni/librust.a: jni/*.rs jni/libcgmath.rlib
	$(PRE_RUSTC) $(RUSTC) --target=arm-linux-androideabi jni/main.rs -C linker=$(ANDROID_NDK_STANDALONE_HOME)/bin/arm-linux-androideabi-gcc --crate-type=staticlib --opt-level=3 -o jni/librust.a -L jni

jni/libcgmath.rlib: external/cgmath-rs/src/*.rs
	$(PRE_RUSTC) $(RUSTC) --target=arm-linux-androideabi external/cgmath-rs/src/cgmath.rs -C linker=$(ANDROID_NDK_STANDALONE_HOME)/bin/arm-linux-androideabi-gcc --crate-type=rlib --opt-level=3 -o jni/libcgmath.rlib

clean:
	rm -rf obj libs jni/*.a jni/*.rlib bin
