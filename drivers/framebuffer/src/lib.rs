// drivers/framebuffer/lib.rs
#![no_std]

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub color: u32,
    pub fb_ptr: *mut u32,
    pub width: u64,
    pub height: u64,
}

impl Cursor {
    pub fn new(ptr: *mut u32, width: u64, height: u64) -> Self {
        Self {
            x: 0,
            y: 0,
            color: 0xFFFFFFFF, // Default White
            fb_ptr: ptr,
            width,
            height,
        }
    }

    /// Writes a single pixel to the framebuffer
    pub unsafe fn write_pixel(&self, x: usize, y: usize, color: u32) {
        if (x as u64) < self.width && (y as u64) < self.height {
            let offset = (y * self.width as usize) + x;
            // .add() is unsafe, hence the wrapping function or block
            *self.fb_ptr.add(offset) = color;
        }
    }

    /// Clears the entire screen with a specific color
    pub unsafe fn clear(&mut self, color: u32) {
        self.color = color;
        let total_pixels = (self.width * self.height) as usize;
        for i in 0..total_pixels {
            *self.fb_ptr.add(i) = self.color;
        }
    }
}
