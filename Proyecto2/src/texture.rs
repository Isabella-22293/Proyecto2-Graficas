use crate::color::Color;

#[derive(Debug, Clone)] // Añadido Clone aquí
pub struct Texture {
    data: Vec<Color>, // Los colores de la textura
    width: usize,
    height: usize,
}

impl Texture {
    pub fn new(data: Vec<Color>, width: usize, height: usize) -> Self {
        assert!(data.len() == width * height, "El tamaño de los datos no coincide con las dimensiones de la textura.");
        Texture { data, width, height }
    }

    pub fn get_color_at(&self, u: f32, v: f32) -> Color {
        if self.data.is_empty() {
            return Color::black();
        }

        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let x = (u * self.width as f32) as usize;
        let y = (v * self.height as f32) as usize;

        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);

        self.data[y * self.width + x]
    }
}
