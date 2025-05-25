use ratatui::{
    prelude::*,
    widgets::{ Clear, Widget},
};

use image::{RgbaImage, Rgba, ImageBuffer};
use std::io::Write;

struct KittyParallelogram {
    width: u32,
    height: u32,
    title: String,
    color: [u8; 4],
}

impl KittyParallelogram {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            title: title.to_string(),
            color: [0, 255, 255, 255], // Cyan RGBA
        }
    }
    
    fn create_parallelogram_image(&self) -> RgbaImage {
        let mut img = ImageBuffer::new(self.width, self.height);
        let bg = Rgba([0, 0, 0, 0]); // Transparent background
        let fg = Rgba(self.color);
        
        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = bg;
        }
        
        // Calculate parallelogram parameters
        let slant = self.width / 6; // How much it slants
        let inner_width = self.width - slant;
        
        // Draw the parallelogram outline
        self.draw_line(&mut img, 
            (slant as i32, 0), 
            ((self.width - 1) as i32, 0), 
            fg
        ); // Bottom
        
        self.draw_line(&mut img,
            (0, (self.height - 1) as i32),
            ((inner_width - 1) as i32, (self.height - 1) as i32),
            fg
        ); // Top
        
        self.draw_line(&mut img,
            (slant as i32, 0),
            (0, (self.height - 1) as i32),
            fg
        ); // Left diagonal
        
        self.draw_line(&mut img,
            ((self.width - 1) as i32, 0),
            ((inner_width - 1) as i32, (self.height - 1) as i32),
            fg
        ); // Right diagonal
        
        img
    }
    
    fn draw_line(&self, img: &mut RgbaImage, start: (i32, i32), end: (i32, i32), color: Rgba<u8>) {
        // Bresenham's line algorithm
        let (mut x0, mut y0) = start;
        let (x1, y1) = end;
        
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        loop {
            if x0 >= 0 && x0 < img.width() as i32 && y0 >= 0 && y0 < img.height() as i32 {
                img.put_pixel(x0 as u32, y0 as u32, color);
            }
            
            if x0 == x1 && y0 == y1 { break; }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    
    pub fn render_to_terminal(&self, x: u16, y: u16) -> std::io::Result<()> {
        let img = self.create_parallelogram_image();
        
        // Convert to PNG bytes
        let mut png_data = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut png_data);
            img.write_to(&mut cursor, image::ImageFormat::Png).expect("Failed to write PNG");
        }
        
        // Encode as base64
        let encoded = base64::encode(&png_data);
        
        // Send Kitty graphics command with positioning
        // a=T (transmit), f=100 (PNG format), C=1 (cursor movement), 
        // X=x, Y=y (position in pixels)
        print!("\x1b_Ga=T,f=100,C=1,X={},Y={};{}\x1b\\", 
            x * 8, y * 16, encoded); // Convert char coords to pixels
        std::io::stdout().flush()?;
        
        Ok(())
    }
}

pub struct GraphicalParallelogram {
    title: String,
    rendered: bool,
}

impl GraphicalParallelogram {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            rendered: false,
        }
    }
}

impl Widget for GraphicalParallelogram {
    fn render(mut self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        // Clear the area first
        Clear.render(area, buf);
        
        if !self.rendered {
            // Position cursor at the widget area
            let _ = self.render_kitty_graphic(area);
            self.rendered = true;
        }

        if !self.title.is_empty() {
            let title_area = Rect {
                x: area.x + area.width / 2 - self.title.len() as u16 / 2,
                y: area.y + area.height / 2,
                width: self.title.len() as u16,
                height: 1,
            };

            for (i, ch) in self.title.chars().enumerate() {
                let rect = Rect {
                    x: title_area.x + i as u16,
                    y: title_area.y,
                    width: 1,
                    height: 1,
                };
                if let Some(cell) = buf.cell_mut(rect) {
                    cell.set_char(ch).set_fg(Color::White);
                }
            }
        }
    }
}

impl GraphicalParallelogram {
    fn render_kitty_graphic(&self, area: Rect) -> std::io::Result<()> {
        // Create the parallelogram image
        let kitty_para = KittyParallelogram::new(&self.title, 
            (area.width * 8) as u32,  // Approximate character cell size
            (area.height * 16) as u32
        );
        
        // Position the cursor and render immediately
        print!("\x1b[{};{}H", area.y + 1, area.x + 1);
        std::io::stdout().flush()?;
        
        // Small delay to ensure cursor positioning
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        // Render the graphic
        kitty_para.render_to_terminal(area.x, area.y)?;
        
        // Force another flush
        std::io::stdout().flush()?;
        
        Ok(())
    }
}
