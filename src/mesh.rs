use cgmath::Vector3;

/// Each face consists of 2 triangles, 4 vertices total.
pub struct CubeFace {
  /// Position coordinates: X, Y, Z, X, Y, Z, ...
  pub position_coords: [f32; 12],
  /// Position indices, point to position coordinates to form counter-clockwise
  /// triangles.
  pub position_indices: [u16; 6],
  /// Texture coordinates: S, T, S, T, ...
  /// Note: T axis is directed from top down.
  pub texture_coords: [f32; 8],
  /// Translation vector to get the cube adjacent with the current cube on this face.
  pub direction: Vector3<i32>,
}

/// Cube faces in standard order: left, right, down, up, forward, back.
lazy_static! {
  pub static ref CUBE_FACES: [CubeFace; 6] = [
    // Left.
    CubeFace {
      position_coords: [
        -0.5, -0.5, -0.5,
        -0.5, -0.5,  0.5,
        -0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
      ],
      position_indices: [
        0, 1, 3,
        3, 1, 2,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        0.5, 0.5,
      ],
      direction: Vector3::new(-1, 0, 0),
    },
    // Right.
    CubeFace {
      position_coords: [
        0.5, -0.5,  0.5,
        0.5, -0.5, -0.5,
        0.5,  0.5, -0.5,
        0.5,  0.5,  0.5,
      ],
      position_indices: [
        0, 1, 3,
        3, 1, 2,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        0.5, 0.5,
      ],
      direction: Vector3::new(1, 0, 0),
    },
    // Down.
    CubeFace {
      position_coords: [
        -0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
         0.5, -0.5,  0.5,
        -0.5, -0.5,  0.5,
      ],
      position_indices: [
        0, 1, 3,
        3, 1, 2,
      ],
      texture_coords: [
        0.0, 0.5,
        0.5, 0.5,
        0.5, 0.0,
        0.0, 0.0,
      ],
      direction: Vector3::new(0, -1, 0),
    },
    // Up.
    CubeFace {
      position_coords: [
        -0.5, 0.5,  0.5,
         0.5, 0.5,  0.5,
         0.5, 0.5, -0.5,
        -0.5, 0.5, -0.5,
      ],
      position_indices: [
        0, 1, 3,
        3, 1, 2,
      ],
      texture_coords: [
        0.0, 1.0,
        0.5, 1.0,
        0.5, 0.5,
        0.0, 0.5,
     ],
      direction: Vector3::new(0, 1, 0),
    },
    // Forward.
    CubeFace {
      position_coords: [
         0.5, -0.5, -0.5,
        -0.5, -0.5, -0.5,
        -0.5,  0.5, -0.5,
         0.5,  0.5, -0.5,
      ],
      position_indices: [
        0, 1, 3,
        3, 1, 2,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        0.5, 0.5,
      ],
      direction: Vector3::new(0, 0, -1),
    },
    // Back.
    CubeFace {
      position_coords: [
        -0.5, -0.5, 0.5,
         0.5, -0.5, 0.5,
         0.5,  0.5, 0.5,
        -0.5,  0.5, 0.5,
      ],
      position_indices: [
        0, 1, 3,
        3, 1, 2,
      ],
      texture_coords: [
        0.5, 1.0,
        1.0, 1.0,
        1.0, 0.5,
        0.5, 0.5,
      ],
      direction: Vector3::new(0, 0, 1),
    },
  ];
}
