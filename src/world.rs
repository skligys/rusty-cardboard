use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use time;

use cgmath;
use cgmath::{BaseNum, Point3};
use collision::{Aabb3, Line2};

use perlin;

pub type Block = Point3<i32>;

// Has to be odd since (0, 0, 0) is at the center of a chunk.
pub const CHUNK_SIZE: i32 = 17;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Chunk(Point3<i32>);

impl Chunk {
  pub fn new(x: i32, y: i32, z: i32) -> Chunk {
    Chunk(Point3 {
      x: x,
      y: y,
      z: z,
    })
  }

  pub fn block_bounds(&self) -> Aabb3<i32> {
    let center = Point3 {
      x: self.0.x * CHUNK_SIZE,
      y: self.0.y * CHUNK_SIZE,
      z: self.0.z * CHUNK_SIZE,
    };
    let half_chunk = (CHUNK_SIZE - 1) / 2;
    Aabb3 {
      min: Point3 {
        x: center.x - half_chunk,
        y: center.y - half_chunk,
        z: center.z - half_chunk,
      },
      max: Point3 {
        x: center.x + half_chunk,
        y: center.y + half_chunk,
        z: center.z + half_chunk,
      }
    }
  }
}

/// World model 𝓦.
pub struct World {
  /// All blocks in the world.
  blocks: HashSet<Block>,
  /// Blocks divided by chunk they belong to.
  chunk_blocks: HashMap<Chunk, Vec<Block>>,
  /// Eye coordinates.
  eye: Option<Point3<i32>>,
}

/// 2 dimensional point on xz plane.
#[derive(Clone, Debug)]
pub struct Point2<T> {
  pub x: T,
  pub z: T,
}

impl <T> Point2<T> {
  pub fn new(x: T, z: T) -> Point2<T> {
    Point2 {
      x: x,
      z: z,
    }
  }
}

impl <T: BaseNum> Point2<T> {
  pub fn as_cgmath(&self) -> cgmath::Point2<T> {
    cgmath::Point2::new(self.x, self.z)
  }
}

/// 2 dimensional segment on xz plane.
#[derive(Debug)]
pub struct Segment2<T> {
  pub start: Point2<T>,
  pub end: Point2<T>,
}

impl <T> Segment2<T> {
  pub fn new(start: Point2<T>, end: Point2<T>) -> Segment2<T> {
    Segment2 {
      start: start,
      end: end,
    }
  }
}

impl Segment2<i32> {
  pub fn as_cgmath(&self) -> Line2<f32> {
    let origin = cgmath::Point2::new(self.start.x as f32, self.start.z as f32);
    let dest = cgmath::Point2::new(self.end.x as f32, self.end.z as f32);
    Line2::new(origin, dest)
  }
}

/// 2 dimensional rectangle on xz plane.
#[derive(Debug)]
struct Rect2<T> {
  min: Point2<T>,
  max: Point2<T>,
}

impl World {
  /// Generates world chunks visible from the start point within the radius.
  pub fn new(start: &Point2<f32>, radius: f32) -> World {
    let start_s = time::precise_time_s();
    log!("*** Generating world...");

    // TODO: Load the chunk at (0, 0, 0) synchronously, load other chunks within radius in the
    // background, while prioritizing chunks in the field of view.
    let mut all_blocks: HashSet<Block> = HashSet::new();
    assert!(radius > 0.0);
    let capacity_estimate = radius as usize * radius as usize;
    let mut chunk_blocks: HashMap<Chunk, Vec<Block>> = HashMap::with_capacity(capacity_estimate);

    let eye = {
      let chunk0 = Chunk::new(0, 0, 0);
      let blocks0 = perlin::generate_blocks(&chunk0.block_bounds());
      all_blocks.extend(blocks0.clone());
      chunk_blocks.insert(chunk0, blocks0.clone());

      let start_block = Point2 {
        x: coord_to_block(start.x),
        z: coord_to_block(start.z),
      };
      place_eye(&blocks0, &start_block)
    };

    for c in within_radius_iter(start, radius) {
      let blocks = perlin::generate_blocks(&c.block_bounds());
      all_blocks.extend(blocks.clone());
      chunk_blocks.insert(c, blocks);
    }

    let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
    log!("*** Generated world: {:.3}ms, {} chunks, {} blocks", spent_ms, chunk_blocks.len(), all_blocks.len());

    World {
      blocks: all_blocks,
      chunk_blocks: chunk_blocks,
      eye: eye,
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.blocks.len()
  }

  #[inline]
  pub fn chunk_blocks(&self) -> hash_map::Iter<Chunk, Vec<Block>> {
    self.chunk_blocks.iter()
  }

  #[inline]
  pub fn contains(&self, block: &Block) -> bool {
    self.blocks.contains(block)
  }

  #[inline]
  pub fn eye(&self) -> Option<Point3<i32>> {
    self.eye
  }
}

struct WithinRadiusIterator {
  start: Point2<f32>,
  radius: f32,
  min_x: i32,
  next_x: i32,
  end_x: i32,
  next_z: i32,
  end_z: i32,
}

/// Returns chunks within given radius of the start point, except for chunk (0, 0, 0).
// Darn, have to hand-code this, there is no way to create ranges and filter_map
// them to return an iterator.
fn within_radius_iter(start: &Point2<f32>, radius: f32) -> WithinRadiusIterator {
  let min_x = coord_to_chunk(start.x - radius);
  let max_x = coord_to_chunk(start.z + radius);
  let min_z = coord_to_chunk(start.z - radius);
  let max_z = coord_to_chunk(start.z + radius);
  WithinRadiusIterator {
    start: start.clone(),
    radius: radius,
    min_x: min_x,
    next_x: min_x,
    end_x: max_x + 1,
    next_z: min_z,
    end_z: max_z + 1,
  }
}

impl WithinRadiusIterator {
  fn skip_chunk(&self) -> bool {
    if self.next_x == 0 && self.next_z == 0 {
      return true;
    }
    let chunk = Point2::new(self.next_x, self.next_z);
    !within_radius(&self.start, self.radius, &chunk)
  }

  fn inc(&mut self) -> () {
    if self.next_x < self.end_x {
      self.next_x += 1;
    } else {
      self.next_x = self.min_x;
      self.next_z += 1;
    }
  }
}

impl Iterator for WithinRadiusIterator {
  type Item = Chunk;
  fn next(&mut self) -> Option<Chunk> {
    while self.next_x < self.end_x && self.next_z < self.end_z {
      if self.skip_chunk() {
        self.inc();
        continue;
      }
      let result = Some(Chunk::new(self.next_x, 0, self.next_z));
      self.inc();
      return result;
    }
    None
  }
}

#[inline]
fn coord_to_chunk(coord: f32) -> i32 {
  (coord / CHUNK_SIZE as f32).round() as i32
}

#[inline]
fn coord_to_block(coord: f32) -> i32 {
  coord.round() as i32
}

fn within_radius(center: &Point2<f32>, radius: f32, chunk: &Point2<i32>) -> bool {
  let vertices = rect_to_vertices(&chunk_rect(chunk));
  let radius_squared = radius * radius;
  let within_radius_vertex = |v: &Point2<f32>| -> bool {
    distance_squared(v, center) <= radius_squared
  };
  within_radius_vertex(&vertices[0]) || within_radius_vertex(&vertices[1]) ||
    within_radius_vertex(&vertices[2]) || within_radius_vertex(&vertices[3])
}

fn chunk_rect(chunk: &Point2<i32>) -> Rect2<f32> {
  let center = Point2 {
    x: chunk.x * CHUNK_SIZE,
    z: chunk.z * CHUNK_SIZE,
  };

  let half_chunk = 0.5 * CHUNK_SIZE as f32;
  Rect2 {
    min: Point2 {
      x: center.x as f32 - half_chunk,
      z: center.z as f32 - half_chunk,
    },
    max: Point2 {
      x: center.x as f32 + half_chunk,
      z: center.z as f32 + half_chunk,
    }
  }
}

fn rect_to_vertices(rect: &Rect2<f32>) -> [Point2<f32>; 4] {
  [
    Point2::new(rect.min.x, rect.min.z),
    Point2::new(rect.max.x, rect.min.z),
    Point2::new(rect.min.x, rect.max.z),
    Point2::new(rect.max.x, rect.max.z),
  ]
}

fn distance_squared(p1: &Point2<f32>, p2: &Point2<f32>) -> f32 {
  (p1.x - p2.x) * (p1.x - p2.x) + (p1.z - p2.z) * (p1.z - p2.z)
}

/// Place the eye on top of the highest block: max {y: (xz.x, y, xz.z) ∈ 𝓦}
fn place_eye(blocks: &Vec<Block>, xz: &Point2<i32>) -> Option<Point3<i32>> {
  let y_of_matching_xz = |b: &Block| -> Option<i32> {
    if b.x == xz.z && b.z == xz.x {
      Some(b.y)
    } else {
      None
    }
  };
  let max_y = blocks.iter().filter_map(|&b| y_of_matching_xz(&b)).max();
  max_y.map(|y| {
    log!("*** Placed eye at: ({}, {}, {})", xz.x, y, xz.z);
    Point3::new(xz.x, y, xz.z)
  })
}

#[cfg(test)]
mod tests {
  use std::ops::Not;
  use super::{CHUNK_SIZE, Point2, coord_to_chunk, within_radius};

  #[test]
  fn coord_to_chunk_test_center_0() {
    assert_eq!(coord_to_chunk(0.0), 0)
  }

  #[test]
  fn coord_to_chunk_test_center_pos_1() {
    assert_eq!(coord_to_chunk(CHUNK_SIZE as f32), 1)
  }

  #[test]
  fn coord_to_chunk_test_center_neg_1() {
    assert_eq!(coord_to_chunk(-CHUNK_SIZE as f32), -1)
  }

  #[test]
  fn coord_to_chunk_test_center_pos_half() {
    assert_eq!(coord_to_chunk(CHUNK_SIZE as f32 * 0.5), 1)
  }

  #[test]
  fn coord_to_chunk_test_center_neg_half() {
    assert_eq!(coord_to_chunk(-CHUNK_SIZE as f32 * 0.5), -1)
  }

  #[test]
  fn within_radius_test_yes() {
    assert!(within_radius(&Point2::new(0.0, 0.0), CHUNK_SIZE as f32, &Point2::new(1, 1)));
  }

  #[test]
  fn within_radius_test_no() {
    assert!(within_radius(&Point2::new(0.0, 0.0), CHUNK_SIZE as f32 * 0.5, &Point2::new(0, 1)).not());
  }
}
