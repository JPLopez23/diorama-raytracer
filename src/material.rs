use raylib::prelude::{Color, Vector3};
use crate::texture::Texture;

#[derive(Debug, Clone, Copy)]
pub enum MaterialType {
    Grass,
    Netherrack,
    Dirt,
    Stone,
    Magma,
    Gold,
    Obsidian,
    StoneStairs,
    StoneSlab,
    StonePillar,
    WoodChest,
    GlowingObsidian,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Vector3,
    pub albedo: [f32; 4], 
    pub specular: f32,
    pub refractive_index: f32,
    pub texture: Option<Texture>,
    pub roughness: f32,
    pub metallic: f32,
    pub emission: f32,
    cached_color: Option<Vector3>,
}

impl Material {
    pub fn new(
        diffuse: Vector3, 
        specular: f32, 
        albedo: [f32; 4], 
        refractive_index: f32
    ) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
            refractive_index,
            texture: None,
            roughness: 0.5,
            metallic: 0.0,
            emission: 0.0,
            cached_color: None,
        }
    }

    pub fn with_texture(
        diffuse: Vector3, 
        specular: f32, 
        albedo: [f32; 4], 
        refractive_index: f32, 
        texture: Texture
    ) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
            refractive_index,
            texture: Some(texture),
            roughness: 0.5,
            metallic: 0.0,
            emission: 0.0,
            cached_color: None,
        }
    }

    pub fn new_with_type(material_type: MaterialType) -> Self {
        match material_type {
            MaterialType::Grass => Material {
                diffuse: Vector3::new(0.4, 0.7, 0.2),
                albedo: [0.85, 0.1, 0.02, 0.0],
                specular: 8.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.8,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.4, 0.7, 0.2)),
            },
            
            MaterialType::Netherrack => Material {
                diffuse: Vector3::new(0.6, 0.2, 0.2),
                albedo: [0.9, 0.05, 0.0, 0.0],
                specular: 4.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.9,
                metallic: 0.0,
                emission: 0.15,
                cached_color: Some(Vector3::new(0.6, 0.2, 0.2)),
            },
            
            MaterialType::Dirt => Material {
                diffuse: Vector3::new(0.5, 0.3, 0.2),
                albedo: [0.95, 0.05, 0.0, 0.0],
                specular: 2.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.95,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.5, 0.3, 0.2)),
            },
            
            MaterialType::Stone => Material {
                diffuse: Vector3::new(0.5, 0.5, 0.5),
                albedo: [0.8, 0.15, 0.05, 0.0],
                specular: 18.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.6,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.5, 0.5, 0.5)),
            },
            
            MaterialType::Magma => Material {
                diffuse: Vector3::new(0.8, 0.3, 0.1),
                albedo: [0.7, 0.1, 0.1, 0.0],
                specular: 25.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.3,
                metallic: 0.0,
                emission: 0.8,
                cached_color: Some(Vector3::new(0.8, 0.3, 0.1)),
            },
            
            MaterialType::Gold => Material {
                diffuse: Vector3::new(1.0, 0.8, 0.0),
                albedo: [0.3, 0.4, 0.3, 0.0], // Mayor reflexión (30%)
                specular: 120.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.15,
                metallic: 1.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(1.0, 0.8, 0.0)),
            },
            
            MaterialType::Obsidian => Material {
                diffuse: Vector3::new(0.15, 0.1, 0.25),
                albedo: [0.5, 0.3, 0.2, 0.0], // Reflexión moderada, sin refracción
                specular: 90.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.15,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.15, 0.1, 0.25)),
            },
            
            MaterialType::GlowingObsidian => Material {
                diffuse: Vector3::new(0.5, 0.3, 0.9),
                albedo: [0.4, 0.3, 0.3, 0.0], // Reflexión alta, sin refracción
                specular: 120.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.1,
                metallic: 0.0,
                emission: 1.2,
                cached_color: Some(Vector3::new(0.5, 0.3, 0.9)),
            },
            
            MaterialType::StoneStairs => Material {
                diffuse: Vector3::new(0.6, 0.6, 0.6),
                albedo: [0.8, 0.15, 0.05, 0.0],
                specular: 15.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.7,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.6, 0.6, 0.6)),
            },
            
            MaterialType::StoneSlab => Material {
                diffuse: Vector3::new(0.55, 0.55, 0.55),
                albedo: [0.8, 0.15, 0.05, 0.0],
                specular: 12.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.65,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.55, 0.55, 0.55)),
            },
            
            MaterialType::StonePillar => Material {
                diffuse: Vector3::new(0.7, 0.7, 0.7),
                albedo: [0.75, 0.2, 0.05, 0.0],
                specular: 22.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.4,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.7, 0.7, 0.7)),
            },
            
            MaterialType::WoodChest => Material {
                diffuse: Vector3::new(0.6, 0.4, 0.2),
                albedo: [0.85, 0.1, 0.05, 0.0],
                specular: 6.0,
                refractive_index: 1.0,
                texture: Some(Texture::new_with_type(material_type)),
                roughness: 0.75,
                metallic: 0.0,
                emission: 0.0,
                cached_color: Some(Vector3::new(0.6, 0.4, 0.2)),
            },
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Vector3::zero(),
            albedo: [0.0, 0.0, 0.0, 0.0],
            specular: 0.0,
            refractive_index: 1.0,
            texture: None,
            roughness: 1.0,
            metallic: 0.0,
            emission: 0.0,
            cached_color: Some(Vector3::zero()),
        }
    }

    pub fn get_diffuse_color_sharp(&self, u: f32, v: f32, _normal: &Vector3) -> Vector3 {
        match &self.texture {
            Some(texture) => {
                let texture_color = texture.get_nearest_color(u, v);
                texture_color * 0.98 + self.diffuse * 0.02
            },
            None => self.diffuse,
        }
    }

    pub fn get_diffuse_color(&self, u: f32, v: f32, normal: &Vector3) -> Vector3 {
        self.get_diffuse_color_sharp(u, v, normal)
    }

    pub fn get_diffuse_color_improved(&self, u: f32, v: f32, normal: &Vector3, lod_bias: f32) -> Vector3 {
        match &self.texture {
            Some(texture) => {
                let texture_color = if lod_bias > 3.0 {
                    texture.get_bilinear_color(u, v)
                } else {
                    texture.get_nearest_color(u, v)
                };
                
                texture_color * 0.96 + self.diffuse * 0.04
            },
            None => {
                self.get_procedural_variation(u, v, normal)
            },
        }
    }

    pub fn get_procedural_variation(&self, u: f32, v: f32, _normal: &Vector3) -> Vector3 {
        let noise = (u * 23.0 + v * 17.0).sin() * 0.005;
        let variation = 1.0 + noise;
        self.diffuse * variation
    }
    
    pub fn get_emission_color(&self, u: f32, v: f32, normal: &Vector3) -> Vector3 {
        if self.emission > 0.0 {
            match &self.texture {
                Some(texture) => {
                    let texture_color = texture.get_nearest_color(u, v);
                    let emission_color = self.diffuse + texture_color * 0.3;
                    emission_color * self.emission
                },
                None => {
                    let base_emission = self.get_procedural_variation(u, v, normal);
                    base_emission * self.emission
                }
            }
        } else {
            Vector3::zero()
        }
    }

    pub fn get_surface_properties(&self, u: f32, v: f32, _normal: &Vector3) -> (f32, f32) {
        match &self.texture {
            Some(texture) => {
                let texture_color = texture.get_nearest_color(u, v);
                let luminance = texture_color.x * 0.299 + texture_color.y * 0.587 + texture_color.z * 0.114;
                
                let roughness_mod = self.roughness * (0.95 + luminance * 0.1);
                let metallic_mod = self.metallic * (0.98 + luminance * 0.04);
                
                (roughness_mod, metallic_mod)
            },
            None => (self.roughness, self.metallic),
        }
    }
}

pub fn vector3_to_color(v: Vector3) -> Color {
    let gamma_corrected = Vector3::new(
        v.x.powf(1.0 / 2.1),
        v.y.powf(1.0 / 2.1), 
        v.z.powf(1.0 / 2.1)
    );
    
    Color::new(
        (gamma_corrected.x.clamp(0.0, 1.0) * 255.0) as u8,
        (gamma_corrected.y.clamp(0.0, 1.0) * 255.0) as u8,
        (gamma_corrected.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    )
}