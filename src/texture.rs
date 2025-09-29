use raylib::prelude::*;
use crate::material::MaterialType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref TEXTURE_CACHE: Arc<Mutex<HashMap<String, Arc<TextureData>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Clone)]
struct TextureData {
    pixels: Vec<Color>,
    width: i32,
    height: i32,
    fallback_color: Vector3,
}

#[derive(Debug, Clone)]
pub struct Texture {
    data: Arc<TextureData>,
}

impl Texture {
    pub fn solid(color: Vector3) -> Self {
        let data = TextureData {
            pixels: vec![Color::new(
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8, 
                (color.z * 255.0) as u8,
                255
            )],
            width: 1,
            height: 1,
            fallback_color: color,
        };
        
        Texture {
            data: Arc::new(data)
        }
    }

    pub fn new_with_type(material_type: MaterialType) -> Self {
        let (filename, fallback_color) = Self::get_material_info(material_type);
        Self::load_cached_texture(&filename, fallback_color)
    }

    fn get_material_info(material_type: MaterialType) -> (String, Vector3) {
        match material_type {
            MaterialType::Grass => ("grass.png".to_string(), Vector3::new(0.4, 0.7, 0.2)),
            MaterialType::Netherrack => ("netherrack.png".to_string(), Vector3::new(0.6, 0.2, 0.2)),
            MaterialType::Dirt => ("dirt.png".to_string(), Vector3::new(0.5, 0.3, 0.2)),
            MaterialType::Stone => ("stone.png".to_string(), Vector3::new(0.5, 0.5, 0.5)),
            MaterialType::Magma => ("magma.png".to_string(), Vector3::new(0.8, 0.3, 0.1)),
            MaterialType::Gold => ("gold.png".to_string(), Vector3::new(1.0, 0.8, 0.0)),
            MaterialType::Obsidian => ("obsidian.png".to_string(), Vector3::new(0.1, 0.1, 0.2)),
            MaterialType::GlowingObsidian => ("glowing_obsidian.png".to_string(), Vector3::new(0.4, 0.2, 0.8)),
            MaterialType::StoneStairs => ("stone_stairs.png".to_string(), Vector3::new(0.6, 0.6, 0.6)),
            MaterialType::StoneSlab => ("stone_slab.png".to_string(), Vector3::new(0.55, 0.55, 0.55)),
            MaterialType::StonePillar => ("stone_pillar.png".to_string(), Vector3::new(0.7, 0.7, 0.7)),
            MaterialType::WoodChest => ("wood_chest.png".to_string(), Vector3::new(0.6, 0.4, 0.2)),
        }
    }

    fn load_cached_texture(filename: &str, fallback_color: Vector3) -> Self {
        // Verificar cache
        {
            let cache = TEXTURE_CACHE.lock().unwrap();
            if let Some(cached_data) = cache.get(filename) {
                return Texture {
                    data: cached_data.clone()
                };
            }
        }

        let image_path = format!("./images/{}", filename);
        println!("Intentando cargar: {}", image_path);
        
        // Intentar cargar TODAS las imágenes automáticamente
        let texture_data = match image::open(&image_path) {
            Ok(img) => {
                let rgba_img = img.to_rgba8();
                let (width, height) = rgba_img.dimensions();
                
                println!("✓ Cargando imagen real: {} ({}x{})", filename, width, height);
                
                let mut pixels = Vec::with_capacity((width * height) as usize);
                for pixel in rgba_img.pixels() {
                    let [r, g, b, a] = pixel.0;
                    pixels.push(Color::new(r, g, b, a));
                }
                
                println!("✓ Imagen cargada exitosamente: {}", filename);
                
                TextureData {
                    pixels,
                    width: width as i32,
                    height: height as i32,
                    fallback_color,
                }
            },
            Err(e) => {
                println!("⚠ Error al cargar {} ({}), usando procedural", filename, e);
                Self::create_procedural_texture(filename, fallback_color)
            }
        };

        let arc_data = Arc::new(texture_data);
        
        // Guardar en cache
        {
            let mut cache = TEXTURE_CACHE.lock().unwrap();
            cache.insert(filename.to_string(), arc_data.clone());
        }

        Texture {
            data: arc_data
        }
    }

    fn create_procedural_texture(filename: &str, base_color: Vector3) -> TextureData {
        let size = 16i32;
        let mut pixels = Vec::with_capacity((size * size) as usize);
        
        println!("Creando textura procedural para: {}", filename);
        
        for y in 0..size {
            for x in 0..size {
                let pattern = Self::get_pattern_for_material(filename, x, y, size);
                let color_variation = base_color * pattern;
                
                pixels.push(Color::new(
                    (color_variation.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (color_variation.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (color_variation.z.clamp(0.0, 1.0) * 255.0) as u8,
                    255
                ));
            }
        }
        
        TextureData {
            pixels,
            width: size,
            height: size,
            fallback_color: base_color,
        }
    }

    fn get_pattern_for_material(filename: &str, x: i32, y: i32, size: i32) -> f32 {
        let fx = x as f32 / size as f32;
        let fy = y as f32 / size as f32;
        
        match filename {
            name if name.contains("grass") => {
                let noise = (fx * 8.0 + fy * 6.0).sin() * 0.2;
                0.8 + noise
            },
            name if name.contains("stone") => {
                let block_x = (fx * 4.0).floor();
                let block_y = (fy * 4.0).floor();
                let checker = ((block_x + block_y) % 2.0) * 0.1;
                0.9 + checker
            },
            name if name.contains("gold") => {
                let shine = (fx * 16.0).sin() * (fy * 16.0).cos() * 0.15;
                1.0 + shine
            },
            name if name.contains("obsidian") => {
                let reflection = (fx * 12.0 + fy * 8.0).sin() * 0.3;
                0.7 + reflection
            },
            name if name.contains("glowing") => {
                let pulse = (fx * 6.0 + fy * 6.0).sin() * 0.4;
                1.2 + pulse
            },
            name if name.contains("magma") => {
                let heat = (fx * 10.0).sin() * (fy * 8.0).cos() * 0.3;
                1.0 + heat
            },
            name if name.contains("dirt") => {
                let grain = (fx * 12.0 + fy * 10.0).sin() * 0.15;
                0.85 + grain
            },
            name if name.contains("netherrack") => {
                let roughness = (fx * 14.0).cos() * (fy * 11.0).sin() * 0.25;
                0.9 + roughness
            },
            _ => {
                let default_pattern = (fx * 8.0 + fy * 8.0).sin() * 0.1;
                0.9 + default_pattern
            }
        }
    }

    #[inline(always)]
    pub fn get_nearest_color(&self, u: f32, v: f32) -> Vector3 {
        if self.data.pixels.len() <= 1 {
            return self.data.fallback_color;
        }
        
        let u_wrapped = u.fract().abs();
        let v_wrapped = v.fract().abs();
        
        let x = (u_wrapped * self.data.width as f32) as i32;
        let y = (v_wrapped * self.data.height as f32) as i32;
        let x = x.clamp(0, self.data.width - 1);
        let y = y.clamp(0, self.data.height - 1);
        
        let index = (y * self.data.width + x) as usize;
        
        if index < self.data.pixels.len() {
            let color = self.data.pixels[index];
            Vector3::new(
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
            )
        } else {
            self.data.fallback_color
        }
    }

    #[inline]
    pub fn get_bilinear_color(&self, u: f32, v: f32) -> Vector3 {
        if self.data.pixels.len() <= 1 {
            return self.data.fallback_color;
        }
        
        let u_wrapped = u.fract().abs();
        let v_wrapped = v.fract().abs();
        
        let tex_x = u_wrapped * (self.data.width - 1) as f32;
        let tex_y = v_wrapped * (self.data.height - 1) as f32;
        
        let x0 = tex_x.floor() as i32;
        let y0 = tex_y.floor() as i32;
        let x1 = (x0 + 1).min(self.data.width - 1);
        let y1 = (y0 + 1).min(self.data.height - 1);
        
        let fx = tex_x - x0 as f32;
        let fy = tex_y - y0 as f32;
        
        let get_pixel = |x: i32, y: i32| -> Vector3 {
            let index = (y * self.data.width + x) as usize;
            if index < self.data.pixels.len() {
                let color = self.data.pixels[index];
                Vector3::new(
                    color.r as f32 / 255.0,
                    color.g as f32 / 255.0,
                    color.b as f32 / 255.0,
                )
            } else {
                self.data.fallback_color
            }
        };
        
        let c00 = get_pixel(x0, y0);
        let c10 = get_pixel(x1, y0);
        let c01 = get_pixel(x0, y1);
        let c11 = get_pixel(x1, y1);
        
        let c0 = c00 * (1.0 - fx) + c10 * fx;
        let c1 = c01 * (1.0 - fx) + c11 * fx;
        
        c0 * (1.0 - fy) + c1 * fy
    }

    #[inline(always)]
    pub fn get_fast_color(&self, u: f32, v: f32) -> Vector3 {
        self.get_nearest_color(u, v)
    }

    #[inline(always)]
    pub fn get_balanced_color(&self, u: f32, v: f32) -> Vector3 {
        self.get_nearest_color(u, v)
    }

    #[inline]
    pub fn get_color_with_lod(&self, u: f32, v: f32, lod_bias: f32) -> Vector3 {
        if lod_bias > 2.0 {
            self.get_bilinear_color(u, v)
        } else {
            self.get_nearest_color(u, v)
        }
    }

    #[inline]
    pub fn get_bicubic_color(&self, u: f32, v: f32) -> Vector3 {
        self.get_bilinear_color(u, v)
    }

    pub fn get_high_quality_color(&self, u: f32, v: f32, _normal: &Vector3) -> Vector3 {
        self.get_nearest_color(u, v)
    }

    pub fn get_trilinear_color(&self, u: f32, v: f32, _lod: f32) -> Vector3 {
        self.get_nearest_color(u, v)
    }

    fn sample_mipmap_level(&self, u: f32, v: f32, _level: usize) -> Vector3 {
        self.get_nearest_color(u, v)
    }
}

pub fn clear_texture_cache() {
    let mut cache = TEXTURE_CACHE.lock().unwrap();
    cache.clear();
    println!("Cache de texturas limpiado");
}

pub fn get_cache_stats() -> (usize, usize) {
    let cache = TEXTURE_CACHE.lock().unwrap();
    let num_textures = cache.len();
    let total_pixels: usize = cache.values()
        .map(|texture| texture.pixels.len())
        .sum();
    (num_textures, total_pixels)
}