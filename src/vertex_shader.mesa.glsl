#version 130
uniform mat4 u_MVPMatrix;
attribute vec4 a_Position;
attribute vec2 a_TextureCoord;
varying vec2 v_TextureCoord;

void main() {
  v_TextureCoord = a_TextureCoord;
  gl_Position = u_MVPMatrix * a_Position;
}
