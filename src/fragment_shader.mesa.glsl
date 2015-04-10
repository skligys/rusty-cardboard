#version 130
uniform sampler2D u_TextureUnit;
varying vec2 v_TextureCoord;

void main() {
  gl_FragColor = texture2D(u_TextureUnit, v_TextureCoord);
}
