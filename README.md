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

This is an attempt to fix that by rewriting everything in Rust.  The goal is
60 FPS while loading and unloading chunks in the background on MotoX 2014.

## Building the project
### Setup

* Download and install Android SDK and Android NDK.  Configure SDK, to support platform version 21 (Lollipop).  Create a standalone NDK toolchain with `--platform=android-21` and `--toolchain=arm-linux-androideabi-4.9`, see "Standalone Toolchain" in NDK documentation.
* Install Rust cross-compiler for Android.  You will need to build from source, and it will sadly take 1-2 hours:
http://web.archive.org/web/20141225095654/https://github.com/rust-lang/rust/wiki/Doc-building-for-android
* Install `cargo`, a recent nightly build will do.
* Check out the project and update submodules:

    ```sh
    $ git clone https://github.com/skligys/rusty-cardboard.git
    $ cd rusty-cardboard
    $ cd external/apk-builder
    $ git checkout master
    $ cd ../..
    ```

### Building

* Build `apk-builder`:

    ```sh
    $ cd external/apk-builder/apk-builder
    $ cargo build --release --features "assets_hack"
    ```

* Build the project:

    ```sh
    $ ANDROID_HOME=<sdk dir> NDK_HOME=<ndk dir> NDK_STANDALONE=<standalone ndk dir> PATH=$NDK_STANDALONE/bin:$PATH cargo build --target=arm-linux-androideabi
    ```

* Install on your phone or emulator:

    ```sh
    $ mv target/arm-linux-androideabi/debug/RustyCardboard target/arm-linux-androideabi/debug/RustyCardboard.apk
    $ adb install -r target/arm-linux-androideabi/debug/RustyCardboard.apk
    ```
