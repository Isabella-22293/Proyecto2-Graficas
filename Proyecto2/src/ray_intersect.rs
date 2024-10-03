use nalgebra_glm::Vec3;
use crate::material::Material;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Intersect {
    pub is_intersecting: bool,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Arc<Material>, // Usar Arc para compartir el material
    pub uv: Option<(f32, f32)>, // Coordenas UV opcionales
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Arc<Material>) -> Self {
        Intersect {
            point,
            normal,
            distance,
            is_intersecting: true,
            material,
            uv: None,
        }
    }

    pub fn empty() -> Self {
        Intersect {
            is_intersecting: false,
            distance: f32::INFINITY,
            point: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            material: Arc::new(Material::default()),
            uv: None,
        }
    }

    // Método para calcular coordenadas UV
    pub fn calculate_uv(&self) -> (f32, f32) {
        let u = 0.5 + (self.normal.x.atan2(self.normal.z) / (2.0 * std::f32::consts::PI));
        let v = 0.5 - (self.normal.y + 1.0) / 2.0;
        (u, v)
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
