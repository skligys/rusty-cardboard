use std::u16;

use cgmath::Vector3;

use program::VertexArray;
use world::{Block, World};

/// This has to have C layout since it is read by the OpenGL driver via a pointer passed to it.
#[repr(C)]
#[derive(Clone)]
pub struct Coords {
  xyz: [f32; 3],
  st: [u16; 2]
}

impl Coords {
  pub fn size_bytes() -> u32 {
    3 * 4 + 2 * 2
  }

  pub fn texture_offset() -> u32 {
    3 * 4
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
  static ref INDICES: [u16; 6] = [
    0, 1, 3,
    3, 1, 2,
  ];

  // 6 cube faces in canonical order: left, right, down, up, forward, back.
  static ref CUBE_FACES: [CubeFace; 6] = [
    // Left.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, -0.5, -0.5],
          st: [0x7fff, 0xffff],
        },
        Coords {
          xyz: [-0.5, -0.5, 0.5],
          st: [0xffff, 0xffff],
        },
        Coords {
          xyz: [-0.5, 0.5, 0.5],
          st: [0xffff, 0x7fff],
        },
        Coords {
          xyz: [-0.5, 0.5, -0.5],
          st: [0x7fff, 0x7fff],
        },
      ],
      direction: Vector3::new(-1, 0, 0),
    },
    // Right.
    CubeFace {
      coords: [
        Coords {
          xyz: [0.5, -0.5, 0.5],
          st: [0x7fff, 0xffff],
        },
        Coords {
          xyz: [0.5, -0.5, -0.5],
          st: [0xffff, 0xffff],
        },
        Coords {
          xyz: [0.5, 0.5, -0.5],
          st: [0xffff, 0x7fff],
        },
        Coords {
          xyz: [0.5, 0.5, 0.5],
          st: [0x7fff, 0x7fff],
        },
      ],
      direction: Vector3::new(1, 0, 0),
    },
    // Down.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, -0.5, -0.5],
          st: [0x0000, 0x7fff],
        },
        Coords {
          xyz: [0.5, -0.5, -0.5],
          st: [0x7fff, 0x7fff],
        },
        Coords {
          xyz: [0.5, -0.5, 0.5],
          st: [0x7fff, 0x0000],
        },
        Coords {
          xyz: [-0.5, -0.5,  0.5],
          st: [0x0000, 0x0000],
        },
      ],
      direction: Vector3::new(0, -1, 0),
    },
    // Up.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, 0.5, 0.5],
          st: [0x0000, 0xffff],
        },
        Coords {
          xyz: [0.5, 0.5,  0.5],
          st: [0x7fff, 0xffff],
        },
        Coords {
          xyz: [0.5, 0.5, -0.5],
          st: [0x7fff, 0x7fff],
        },
        Coords {
          xyz: [-0.5, 0.5, -0.5],
          st: [0x0000, 0x7fff],
        },
      ],
      direction: Vector3::new(0, 1, 0),
    },
    // Forward.
    CubeFace {
      coords: [
        Coords {
          xyz: [0.5, -0.5, -0.5],
          st: [0x7fff, 0xffff],
        },
        Coords {
          xyz: [-0.5, -0.5, -0.5],
          st: [0xffff, 0xffff],
        },
        Coords {
          xyz: [-0.5, 0.5, -0.5],
          st: [0xffff, 0x7fff],
        },
        Coords {
          xyz: [0.5,  0.5, -0.5],
          st: [0x7fff, 0x7fff],
        },
      ],
      direction: Vector3::new(0, 0, -1),
    },
    // Back.
    CubeFace {
      coords: [
        Coords {
          xyz: [-0.5, -0.5, 0.5],
          st: [0x7fff, 0xffff],
        },
        Coords {
          xyz: [0.5, -0.5, 0.5],
          st: [0xffff, 0xffff],
        },
        Coords {
          xyz: [0.5, 0.5, 0.5],
          st: [0xffff, 0x7fff],
        },
        Coords {
          xyz: [-0.5, 0.5, 0.5],
          st: [0x7fff, 0x7fff],
        },
      ],
      direction: Vector3::new(0, 0, 1),
    },
  ];
}

pub struct Vertices {
  coords: Vec<Coords>,
  indices: Vec<u16>,
}

impl Vertices {
  pub fn new(cube_count: usize) -> Vertices {
    Vertices {
      // If the world nas N cubes in it, the mesh may have up to 6 * N faces
      // and up to 6 * 4 * N vertices.  Set capacity to half of that since some
      // faces will be hidden.
      coords: Vec::with_capacity(12 * cube_count),
      // Up to 6 * 6 * N indices, halve it.
      indices: Vec::with_capacity(18 * cube_count),
    }
  }

  pub fn add(&mut self, coords: &[Coords; 4], indices: &[u16; 6]) {
    let old_vertex_count = self.coords.len();
    let new_vertex_count = old_vertex_count + 4;
    assert!(new_vertex_count <= u16::MAX as usize, "Too many vertices: {}", new_vertex_count);

    self.coords.extend(coords.into_iter().cloned());
    self.indices.extend(shift(indices, old_vertex_count as u16).into_iter());
  }

  pub fn coords(&self) -> &[Coords] {
    &self.coords[..]
  }

  pub fn coord_count(&self) -> usize {
    self.coords.len()
  }

  pub fn position_coord_array(&self) -> VertexArray {
    VertexArray {
      components: 3,
      stride: Coords::size_bytes(),
    }
  }

  pub fn texture_coord_array(&self) -> VertexArray {
    VertexArray {
      components: 2,
      stride: Coords::size_bytes(),
    }
  }

  pub fn indices(&self) -> &[u16] {
    &self.indices[..]
  }

  pub fn index_count(&self) -> usize {
    self.indices.len()
  }
}

fn shift(indices: &[u16; 6], by: u16) -> [u16; 6] {
  [
    indices[0] + by,
    indices[1] + by,
    indices[2] + by,
    indices[3] + by,
    indices[4] + by,
    indices[5] + by,
  ]
}

pub fn create_mesh_vertices(blocks: &Vec<Block>, world: &World) -> Vertices {
  let mut vertices = Vertices::new(world.len());
  for block in blocks {
    // Eliminate definitely invisible faces, i.e. those between two neighboring cubes.
    for face in CUBE_FACES.iter() {
      if !world.contains(&(block + face.direction)) {
        vertices.add(&translate(&face.coords, block), &INDICES);
      }
    }
  }
  vertices
}

/// Accepts vertex and texture coordinates.  Translates vertex coordinates only along the vector
// corresponding to the block center position.
fn translate(coords: &[Coords; 4], block: &Block) -> [Coords; 4] {
  let x = block.x as f32;
  let y = block.y as f32;
  let z = block.z as f32;

  [
    coords[0].translate(x, y, z),
    coords[1].translate(x, y, z),
    coords[2].translate(x, y, z),
    coords[3].translate(x, y, z),
  ]
}
