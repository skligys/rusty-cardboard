.PHONY: clean deploy_debug deploy_release

include config.mk
ADB=$(ANDROID_SDK_HOME)/platform-tools/adb

deploy_debug: bin/RustyCardboard-debug.apk
	$(ADB) install -r $<
	$(ADB) shell am start -n com.example.native_activity/android.app.NativeActivity
	$(ADB) logcat | grep native-activity

deploy_release: bin/RustyCardboard-release.apk
	$(ADB) install -r $<
	$(ADB) shell am start -n com.example.native_activity/android.app.NativeActivity

bin/RustyCardboard-debug.apk: build.xml jni/main.c jni/Android.mk jni/Application.mk jni/librust.a
	$(ANDROID_NDK_HOME)/ndk-build
	ant debug

bin/RustyCardboard-release.apk: build.xml jni/main.c jni/Android.mk jni/Application.mk jni/librust.a
	$(ANDROID_NDK_HOME)/ndk-build
	ant release

jni/librust.a: jni/main.rs
	$(PRE_RUSTC) $(RUSTC) --target=arm-linux-androideabi jni/main.rs -C linker=$(ANDROID_NDK_STANDALONE_HOME)/bin/arm-linux-androideabi-gcc --crate-type=staticlib -o jni/librust.a

clean:
	rm -rf obj libs jni/*.a
