/// In-memory representation of the 10k×10k canvas.
/// Each pixel is stored as a u32 (RGBA).
/// In production, this would be backed by Redis or a memory-mapped file.
pub struct CanvasBuffer {
    /// 10_000 × 10_000 pixels, row-major order.
    pixels: Vec<u32>,
    width: u32,
    height: u32,
}

impl CanvasBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![0xFFFFFFFF; (width * height) as usize], // white
            width,
            height,
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    /// Export a rectangular region as raw bytes for snapshot/streaming.
    pub fn export_region(&self, x: u32, y: u32, w: u32, h: u32) -> Vec<u32> {
        let mut region = Vec::with_capacity((w * h) as usize);
        for row in y..(y + h).min(self.height) {
            for col in x..(x + w).min(self.width) {
                region.push(self.pixels[(row * self.width + col) as usize]);
            }
        }
        region
    }
}
