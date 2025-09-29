use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use raylib::prelude::Vector3;

pub struct Cube {
    pub center: Vector3,
    pub size: f32,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let half_size = self.size / 2.0;
        let min_bounds = self.center - Vector3::new(half_size, half_size, half_size);
        let max_bounds = self.center + Vector3::new(half_size, half_size, half_size);
        
        let inv_dir = Vector3::new(1.0 / ray_direction.x, 1.0 / ray_direction.y, 1.0 / ray_direction.z);
        
        let t1 = (min_bounds.x - ray_origin.x) * inv_dir.x;
        let t2 = (max_bounds.x - ray_origin.x) * inv_dir.x;
        let t3 = (min_bounds.y - ray_origin.y) * inv_dir.y;
        let t4 = (max_bounds.y - ray_origin.y) * inv_dir.y;
        let t5 = (min_bounds.z - ray_origin.z) * inv_dir.z;
        let t6 = (max_bounds.z - ray_origin.z) * inv_dir.z;
        
        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
        
        if tmax < 0.0 || tmin > tmax {
            return Intersect::empty();
        }
        
        let t = if tmin > 0.0 { tmin } else { tmax };
        
        if t <= 0.0 {
            return Intersect::empty();
        }
        
        let point = *ray_origin + *ray_direction * t;
        
        let local_point = point - self.center;
        let abs_local = Vector3::new(local_point.x.abs(), local_point.y.abs(), local_point.z.abs());
        
        let (normal, u, v) = if abs_local.x > abs_local.y && abs_local.x > abs_local.z {
            let normal = if local_point.x > 0.0 { 
                Vector3::new(1.0, 0.0, 0.0) 
            } else { 
                Vector3::new(-1.0, 0.0, 0.0) 
            };
            let u = (local_point.z / half_size + 1.0) * 0.5;
            let v = (local_point.y / half_size + 1.0) * 0.5;
            (normal, u, v)
        } else if abs_local.y > abs_local.z {
            let normal = if local_point.y > 0.0 { 
                Vector3::new(0.0, 1.0, 0.0) 
            } else { 
                Vector3::new(0.0, -1.0, 0.0) 
            };
            let u = (local_point.x / half_size + 1.0) * 0.5;
            let v = (local_point.z / half_size + 1.0) * 0.5;
            (normal, u, v)
        } else {
            let normal = if local_point.z > 0.0 { 
                Vector3::new(0.0, 0.0, 1.0) 
            } else { 
                Vector3::new(0.0, 0.0, -1.0) 
            };
            let u = (local_point.x / half_size + 1.0) * 0.5;
            let v = (local_point.y / half_size + 1.0) * 0.5;
            (normal, u, v)
        };
        
        Intersect::new_with_uv(point, normal, t, self.material.clone(), u, v)
    }
}