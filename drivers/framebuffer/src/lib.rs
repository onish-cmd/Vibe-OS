// drivers/framebuffer/lib.rs
#![no_std]

#[repr(C)]
pub struct Psf2Header {
    pub magic: [u8; 4],
    pub version: u32,
    pub header_size: u32,
    pub flags: u32,
    pub length: u32,        // Number of glyphs
    pub char_size: u32,     // Bytes per glyph
    pub height: u32,        // Height in pixels
    pub width: u32,         // Width in pixels
}

const PSF2_MAGIC: [u8; 4] = [0x72, 0xb5, 0x4a, 0x86];

pub struct Font<'a> {
    pub header: &'a Psf2Header,
    pub glyphs: &'a [u8],
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header = unsafe { &*(data.as_ptr() as *const Psf2Header) };
        
        // Safety check: verify the magic number
        if header.magic != PSF2_MAGIC {
            panic!("Invalid PSF2 font!");
        }

        // The glyphs start after the header
        let glyphs_start = header.header_size as usize;
        let glyphs = &data[glyphs_start..];

        Self { header, glyphs }
    }

    pub fn get_glyph(&self, c: char) -> &[u8] {
        let index = c as usize;
        let start = index * self.header.char_size as usize;
        let end = start + self.header.char_size as usize;
        &self.glyphs[start..end]
    }
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub color: u32,
    pub color_fg: u32,
    pub fb_ptr: *mut u32,
    pub width: u64,
    pub height: u64,
    pub font: Option<Font<'static>>,
}

impl Cursor {
    pub fn new(ptr: *mut u32, width: u64, height: u64) -> Self {
        Self {
            x: 0,
            y: 0,
            color: 0xFFFFFFFF, // Default White
            color_fg: 0xFF000000, // Default black
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

   pub fn draw_char(&mut self, c: char) {
    // 1. Check for overflow BEFORE borrowing the font
    let f_height = if let Some(ref f) = self.font { f.header.height as usize } else { 16 };
    
    if self.y + f_height > self.height as usize {
        self.y = 0;
        self.x = 0;
        unsafe { self.clear(self.color); } // Added unsafe block
    }

    // 2. Now borrow the font for the actual drawing
    if let Some(ref font) = self.font {
        let f_width = font.header.width as usize;
        let bytes_per_row = (font.header.width + 7) / 8;

        if c == '\n' {
            self.x = 0;
            self.y += f_height;
            return;
        }

        if self.x + f_width > self.width as usize {
            self.x = 0;
            self.y += f_height;
        }

        let glyph = font.get_glyph(c);
        for py in 0..font.header.height {
            for px in 0..font.header.width {
                let byte_offset = (py * bytes_per_row + px / 8) as usize;
                let bit_offset = 7 - (px % 8);
                let bit_is_set = (glyph[byte_offset] >> bit_offset) & 1;

                if bit_is_set == 1 {
                    unsafe {
                        self.write_pixel(self.x + px as usize, self.y + py as usize, self.color_fg);
                    }
                }
            }
        }
        self.x += f_width;
    }
} 
}
