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
