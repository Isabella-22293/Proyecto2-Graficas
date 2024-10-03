use nalgebra_glm::Vec3;
use crate::cube::Cube;
use crate::material::Material;

pub struct Skybox {
    pub cube: Cube,
}

impl Skybox {
    pub fn new(size: f32, material: Material) -> Self {
        let half_size = size / 2.0;
        let cube = Cube::new(
            Vec3::new(-half_size, -half_size, -half_size),
            Vec3::new(half_size, half_size, half_size),
            material.into(),
        );
        
        Skybox { cube }
    }
}
