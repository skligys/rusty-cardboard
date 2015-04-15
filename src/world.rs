use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::default::Default;

use cgmath::Point3;

pub type Block = Point3<i32>;
pub struct World(HashSet<Block>);

impl Default for World {
  fn default() -> World {
    // A few blocks arranged for testing.
    let v = vec![
      // 3x3 ground.
      Block::new(-1, 0, -1),
      Block::new(-1, 0, 0),
      Block::new(-1, 0, 1),
      Block::new(0, 0, -1),
      Block::new(0, 0, 0),
      Block::new(0, 0, 1),
      Block::new(1, 0, -1),
      Block::new(1, 0, 0),
      Block::new(1, 0, 1),
      // 3 high tower.
      Block::new(0, 1, 0),
      Block::new(0, 2, 0),
      Block::new(0, 3, 0),
      // 1 high tower.
      Block::new(0, 1, 1),
      // 2 high tower.
      Block::new(1, 1, 0),
      Block::new(1, 2, 0),
    ];
    World(v.into_iter().collect())
  }
}

impl World {
  #[inline]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  #[inline]
  pub fn iter(&self) -> Iter<Block> {
    self.0.iter()
  }

  #[inline]
  pub fn contains(&self, block: &Block) -> bool {
    self.0.contains(block)
  }
}
