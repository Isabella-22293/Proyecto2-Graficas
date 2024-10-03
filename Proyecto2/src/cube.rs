use nalgebra_glm::Vec3;
use crate::material::Material;
use crate::ray_intersect::{RayIntersect, Intersect};
use std::sync::Arc;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Arc<Material>, // Usar Arc aquí para permitir compartición de datos
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material: Arc<Material>) -> Self {
        Cube { min, max, material }
    }

    pub fn calculate_uv(&self, intersect: &Intersect) -> (f32, f32) {
        let local_point = intersect.point - self.min; // Coordenada local dentro del cubo
        let size = self.size();

        let (u, v) = if intersect.normal.x.abs() > 0.0 {
            // Cara del cubo que es paralela al plano YZ
            ((local_point.z / size.z) % 1.0, (local_point.y / size.y) % 1.0)
        } else if intersect.normal.y.abs() > 0.0 {
            // Cara del cubo que es paralela al plano XZ
            ((local_point.x / size.x) % 1.0, (local_point.z / size.z) % 1.0)
        } else {
            // Cara del cubo que es paralela al plano XY
            ((local_point.x / size.x) % 1.0, (local_point.y / size.y) % 1.0)
        };

        (u.abs(), v.abs()) // Aseguramos que las coordenadas UV sean positivas
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Intersect {
        // Evitar divisiones por cero y manejar rayos paralelos a los planos del cubo
        let inv_dir_x = if direction.x != 0.0 { 1.0 / direction.x } else { f32::INFINITY };
        let inv_dir_y = if direction.y != 0.0 { 1.0 / direction.y } else { f32::INFINITY };
        let inv_dir_z = if direction.z != 0.0 { 1.0 / direction.z } else { f32::INFINITY };

        let mut tmin = (self.min.x - origin.x) * inv_dir_x;
        let mut tmax = (self.max.x - origin.x) * inv_dir_x;

        if tmin > tmax {
            (tmin, tmax) = (tmax, tmin); // Intercambia si tmin > tmax
        }

        let mut tymin = (self.min.y - origin.y) * inv_dir_y;
        let mut tymax = (self.max.y - origin.y) * inv_dir_y;

        if tymin > tymax {
            (tymin, tymax) = (tymax, tymin);
        }

        if tmin > tymax || tymin > tmax {
            return Intersect::empty();
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min.z - origin.z) * inv_dir_z;
        let mut tzmax = (self.max.z - origin.z) * inv_dir_z;

        if tzmin > tzmax {
            (tzmin, tzmax) = (tzmax, tzmin);
        }

        if tmin > tzmax || tzmin > tmax {
            return Intersect::empty();
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        let point = origin + direction * tmin;
        let normal = self.get_normal(&point);

        let uv = Some(self.calculate_uv(&Intersect {
            is_intersecting: true,
            distance: tmin,
            point,
            normal,
            material: self.material.clone(),
            uv: None,
        }));

        Intersect {
            is_intersecting: true,
            distance: tmin,
            point,
            normal,
            material: self.material.clone(), // Clonamos el material
            uv,
        }
    }
}

impl Cube {
    fn get_normal(&self, point: &Vec3) -> Vec3 {
        // Determinar la normal en función de la posición del punto de intersección
        if (point.x - self.min.x).abs() < 1e-3 {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max.x).abs() < 1e-3 {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min.y).abs() < 1e-3 {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max.y).abs() < 1e-3 {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min.z).abs() < 1e-3 {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}
