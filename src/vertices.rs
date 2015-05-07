use program::VertexArray;

pub struct Vertices {
  position_coords: Vec<f32>,
  texture_coords: Vec<f32>,
  vertex_count: usize,
}

impl Vertices {
  pub fn new(cube_count: usize) -> Vertices {
    Vertices {
      // If the world nas N cubes in it, the mesh may have up to 12 * N triangles
      // and up to 9 * 12 * N vertices.  Set capacity to half of that.
      position_coords: Vec::with_capacity(54 * cube_count),
      // Up to 6 * 12 * N texture coordinates, also halve it.
      texture_coords: Vec::with_capacity(36 * cube_count),
      vertex_count: 0,
    }
  }

  pub fn add(&mut self, position_coords: &[f32; 18], texture_coords: &[f32; 12]) {
    self.position_coords.push_all(position_coords);
    self.texture_coords.push_all(texture_coords);
    self.vertex_count += 6;
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

  pub fn texture_coord_array(&self) -> VertexArray {
    VertexArray {
      data: &self.texture_coords[0..],
      components: 2,
      stride: 8,
    }
  }
}
