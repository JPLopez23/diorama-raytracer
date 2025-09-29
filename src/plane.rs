use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use raylib::prelude::Vector3;

pub struct Plane {
    pub point: Vector3,   
    pub normal: Vector3,  
    pub material: Material,
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let denom = self.normal.dot(*ray_direction);
        
        if denom.abs() < 1e-6 {
            return Intersect::empty();
        }
        
        let t = (self.point - *ray_origin).dot(self.normal) / denom;
        
        if t <= 0.0 {
            return Intersect::empty();
        }
        
        let point = *ray_origin + *ray_direction * t;
        let normal = if denom < 0.0 { self.normal } else { -self.normal };
        
        Intersect::new(point, normal, t, self.material.clone())
    }
}