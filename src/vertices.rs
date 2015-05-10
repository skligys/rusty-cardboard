use std::u16;

use mesh::Coords;
use program::VertexArray;

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

    self.coords.push_all(coords);
    self.indices.push_all(&shift(indices, old_vertex_count as u16));
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
