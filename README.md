rusty-cardboard
===============

A simple clone of Minecraft written in Rust to run on Google Cardboard.

This was inspired by a nice Minecraft-like demo written in Pyglet:
https://github.com/fogleman/Minecraft

I rewrote it in Java and OpenGL ES 2.0 in
https://github.com/skligys/cardboard-creeper
only to find that with a sufficiently complex and large landscape, generating
additional chunks and loading them in the background thread would make frames
stutter like there is no tomorrow (mostly because of garbage collection in ART).

This is an attempt to fix that by rewriting everything in Rust (fully native
code) and using hard floating point (which is not even available in Java).  The
goal is 60 FPS while loading and unloading chunks in the background on MotoX.

## Building the project
### Setup

* Download and install Android SDK and Android NDK.
* Configure Android SDK, to support version 19.
* Install Rust compiler for Android, here are instructions: https://github.com/rust-lang/rust/wiki/Doc-building-for-android
* In the root directory, create file `config.mk` containing locations of
Android SDK and NDK directories, Rust compiler, and also where to create the
standalone NDK, e.g.
```
ANDROID_SDK_HOME=/opt/android-sdk
ANDROID_NDK_HOME=/opt/android-ndk
ANDROID_NDK_STANDALONE_HOME=/home/user/android-ndk-standalone
RUSTC=/opt/rust/bin/rustc
```
* Run script `./configure`.  It will create (or delete and re-create) the
standalone NDK and the Android SDK project.

### Building

* Run `make`.  It will compile everything, upload the APK to your phone and run it.
