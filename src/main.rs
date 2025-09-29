#![allow(unused_imports)]
#![allow(clippy::needless_return)]

use raylib::prelude::*;
use rayon::prelude::*;
use std::sync::Arc;
use std::f32::consts::PI;

mod voxel;
mod material;
mod light;
mod texture;
mod ray_intersect;
mod camera;
mod cube;
mod framebuffer;
mod plane;
mod sphere;

use voxel::VoxelGrid;
use material::{Material, MaterialType};

const WIDTH: i32 = 1200;
const HEIGHT: i32 = 800;
const RENDER_SCALE: i32 = 1;
const MAX_RAY_DIST: f32 = 50.0;
const MAX_DDA_STEPS: u32 = 100;
const MAX_REFLECTIONS: u32 = 3;

#[derive(Clone, Copy)]
struct BoundingSphere {
    center: Vector3,
    radius: f32,
}

#[derive(Clone, Copy, Debug)]
struct SimpleCamera {
    pub eye: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub vfov_deg: f32,
    forward: Vector3,
    right: Vector3,
    up_vec: Vector3,
}

impl SimpleCamera {
    pub fn new(eye: Vector3, target: Vector3, up: Vector3, vfov_deg: f32) -> Self {
        let mut cam = Self { 
            eye, target, up, vfov_deg,
            forward: Vector3::zero(),
            right: Vector3::zero(),
            up_vec: Vector3::zero(),
        };
        cam.update_vectors();
        cam
    }

    fn update_vectors(&mut self) {
        self.forward = (self.target - self.eye).normalized();
        self.right = self.forward.cross(self.up).normalized();
        self.up_vec = self.right.cross(self.forward);
    }

    pub fn set_position(&mut self, eye: Vector3, target: Vector3) {
        self.eye = eye;
        self.target = target;
        self.update_vectors();
    }

    #[inline]
    pub fn get_ray(&self, x: i32, y: i32, width: i32, height: i32) -> Ray {
        let aspect = width as f32 / height as f32;
        let fov = (self.vfov_deg * PI / 180.0).tan();

        let ndc_x = ((x as f32 + 0.5) / width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - ((y as f32 + 0.5) / height as f32) * 2.0;

        let direction = self.right * (ndc_x * aspect * fov) + 
                       self.up_vec * (ndc_y * fov) + 
                       self.forward;
        
        Ray {
            origin: self.eye,
            direction: direction.normalized(),
        }
    }
}
struct Ray {
    origin: Vector3,
    direction: Vector3,
}

#[derive(Clone, Copy)]
struct DirLight {
    dir: Vector3,
    color: Color,
    intensity: f32,
}

struct SharedRenderState {
    grid: Arc<VoxelGrid>,
    bounding_sphere: BoundingSphere,
    sun: DirLight,
    skybox_enabled: bool,
}

unsafe impl Sync for SharedRenderState {}
unsafe impl Send for SharedRenderState {}

#[inline]
fn ray_intersects_sphere(ray: &Ray, sphere: &BoundingSphere) -> bool {
    let oc = ray.origin - sphere.center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0 && (-b - discriminant.sqrt()) / (2.0 * a) > 0.0
}

#[inline]
fn enhanced_skybox(rd: Vector3) -> Color {
    let t = (rd.y * 0.5 + 0.5).clamp(0.0, 1.0);
    
    let horizon_color = Vector3::new(1.0, 0.9, 0.8);
    let zenith_color = Vector3::new(0.2, 0.5, 1.0);
    let nadir_color = Vector3::new(0.3, 0.4, 0.6);
    
    let sky_color = if t > 0.5 {
        let t_upper = (t - 0.5) * 2.0;
        let smooth_t = t_upper.powf(0.6);
        horizon_color + (zenith_color - horizon_color) * smooth_t
    } else {
        let t_lower = t * 2.0;
        let smooth_t = t_lower.powf(1.4);
        nadir_color + (horizon_color - nadir_color) * smooth_t
    };
    
    let noise = (rd.x * 5.0 + rd.z * 3.0).sin() * 0.02;
    let final_color = sky_color + Vector3::new(noise, noise * 0.5, noise * 0.3);
    
    Color::new(
        (final_color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (final_color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (final_color.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    )
}

fn cast_ray_recursive(ray: &Ray, state: &SharedRenderState, depth: u32) -> Color {
    if depth > MAX_REFLECTIONS {
        return if state.skybox_enabled {
            enhanced_skybox(ray.direction)
        } else {
            Color::BLACK
        };
    }

    // Eliminar el chequeo de bounding sphere que causa desapariciones
    // Solo verificamos si hay intersección directa con voxels

    let hit = state.grid.intersect_ray(&ray.origin, &ray.direction, MAX_RAY_DIST, MAX_DDA_STEPS);

    if !hit.is_intersecting {
        return if state.skybox_enabled {
            enhanced_skybox(ray.direction)
        } else {
            Color::BLACK
        };
    }

    // Shading con reflexión mejorada
    let base_color = shade_pixel_pbr(
        hit.point, hit.normal, &hit.material, hit.u, hit.v, 
        &state.sun, &state.grid, ray.origin, ray.direction
    );
    
    let mut final_color = base_color;

    // Reflexión con tinte del material
    let reflection_strength = hit.material.albedo[2];
    if reflection_strength > 0.01 && depth < MAX_REFLECTIONS {
        let reflect_dir = reflect_vector(ray.direction, hit.normal);
        let reflect_origin = hit.point + hit.normal * 0.0001; // Offset reducido
        let reflect_ray = Ray {
            origin: reflect_origin,
            direction: reflect_dir,
        };
        
        let reflect_color = cast_ray_recursive(&reflect_ray, state, depth + 1);
        let reflect_vec = color_to_vector3(reflect_color);
        
        // Para metales: tinte del reflejo con el color del material
        if hit.material.metallic > 0.5 {
            let tinted_reflection = Vector3::new(
                reflect_vec.x * (0.3 + base_color.x * 0.7),
                reflect_vec.y * (0.3 + base_color.y * 0.7),
                reflect_vec.z * (0.3 + base_color.z * 0.7)
            );
            final_color = base_color * (1.0 - reflection_strength * 0.5) + tinted_reflection * reflection_strength * 0.5;
        } else {
            // Para no-metales: reflexión con menos mezcla del color base
            let tinted_reflection = Vector3::new(
                reflect_vec.x * (0.5 + base_color.x * 0.5),
                reflect_vec.y * (0.5 + base_color.y * 0.5),
                reflect_vec.z * (0.5 + base_color.z * 0.5)
            );
            final_color = base_color * (1.0 - reflection_strength * 0.4) + tinted_reflection * reflection_strength * 0.4;
        }
    }

    vector3_to_color(final_color)
}

fn shade_pixel_pbr(
    hit_point: Vector3,
    normal: Vector3,
    mat: &Material,
    u: f32,
    v: f32,
    sun: &DirLight,
    grid: &VoxelGrid,
    cam_pos: Vector3,
    _ray_dir: Vector3,
) -> Vector3 {
    let albedo = mat.get_diffuse_color_sharp(u, v, &normal);
    
    let ndotl = normal.dot(-sun.dir).max(0.0);
    let ambient = 0.25;
    let diffuse = ndotl * 0.75;
    
    let shadow_factor = if ndotl > 0.05 {
        let shadow_origin = hit_point + normal * 0.0001; // Offset reducido
        let shadow_ray = grid.intersect_ray(&shadow_origin, &-sun.dir, 20.0, 50);
        if shadow_ray.is_intersecting { 0.3 } else { 1.0 }
    } else {
        1.0
    };
    
    let specular = if mat.specular > 5.0 {
        let view_dir = (cam_pos - hit_point).normalized();
        let reflect_dir = reflect_vector(sun.dir, normal);
        let spec_dot = view_dir.dot(reflect_dir).max(0.0);
        let roughness_factor = 1.0 / (mat.roughness * 50.0 + 1.0);
        spec_dot.powf(mat.specular * roughness_factor) * 0.5 * shadow_factor
    } else {
        0.0
    };
    
    let emission = if mat.emission > 0.0 {
        mat.get_emission_color(u, v, &normal) * mat.emission
    } else {
        Vector3::zero()
    };
    
    let total_lighting = ambient + (diffuse + specular) * shadow_factor;
    let lit_color = albedo * total_lighting;
    let final_color = lit_color + emission;
    
    Vector3::new(
        final_color.x / (1.0 + final_color.x * 0.8),
        final_color.y / (1.0 + final_color.y * 0.8),
        final_color.z / (1.0 + final_color.z * 0.8)
    )
}

fn reflect_vector(incident: Vector3, normal: Vector3) -> Vector3 {
    incident - normal * 2.0 * incident.dot(normal)
}

fn color_to_vector3(color: Color) -> Vector3 {
    Vector3::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    )
}

fn vector3_to_color(v: Vector3) -> Color {
    Color::new(
        (v.x.clamp(0.0, 1.0) * 255.0) as u8,
        (v.y.clamp(0.0, 1.0) * 255.0) as u8,
        (v.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    )
}

fn render_parallel_optimized(
    camera: &SimpleCamera,
    width: i32,
    height: i32,
    state: &SharedRenderState,
) -> Vec<Color> {
    let pixel_coords: Vec<(i32, i32)> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .collect();

    pixel_coords
        .par_iter()
        .map(|(x, y)| {
            let ray = camera.get_ray(*x, *y, width, height);
            cast_ray_recursive(&ray, state, 0)
        })
        .collect()
}

fn create_diorama_grid() -> VoxelGrid {
    let mut grid = VoxelGrid::new();

    let mat_of = |ch: char| -> Option<Material> {
        match ch {
            'M' => Some(Material::new_with_type(MaterialType::Dirt)),
            'T' => Some(Material::new_with_type(MaterialType::Grass)),
            'P' => Some(Material::new_with_type(MaterialType::Netherrack)),
            'R' => Some(Material::new_with_type(MaterialType::Stone)),
            'L' => Some(Material::new_with_type(MaterialType::Magma)),
            'O' => Some(Material::new_with_type(MaterialType::Gold)),
            'B' => Some(Material::new_with_type(MaterialType::Obsidian)),
            'S' => Some(Material::new_with_type(MaterialType::StoneStairs)),
            'Z' => Some(Material::new_with_type(MaterialType::StoneSlab)),
            'J' => Some(Material::new_with_type(MaterialType::StonePillar)),
            'C' => Some(Material::new_with_type(MaterialType::WoodChest)),
            'W' => Some(Material::new_with_type(MaterialType::GlowingObsidian)),
            'V' | '.' => None,
            _ => None,
        }
    };

    for layer_num in 1..=9 {
        let filename = format!("layers/Capa {}.txt", layer_num);
        
        match std::fs::read_to_string(&filename) {
            Ok(content) => {
                let y = layer_num - 1;
                println!("✓ Cargando {}: y={}", filename, y);
                
                for (z, line) in content.lines().enumerate() {
                    for (x, ch) in line.chars().enumerate() {
                        if let Some(material) = mat_of(ch) {
                            grid.insert(x as i32, y, z as i32, material);
                        }
                    }
                }
            }
            Err(_) => {
                println!("⚠ No se pudo cargar {}, creando capa de prueba", filename);
                if layer_num == 1 {
                    for x in 0..16 {
                        for z in 0..16 {
                            let mat = match (x + z) % 5 {
                                0 => Material::new_with_type(MaterialType::Grass),
                                1 => Material::new_with_type(MaterialType::Stone),
                                2 => Material::new_with_type(MaterialType::Gold),
                                3 => Material::new_with_type(MaterialType::GlowingObsidian),
                                _ => Material::new_with_type(MaterialType::Dirt),
                            };
                            grid.insert(x, 0, z, mat);
                        }
                    }
                }
            }
        }
    }

    println!("✓ Grilla creada con {} voxels", grid.cells.len());
    grid
}

fn main() {
    let num_threads = num_cpus::get();
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to build thread pool");

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Minecraft Diorama - Raytracer Optimizado")
        .build();

    rl.set_target_fps(60);

    let mut grid = create_diorama_grid();
    let structure_center = grid.get_center();
    let structure_radius = grid.get_bounding_sphere_radius();
    
    let bounding_sphere = BoundingSphere {
        center: structure_center,
        radius: structure_radius * 1.2,
    };
    
    println!("Centro de estructura: {:?}", structure_center);
    println!("Radio de estructura: {:.2}", structure_radius);
    
    let mut skybox_enabled = true;
    
    let camera_distance = structure_radius * 2.5;
    let mut cam = SimpleCamera::new(
        structure_center + Vector3::new(
            camera_distance * 0.7, 
            camera_distance * 0.4, 
            camera_distance * 0.7
        ),
        structure_center,
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let render_width = WIDTH / RENDER_SCALE;
    let render_height = HEIGHT / RENDER_SCALE;
    
    let mut img = Image::gen_image_color(render_width, render_height, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&thread, &img)
                    .expect("Failed to create texture");

    let mut frame_count = 0;
    let mut fps_timer = std::time::Instant::now();
    let mut render_times = vec![0.0f32; 60];
    let mut render_time_index = 0;

    println!("Raytracer Optimizado iniciado!");
    println!("   Usando {} threads para renderizado paralelo", num_threads);
    println!("   Resolución: {}x{}", render_width, render_height);

    while !rl.window_should_close() {
        handle_input_smooth(&mut cam, &rl);
        
        if rl.is_key_pressed(KeyboardKey::KEY_K) {
            skybox_enabled = !skybox_enabled;
            println!("Skybox: {}", if skybox_enabled { "Activado" } else { "Desactivado" });
        }

        let render_state = SharedRenderState {
            grid: Arc::new(grid.clone()),
            bounding_sphere,
            sun: {
                let mut sun_dir = Vector3::new(-0.6, -0.8, -0.4);
                sun_dir.normalize();
                DirLight {
                    dir: sun_dir,
                    color: Color::WHITE,
                    intensity: 1.2,
                }
            },
            skybox_enabled,
        };

        let render_start = std::time::Instant::now();
        let pixels = render_parallel_optimized(&cam, render_width, render_height, &render_state);
        let render_time = render_start.elapsed();

        render_times[render_time_index] = render_time.as_secs_f32() * 1000.0;
        render_time_index = (render_time_index + 1) % render_times.len();
        let avg_render_time: f32 = render_times.iter().sum::<f32>() / render_times.len() as f32;

        if pixels.len() == (render_width * render_height) as usize {
            let mut pixel_data = Vec::with_capacity(pixels.len() * 4);
            for color in &pixels {
                pixel_data.extend_from_slice(&[color.r, color.g, color.b, color.a]);
            }
            
            img = Image::gen_image_color(render_width, render_height, Color::BLACK);
            
            unsafe {
                if !img.data.is_null() {
                    let data_ptr = img.data as *mut u8;
                    std::ptr::copy_nonoverlapping(
                        pixel_data.as_ptr(),
                        data_ptr,
                        pixel_data.len().min((render_width * render_height * 4) as usize)
                    );
                }
            }
            
            tex = rl.load_texture_from_image(&thread, &img)
                   .expect("Failed to update texture");
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        
        d.draw_texture_pro(
            &tex, 
            Rectangle::new(0.0, 0.0, render_width as f32, render_height as f32),
            Rectangle::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32),
            Vector2::zero(), 
            0.0, 
            Color::WHITE
        );
        
        d.draw_text("Minecraft Diorama - Raytracer Optimizado", 10, 10, 20, Color::YELLOW);
        
        d.draw_text(&format!("Threads: {} | Render: {:.1}ms (avg: {:.1}ms)", 
                            num_threads,
                            render_time.as_secs_f32() * 1000.0,
                            avg_render_time), 
                   10, 35, 12, Color::LIGHTGRAY);
        
        d.draw_text("WASD: mover | QE: altura | Mouse+Click: rotar | Shift: rápido", 10, 50, 11, Color::LIGHTGRAY);
        d.draw_text("K: Toggle skybox", 10, 65, 11, Color::LIGHTGRAY);
        
        frame_count += 1;
        if fps_timer.elapsed().as_secs() >= 1 {
            d.draw_text(&format!("FPS: {}", frame_count), WIDTH - 80, 10, 20, Color::GREEN);
            frame_count = 0;
            fps_timer = std::time::Instant::now();
        }
        
        if skybox_enabled {
            d.draw_text("SKYBOX: ON", WIDTH - 150, HEIGHT - 50, 12, Color::CYAN);
        }
        
        let performance_color = if avg_render_time < 16.0 {
            Color::GREEN
        } else if avg_render_time < 33.0 {
            Color::YELLOW  
        } else {
            Color::RED
        };
        d.draw_text(&format!("Perf: {:.1}ms", avg_render_time), WIDTH - 150, HEIGHT - 20, 12, performance_color);
    }
}

fn handle_input_smooth(cam: &mut SimpleCamera, rl: &RaylibHandle) {
    let dt = rl.get_frame_time();
    let base_speed = 8.0 * dt;
    let speed = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) { 
        base_speed * 3.0 
    } else { 
        base_speed 
    };
    
    let mut eye = cam.eye;
    let mut target = cam.target;
    
    let forward = cam.forward;
    let right = cam.right;

    if rl.is_key_down(KeyboardKey::KEY_W) { 
        eye += forward * speed; 
        target += forward * speed; 
    }
    if rl.is_key_down(KeyboardKey::KEY_S) { 
        eye -= forward * speed; 
        target -= forward * speed; 
    }
    if rl.is_key_down(KeyboardKey::KEY_A) { 
        eye -= right * speed;   
        target -= right * speed; 
    }
    if rl.is_key_down(KeyboardKey::KEY_D) { 
        eye += right * speed;   
        target += right * speed; 
    }
    if rl.is_key_down(KeyboardKey::KEY_Q) { 
        eye.y -= speed; 
        target.y -= speed; 
    }
    if rl.is_key_down(KeyboardKey::KEY_E) { 
        eye.y += speed; 
        target.y += speed; 
    }

    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        let mouse_delta = rl.get_mouse_delta();
        let sensitivity = 0.003;
        
        if mouse_delta.length() > 0.01 {
            let yaw = mouse_delta.x * sensitivity;   
            let pitch = mouse_delta.y * sensitivity; 
            
            let relative = eye - target;
            let radius = relative.length();
            
            let theta = relative.z.atan2(relative.x) + yaw;
            let phi = (relative.y / radius).asin() + pitch;
            let phi = phi.clamp(-1.5, 1.5);
            
            let new_relative = Vector3::new(
                radius * phi.cos() * theta.cos(),
                radius * phi.sin(),
                radius * phi.cos() * theta.sin(),
            );
            
            eye = target + new_relative;
        }
    }

    cam.set_position(eye, target);
}