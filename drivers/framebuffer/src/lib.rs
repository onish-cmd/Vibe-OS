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
            color: 0xFFFFFFFF,
            fb_ptr: ptr,
            width,
            height,
        }
    }

    pub unsafe fn write_pixel(&self, x: usize, y: usize, color: u32) {
        // Safety check to prevent crashing if x/y are off-screen
        if (x as u64) < self.width && (y as u64) < self.height {
            let offset = (y * self.width as usize) + x;
            *self.fb_ptr.add(offset) = color;
        }
    }

    pub unsafe fn clear(color: u32) {
        cursor.color = color;
        for i in 0..(cursor.width * cursor.height) {
            unsafe {
                *cursor.fb_ptr.add(i as usize) = cursor.color;
            }
        }
    }
}
