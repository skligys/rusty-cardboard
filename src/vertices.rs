use std::u16;

use program::VertexArray;

pub struct Vertices {
  position_coords: Vec<f32>,
  position_indices: Vec<u16>,
  texture_coords: Vec<f32>,
  vertex_count: usize,
}

impl Vertices {
  pub fn new(cube_count: usize) -> Vertices {
    Vertices {
      // If the world nas N cubes in it, the mesh may have up to 12 * N triangles
      // and up to 6 * 12 * N vertices.  Set capacity to half of that since some
      // faces will be hidden.
      position_coords: Vec::with_capacity(36 * cube_count),
      // Up to 3 * 12 * N indices, halve it.
      position_indices: Vec::with_capacity(18 * cube_count),
      // Up to 4 * 12 * N texture coordinates, also halve it.
      texture_coords: Vec::with_capacity(24 * cube_count),
      vertex_count: 0,
    }
  }

  pub fn add(&mut self, position_coords: &[f32; 12], indices: &[u16; 6], texture_coords: &[f32; 8]) {
    let new_vertex_count = self.vertex_count + 4;
    assert!(new_vertex_count <= u16::MAX as usize, "Too many vertices: {}", new_vertex_count);

    self.position_coords.push_all(position_coords);
    self.position_indices.push_all(&shift(indices, self.vertex_count as u16));
    self.texture_coords.push_all(texture_coords);
    self.vertex_count += 4;
  }

  pub fn position_coord_len(&self) -> usize {
    self.position_coords.len()
  }

  pub fn texture_coord_len(&self) -> usize {
    self.texture_coords.len()
  }

  pub fn position_coord_array(&self) -> VertexArray {
    VertexArray {
      data: &self.position_coords[0..],
      components: 3,
      stride: 12,
    }
  }

  pub fn position_indices(&self) -> Vec<u16> {
    self.position_indices.clone()
  }

  pub fn texture_coord_array(&self) -> VertexArray {
    VertexArray {
      data: &self.texture_coords[0..],
      components: 2,
      stride: 8,
    }
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
