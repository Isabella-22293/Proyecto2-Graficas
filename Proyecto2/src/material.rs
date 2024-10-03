use crate::{color::Color, Texture};

#[derive(Debug, Clone)] // Solo Debug, sin Clone
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub texture: Option<Texture>, // Campo texture definido aquí
}

impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture: Option<Texture>, // Añadido el campo texture al constructor
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            texture, // Inicialización del campo texture
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
            refractive_index: 0.0,
            texture: None, // Inicializa texture como None
        }
    }
}

impl Default for Material {
  fn default() -> Self {
      Self::black()
  }
}