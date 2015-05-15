use std::ops::Range;
use time;

use cgmath::Aabb3;
use noise;
use noise::{Brownian3, Seed};

use world::Block;

pub fn generate_blocks(boundaries: &Aabb3<i32>) -> Vec<Block> {
  let start_s = time::precise_time_s();

  let seed = Seed::new(1);
  let noise = Brownian3::new(noise::perlin3, 4).wavelength(16.0);

  let x_range = Range {
    start: boundaries.min.x,
    end: boundaries.max.x + 1,
  };
  let y_range = Range {
    start: boundaries.min.y,
    end: boundaries.max.y + 1,
  };
  let z_range = Range {
    start: boundaries.min.z,
    end: boundaries.max.z + 1,
  };
  let y_scale = 1.0 / (y_range.end as f64 - 1.0 - y_range.start as f64);
  let y_min = y_range.start as f64;

  let mut blocks = Vec::new();
  for y in y_range {
    // Normalize into [0, 1].
    let normalized_y = (y as f64 - y_min) * y_scale;
    for x in x_range.clone() {
      for z in z_range.clone() {
        let p = [x as f64, y as f64, z as f64];
        let val = noise.apply(&seed, &p);

        // Probablility to have a block added linearly increases from 0.0 at y_max to 1.0 at y_min.
        if 0.5 * (val + 1.0) >= normalized_y {
          blocks.push(Block::new(x, y, z));
        }
      }
    }
  }

  let spent_ms = (time::precise_time_s() - start_s) * 1000.0;
  log!("*** Generated a chunk of perlin: {:.3}ms, {} blocks", spent_ms, blocks.len());

  blocks
}
