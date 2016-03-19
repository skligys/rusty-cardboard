use std::collections::{HashMap, HashSet};
use std::collections::hash_map;
use std::ops::Range;
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

impl Rect2<i32> {
  fn x_range(&self) -> Range<i32> {
    Range {
      start: self.min.x,
      // Important: +1 since the end is exclusive.
      end: self.max.x + 1,
    }
  }

  fn z_range(&self) -> Range<i32> {
    Range {
      start: self.min.z,
      // Important: +1 since the end is exclusive.
      end: self.max.z + 1,
    }
  }
}

impl World {
  /// Generates world chunks visible from the start point within the radius.
  pub fn new(start: &Point2<f32>, radius: f32) -> World {
    let start_s = time::precise_time_s();
    log!("*** Generating world...");

    let xz_chunk_rect = start_radius_to_chunk_rect(start, radius);

    let mut chunks: Vec<Point2<i32>> = Vec::new();
    for x_chunk in xz_chunk_rect.x_range() {
      for z_chunk in xz_chunk_rect.z_range() {
        let chunk = Point2::new(x_chunk, z_chunk);
        if within_radius(start, radius, &chunk) {
          chunks.push(chunk);
        }
      }
    }
    let chunks = &chunks;

    let mut all_blocks: HashSet<Block> = HashSet::new();
    let mut chunk_blocks: HashMap<Chunk, Vec<Block>> = HashMap::with_capacity(chunks.len());
    for c in chunks {
      let chunk_3d = Chunk::new(c.x, 0, c.z);
      let block_bounds = chunk_3d.block_bounds();
      let blocks = perlin::generate_blocks(&block_bounds);

      all_blocks.extend(blocks.clone());
      chunk_blocks.insert(chunk_3d, blocks);
    }

    let eye = {
      let chunk0_blocks = chunk_blocks.get(&Chunk::new(0, 0, 0)).unwrap();
      let start_block = Point2 {
        x: coord_to_block(start.x),
        z: coord_to_block(start.z),
      };
      place_eye(&chunk0_blocks, &start_block)
    };

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

fn start_radius_to_chunk_rect(start: &Point2<f32>, radius: f32) -> Rect2<i32> {
  Rect2 {
    min: Point2 {
      x: coord_to_chunk(start.x - radius),
      z: coord_to_chunk(start.z - radius),
    },
    max: Point2 {
      x: coord_to_chunk(start.x + radius),
      z: coord_to_chunk(start.z + radius),
    },
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
