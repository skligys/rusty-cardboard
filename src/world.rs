use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::i32;

use cgmath::Point3;

pub type Block = Point3<i32>;

// Has to be odd since (0, 0, 0) is at the center of a chunk.
pub const CHUNK_SIZE: i32 = 17;
pub type Chunk = Point3<i32>;

/// World model 𝓦.
#[derive(Debug)]
pub struct World {
  blocks: HashSet<Block>,
  /// Eye coordinates.
  eye: Option<Point3<i32>>,
}

impl World {
  pub fn new(blocks: Vec<Block>, eye_x: i32, eye_z: i32) -> World {
    let eye = place_eye(&blocks, eye_x, eye_z);
    World {
      blocks: blocks.into_iter().collect(),
      eye: eye,
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.blocks.len()
  }

  #[inline]
  pub fn iter(&self) -> Iter<Block> {
    self.blocks.iter()
  }

  #[inline]
  pub fn contains(&self, block: &Block) -> bool {
    self.blocks.contains(block)
  }

  pub fn eye(&self) -> Option<Point3<i32>> {
    self.eye
  }
}

/// Place the eye on top of the highest block among {(x, y, z), ∀y}
pub fn place_eye(blocks: &Vec<Block>, x: i32, z: i32) -> Option<Point3<i32>> {
  let mut y = i32::MIN;
  for b in blocks.iter() {
    if b.x == x && b.z == z && b.y > y {
      y = b.y;
    }
  }
  if y > i32::MIN {
    log!("*** Placed eye at: ({}, {}, {})", x, y, z);
    Some(Point3::new(x, y, z))
  } else {
    None
  }
}
