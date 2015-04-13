use std::mem;

pub fn vertex_count() -> u32 {
  6 * 6
}

pub struct VertexArray {
  pub data: &'static [f32],
  pub components: u32,
  pub stride: u32,
}

pub fn vertex_coords() -> VertexArray {
  VertexArray {
    data: unsafe { mem::transmute(&VERTICES.front[0..]) },
    components: 3,
    stride: STRIDE,
  }
}

pub fn texture_coords() -> VertexArray {
  VertexArray {
    data: unsafe { mem::transmute(&VERTICES.front[3..]) },
    components: 2,
    stride: STRIDE,
  }
}

const NUMBERS_PER_VERTEX: u32 = 5;
const BYTES_PER_F32: u32 = 4;
const STRIDE: u32 = NUMBERS_PER_VERTEX * BYTES_PER_F32;

/// Each face consists of 2 triangles, 6 vertices listed sequentially in the
/// form: X, Y, Z (vertex coordinates), S, T (texture coordinates; note: T axis
/// is directed from top down).
struct CubeFaces {
  front: [f32; 30],
  left: [f32; 30],
  top: [f32; 30],
  back: [f32; 30],
  right: [f32; 30],
  bottom: [f32; 30],
}

static VERTICES: CubeFaces = CubeFaces {
  front: [
    -0.5, -0.5, 0.5,
    0.5, 1.0,

    0.5, -0.5, 0.5,
    1.0, 1.0,

    0.5, 0.5, 0.5,
    1.0, 0.5,

    0.5, 0.5, 0.5,
    1.0, 0.5,

    -0.5, 0.5, 0.5,
    0.5, 0.5,

    -0.5, -0.5, 0.5,
    0.5, 1.0,
  ],
  left: [
    -0.5, -0.5, -0.5,
    0.5, 1.0,

    -0.5, -0.5, 0.5,
    1.0, 1.0,

    -0.5, 0.5, 0.5,
    1.0, 0.5,

    -0.5, 0.5, 0.5,
    1.0, 0.5,

    -0.5, 0.5, -0.5,
    0.5, 0.5,

    -0.5, -0.5, -0.5,
    0.5, 1.0,
  ],
  top: [
    -0.5, 0.5, 0.5,
    0.0, 1.0,

    0.5, 0.5, 0.5,
    0.5, 1.0,

    0.5, 0.5, -0.5,
    0.5, 0.5,

    0.5, 0.5, -0.5,
    0.5, 0.5,

    -0.5, 0.5, -0.5,
    0.0, 0.5,

    -0.5, 0.5, 0.5,
    0.0, 1.0,
  ],
  back: [
    0.5, -0.5, -0.5,
    0.5, 1.0,

    -0.5, -0.5, -0.5,
    1.0, 1.0,

    -0.5, 0.5, -0.5,
    1.0, 0.5,

    -0.5, 0.5, -0.5,
    1.0, 0.5,

    0.5, 0.5, -0.5,
    0.5, 0.5,

    0.5, -0.5, -0.5,
    0.5, 1.0,
  ],
  right: [
    0.5, -0.5, 0.5,
    0.5, 1.0,

    0.5, -0.5, -0.5,
    1.0, 1.0,

    0.5, 0.5, -0.5,
    1.0, 0.5,

    0.5, 0.5, -0.5,
    1.0, 0.5,

    0.5, 0.5, 0.5,
    0.5, 0.5,

    0.5, -0.5, 0.5,
    0.5, 1.0,
  ],
  bottom: [
    0.5, -0.5, 0.5,
    0.0, 0.5,

    -0.5, -0.5, 0.5,
    0.5, 0.5,

    -0.5, -0.5, -0.5,
    0.5, 0.0,

    -0.5, -0.5, -0.5,
    0.5, 0.0,

    0.5, -0.5, -0.5,
    0.0, 0.0,

    0.5, -0.5, 0.5,
    0.0, 0.5,
  ],
};
