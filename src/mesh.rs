use cgmath::Vector3;

/// Each face consists of 2 triangles, 6 vertices total.
pub struct CubeFace {
  /// Triangle vertices: X, Y, Z, X, Y, Z, ...
  pub vertices: [f32; 18],
  /// Texture coordinated: S, T, S, T, ...
  /// Note: T axis is directed from top down.
  pub texture_coords: [f32; 12],
  /// Translation vector to get the cube adjacent with the current cube on this face.
  pub direction: Vector3<i32>,
}

/// Cube faces in standard order: left, right, down, up, forward, back.
lazy_static! {
  pub static ref CUBE_FACES: [CubeFace; 6] = [
    // Left.
    CubeFace {
      vertices: [
        -0.5, -0.5, -0.5,
        -0.5, -0.5, 0.5,
        -0.5, 0.5, 0.5,
        -0.5, 0.5, 0.5,
        -0.5, 0.5, -0.5,
        -0.5, -0.5, -0.5,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        1.0, 0.5,
        0.5, 0.5,
        0.5, 1.0,
      ],
      direction: Vector3::new(-1, 0, 0),
    },
    // Right.
    CubeFace {
      vertices: [
        0.5, -0.5, 0.5,
        0.5, -0.5, -0.5,
        0.5, 0.5, -0.5,
        0.5, 0.5, -0.5,
        0.5, 0.5, 0.5,
        0.5, -0.5, 0.5,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        1.0, 0.5,
        0.5, 0.5,
        0.5, 1.0,
      ],
      direction: Vector3::new(1, 0, 0),
    },
    // Down.
    CubeFace {
      vertices: [
        0.5, -0.5, 0.5,
        -0.5, -0.5, 0.5,
        -0.5, -0.5, -0.5,
        -0.5, -0.5, -0.5,
        0.5, -0.5, -0.5,
        0.5, -0.5, 0.5,
      ],
      texture_coords: [
        0.0, 0.5,
        0.5, 0.5,
        0.5, 0.0,
        0.5, 0.0,
        0.0, 0.0,
        0.0, 0.5,
      ],
      direction: Vector3::new(0, -1, 0),
    },
    // Up.
    CubeFace {
      vertices: [
        -0.5, 0.5, 0.5,
        0.5, 0.5, 0.5,
        0.5, 0.5, -0.5,
        0.5, 0.5, -0.5,
        -0.5, 0.5, -0.5,
        -0.5, 0.5, 0.5,
      ],
      texture_coords: [
        0.0, 1.0,
        0.5, 1.0,
        0.5, 0.5,
        0.5, 0.5,
        0.0, 0.5,
        0.0, 1.0,
      ],
      direction: Vector3::new(0, 1, 0),
    },
    // Forward.
    CubeFace {
      vertices: [
        0.5, -0.5, -0.5,
        -0.5, -0.5, -0.5,
        -0.5, 0.5, -0.5,
        -0.5, 0.5, -0.5,
        0.5, 0.5, -0.5,
        0.5, -0.5, -0.5,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        1.0, 0.5,
        0.5, 0.5,
        0.5, 1.0,
      ],
      direction: Vector3::new(0, 0, -1),
    },
    // Back.
    CubeFace {
      vertices: [
        -0.5, -0.5, 0.5,
        0.5, -0.5, 0.5,
        0.5, 0.5, 0.5,
        0.5, 0.5, 0.5,
        -0.5, 0.5, 0.5,
        -0.5, -0.5, 0.5,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        1.0, 0.5,
        0.5, 0.5,
        0.5, 1.0,
      ],
      direction: Vector3::new(0, 0, 1),
    },
  ];
}
