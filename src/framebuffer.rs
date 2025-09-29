// framebuffer.rs - Versión Optimizada con Buffer Reutilizable

use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    // Cache para texture reutilizable
    cached_texture: Option<Texture2D>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
            cached_texture: None,
        }
    }

    pub fn clear(&mut self) {
        // Optimización: reutilizar el buffer existente en lugar de crear uno nuevo
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                self.color_buffer.draw_pixel(x, y, self.background_color);
            }
        }
    }

    #[inline] // Inline para mejor rendimiento en el bucle principal
    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    #[inline]
    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn _render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }

    // Versión optimizada que reutiliza texturas cuando es posible
    pub fn swap_buffers(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        match &self.cached_texture {
            Some(_) => {
                if let Ok(new_texture) = rl.load_texture_from_image(thread, &self.color_buffer) {
                    self.cached_texture = Some(new_texture);
                }
            },
            None => {
                if let Ok(new_texture) = rl.load_texture_from_image(thread, &self.color_buffer) {
                    self.cached_texture = Some(new_texture);
                }
            }
        }

        if let Some(texture) = &self.cached_texture {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            
            // Escalar con suavizado
            d.draw_texture_pro(
                texture,
                Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
                Rectangle::new(0.0, 0.0, d.get_screen_width() as f32, d.get_screen_height() as f32),
                Vector2::zero(),
                0.0,
                Color::WHITE
            );
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        // Limpiar recursos si es necesario
        // Raylib maneja automáticamente la limpieza de texturas
    }
}