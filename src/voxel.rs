use std::collections::HashMap;
use raylib::prelude::Vector3;
use crate::material::Material;
use crate::ray_intersect::Intersect;

#[derive(Clone)]
pub struct VoxelGrid {
    pub cells: HashMap<(i32, i32, i32), Material>,
    bounds_min: Vector3,
    bounds_max: Vector3,
    has_bounds: bool,
    center: Vector3,
    radius: f32,
    bounds_cached: bool,
}

impl VoxelGrid {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            bounds_min: Vector3::new(0.0, 0.0, 0.0),
            bounds_max: Vector3::new(0.0, 0.0, 0.0),
            has_bounds: false,
            center: Vector3::zero(),
            radius: 0.0,
            bounds_cached: false,
        }
    }

    #[inline]
    pub fn insert(&mut self, x: i32, y: i32, z: i32, m: Material) {
        self.cells.insert((x, y, z), m);
        self.bounds_cached = false;
        
        if !self.has_bounds {
            self.bounds_min = Vector3::new(x as f32, y as f32, z as f32);
            self.bounds_max = self.bounds_min;
            self.has_bounds = true;
        } else {
            if (x as f32) < self.bounds_min.x { self.bounds_min.x = x as f32; }
            if (y as f32) < self.bounds_min.y { self.bounds_min.y = y as f32; }
            if (z as f32) < self.bounds_min.z { self.bounds_min.z = z as f32; }
            if (x as f32) > self.bounds_max.x { self.bounds_max.x = x as f32; }
            if (y as f32) > self.bounds_max.y { self.bounds_max.y = y as f32; }
            if (z as f32) > self.bounds_max.z { self.bounds_max.z = z as f32; }
        }
    }

    fn update_cached_values(&mut self) {
        if !self.bounds_cached {
            let (min, max) = self.calculate_bounds();
            self.center = Vector3::new(
                0.5 * (min.x + max.x),
                0.5 * (min.y + max.y),
                0.5 * (min.z + max.z),
            );
            let rx = (max.x - self.center.x).abs();
            let ry = (max.y - self.center.y).abs();
            let rz = (max.z - self.center.z).abs();
            self.radius = (rx * rx + ry * ry + rz * rz).sqrt();
            self.bounds_cached = true;
        }
    }

    /// Devuelve (min, max) de la estructura.
    pub fn calculate_bounds(&self) -> (Vector3, Vector3) {
        if !self.has_bounds || self.cells.is_empty() {
            return (Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0));
        }
        
        let min = Vector3::new(
            self.bounds_min.x - 0.1,
            self.bounds_min.y - 0.1,
            self.bounds_min.z - 0.1,
        );
        let max = Vector3::new(
            self.bounds_max.x + 1.1,
            self.bounds_max.y + 1.1,
            self.bounds_max.z + 1.1,
        );
        (min, max)
    }

    pub fn get_center(&mut self) -> Vector3 {
        self.update_cached_values();
        self.center
    }

    pub fn get_bounding_sphere_radius(&mut self) -> f32 {
        self.update_cached_values();
        self.radius
    }

    pub fn intersect_ray(&self, ro: &Vector3, rd: &Vector3, t_max: f32, max_steps: u32) -> Intersect {
        if self.cells.is_empty() {
            return Intersect::empty();
        }

        let (bmin, bmax) = self.calculate_bounds();

        let (mut tmin, mut tmax_box) = (0.0f32, t_max);
        for i in 0..3 {
            let (ro_i, rd_i, min_i, max_i) = match i {
                0 => (ro.x, rd.x, bmin.x, bmax.x),
                1 => (ro.y, rd.y, bmin.y, bmax.y),
                _ => (ro.z, rd.z, bmin.z, bmax.z),
            };
            
            if rd_i.abs() < 1e-6 {
                if ro_i < min_i || ro_i > max_i {
                    return Intersect::empty();
                }
            } else {
                let inv_d = 1.0 / rd_i;
                let mut t0 = (min_i - ro_i) * inv_d;
                let mut t1 = (max_i - ro_i) * inv_d;
                if inv_d < 0.0 { 
                    std::mem::swap(&mut t0, &mut t1); 
                }
                if t0 > tmin { 
                    tmin = t0; 
                }
                if t1 < tmax_box { 
                    tmax_box = t1; 
                }
                if tmax_box < tmin {
                    return Intersect::empty();
                }
            }
        }

        let t_entry = if tmin < 0.0 { 0.0 } else { tmin };
        let start = Vector3::new(
            ro.x + rd.x * t_entry, 
            ro.y + rd.y * t_entry, 
            ro.z + rd.z * t_entry
        );
        
        let mut cx = start.x.floor() as i32;
        let mut cy = start.y.floor() as i32;
        let mut cz = start.z.floor() as i32;

        let step_x = if rd.x > 0.0 { 1 } else { -1 };
        let step_y = if rd.y > 0.0 { 1 } else { -1 };
        let step_z = if rd.z > 0.0 { 1 } else { -1 };

        let t_delta_x = if rd.x.abs() < 1e-6 { f32::INFINITY } else { 1.0 / rd.x.abs() };
        let t_delta_y = if rd.y.abs() < 1e-6 { f32::INFINITY } else { 1.0 / rd.y.abs() };
        let t_delta_z = if rd.z.abs() < 1e-6 { f32::INFINITY } else { 1.0 / rd.z.abs() };

        let next_boundary_x = if rd.x > 0.0 { cx as f32 + 1.0 } else { cx as f32 };
        let next_boundary_y = if rd.y > 0.0 { cy as f32 + 1.0 } else { cy as f32 };
        let next_boundary_z = if rd.z > 0.0 { cz as f32 + 1.0 } else { cz as f32 };

        let mut t_max_x = if rd.x.abs() < 1e-6 { 
            f32::INFINITY 
        } else { 
            (next_boundary_x - start.x).abs() / rd.x.abs() 
        };
        let mut t_max_y = if rd.y.abs() < 1e-6 { 
            f32::INFINITY 
        } else { 
            (next_boundary_y - start.y).abs() / rd.y.abs() 
        };
        let mut t_max_z = if rd.z.abs() < 1e-6 { 
            f32::INFINITY 
        } else { 
            (next_boundary_z - start.z).abs() / rd.z.abs() 
        };

        let mut hit_face = 3;

        let limit_min = (bmin.x.floor() as i32, bmin.y.floor() as i32, bmin.z.floor() as i32);
        let limit_max = (bmax.x.ceil() as i32, bmax.y.ceil() as i32, bmax.z.ceil() as i32);

        for _ in 0..max_steps {
            if cx < limit_min.0 || cy < limit_min.1 || cz < limit_min.2 ||
               cx > limit_max.0 || cy > limit_max.1 || cz > limit_max.2 {
                break;
            }

            if let Some(mat) = self.cells.get(&(cx, cy, cz)) {
                let t_hit = if hit_face == 0 {
                    t_max_x - t_delta_x
                } else if hit_face == 1 {
                    t_max_y - t_delta_y
                } else if hit_face == 2 {
                    t_max_z - t_delta_z
                } else {
                    t_entry
                } + t_entry;

                if t_hit > t_max { 
                    break; 
                }

                let hit_point = Vector3::new(
                    ro.x + rd.x * t_hit,
                    ro.y + rd.y * t_hit,
                    ro.z + rd.z * t_hit,
                );

                let normal = match hit_face {
                    0 => Vector3::new(if rd.x > 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0),
                    1 => Vector3::new(0.0, if rd.y > 0.0 { -1.0 } else { 1.0 }, 0.0),
                    2 => Vector3::new(0.0, 0.0, if rd.z > 0.0 { -1.0 } else { 1.0 }),
                    _ => Vector3::new(0.0, 1.0, 0.0),
                };

                let (u, v) = face_uv_optimized(&hit_point, cx, cy, cz, &normal);

                return Intersect::new_with_uv(hit_point, normal, t_hit, mat.clone(), u, v);
            }

            if t_max_x <= t_max_y && t_max_x <= t_max_z {
                cx += step_x;
                t_max_x += t_delta_x;
                hit_face = 0;
            } else if t_max_y <= t_max_z {
                cy += step_y;
                t_max_y += t_delta_y;
                hit_face = 1;
            } else {
                cz += step_z;
                t_max_z += t_delta_z;
                hit_face = 2;
            }
        }

        Intersect::empty()
    }
}

#[inline]
fn face_uv_optimized(hit: &Vector3, cx: i32, cy: i32, cz: i32, normal: &Vector3) -> (f32, f32) {
    let fx = hit.x - cx as f32;
    let fy = hit.y - cy as f32;
    let fz = hit.z - cz as f32;

    // Mapeo preciso para cada cara del cubo
    if normal.x.abs() > 0.5 {
        // Cara X (izquierda/derecha)
        (wrap01_precise(fz), 1.0 - wrap01_precise(fy))
    } else if normal.y.abs() > 0.5 {
        // Cara Y (arriba/abajo)
        (wrap01_precise(fx), 1.0 - wrap01_precise(fz))
    } else {
        // Cara Z (frente/atrás)
        (wrap01_precise(fx), 1.0 - wrap01_precise(fy))
    }
}

#[inline(always)]
fn wrap01_precise(v: f32) -> f32 {
    let result = v - v.floor();
    if result < 0.0 { 
        result + 1.0 
    } else { 
        result 
    }
}