use cgmath::Vector3;

/// This has to have C layout since it is read by the OOpenGL driver via a pointer passed to it.
#[repr(C)]
#[derive(Clone)]
pub struct Coords {
  pub xyz: [f32; 3],
  pub st: [f32; 2]
}

impl Coords {
  pub fn size_bytes() -> u32 {
    5 * 4
  }

  pub fn translate(&self, x: f32, y: f32, z: f32) -> Coords {
    Coords {
      xyz: [
        self.xyz[0] + x,
        self.xyz[1] + y,
        self.xyz[2] + z,
      ],
      st: self.st,
    }
  }
}

/// Each face consists of 2 triangles, 4 vertices total.
pub struct CubeFace {
  /// Position and texture coordinates.
  pub coords: [Coords; 4],
  /// Translation vector to get the cube adjacent with the current cube on this face.
  pub direction: Vector3<i32>,
}

/// Cube faces in standard order: left, right, down, up, forward, back.
lazy_static! {
  // Position indices, point to face coordinates to form 2 counter-clockwise triangles.
  pub static ref INDICES: [u16; 6] = [
    0, 1, 3,
    3, 1, 2,
  ];

  // 6 cube faces in canonical order: left, right, down, up, forward, back.
  pub static ref CUBE_FACES: [CubeFace; 6] = [
    // Left.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, -0.5, -0.5],
          st: [0.5, 1.0],
        },
        Coords {
          xyz: [-0.5, -0.5, 0.5],
          st: [1.0, 1.0],
        },
        Coords {
          xyz: [-0.5, 0.5, 0.5],
          st: [1.0, 0.5],
        },
        Coords {
          xyz: [-0.5, 0.5, -0.5],
          st: [0.5, 0.5],
        },
      ],
      direction: Vector3::new(-1, 0, 0),
    },
    // Right.
    CubeFace {
      coords: [
        Coords {
          xyz: [0.5, -0.5, 0.5],
          st: [0.5, 1.0],
        },
        Coords {
          xyz: [0.5, -0.5, -0.5],
          st: [1.0, 1.0],
        },
        Coords {
          xyz: [0.5, 0.5, -0.5],
          st: [1.0, 0.5],
        },
        Coords {
          xyz: [0.5, 0.5, 0.5],
          st: [0.5, 0.5],
        },
      ],
      direction: Vector3::new(1, 0, 0),
    },
    // Down.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, -0.5, -0.5],
          st: [0.0, 0.5],
        },
        Coords {
          xyz: [0.5, -0.5, -0.5],
          st: [0.5, 0.5],
        },
        Coords {
          xyz: [0.5, -0.5, 0.5],
          st: [0.5, 0.0],
        },
        Coords {
          xyz: [-0.5, -0.5,  0.5],
          st: [0.0, 0.0],
        },
      ],
      direction: Vector3::new(0, -1, 0),
    },
    // Up.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, 0.5, 0.5],
          st: [0.0, 1.0],
        },
        Coords {
          xyz: [0.5, 0.5,  0.5],
          st: [0.5, 1.0],
        },
        Coords {
          xyz: [0.5, 0.5, -0.5],
          st: [0.5, 0.5],
        },
        Coords {
          xyz: [-0.5, 0.5, -0.5],
          st: [0.0, 0.5],
        },
      ],
      direction: Vector3::new(0, 1, 0),
    },
    // Forward.
    CubeFace {
      coords: [
        Coords {
          xyz: [0.5, -0.5, -0.5],
          st: [0.5, 1.0],
        },
        Coords {
          xyz: [-0.5, -0.5, -0.5],
          st: [1.0, 1.0],
        },
        Coords {
          xyz: [-0.5, 0.5, -0.5],
          st: [1.0, 0.5],
        },
        Coords {
          xyz: [0.5,  0.5, -0.5],
          st: [0.5, 0.5],
        },
      ],
      direction: Vector3::new(0, 0, -1),
    },
    // Back.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, -0.5, 0.5],
          st: [0.5, 1.0],
        },
        Coords {
          xyz: [0.5, -0.5, 0.5],
          st: [1.0, 1.0],
        },
        Coords {
          xyz: [0.5, 0.5, 0.5],
          st: [1.0, 0.5],
        },
        Coords {
          xyz: [-0.5, 0.5, 0.5],
          st: [0.5, 0.5],
        },
      ],
      direction: Vector3::new(0, 0, 1),
    },
  ];
}
