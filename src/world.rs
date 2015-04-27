use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::default::Default;

use cgmath::Point3;

pub type Block = Point3<i32>;
pub struct World(HashSet<Block>);

impl Default for World {
  // An empty world.
  fn default() -> World {
    World (
      HashSet::new()
    )
  }
}

impl World {
  #[inline]
  pub fn add(&mut self, block: Block) {
    self.0.insert(block);
  }

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
