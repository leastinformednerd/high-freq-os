use limine::framebuffer::Framebuffer;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel
};

pub struct DebugGraphicsState {
    framebuffer: Framebuffer<'static>,
    colour_handler: ColourHandler,
}

impl DebugGraphicsState {
    pub fn framebuffer_as_pixel_slice(&mut self) -> &'static mut [u32] {
        unsafe { core::slice::from_raw_parts_mut(
            self.framebuffer.addr() as *mut u32,
            (self.framebuffer.pitch() * self.framebuffer.height()) as usize
        ) }
    }

    pub fn new(framebuffer: Framebuffer<'static>) -> Self{
        Self {
            colour_handler: ColourHandler::from_framebuffer(&framebuffer),
            framebuffer
        }
    }

    pub fn make_colour(&self, red: u8, green: u8, blue: u8) -> Colour {
        self.colour_handler.make_colour(red, green, blue)
    }
    
    pub fn colour_from_rgb888(&self, p: Rgb888) -> Colour {
        self.colour_handler.colour_from_rgb888(p)
    }
}

impl OriginDimensions for DebugGraphicsState {
    fn size(&self) -> Size {
        Size { width: self.framebuffer.width() as u32, height: self.framebuffer.height() as u32}
    }
}

#[derive(Debug)]
pub enum DebugGraphicsDrawError {
    BoundsError
}

impl DrawTarget for DebugGraphicsState {
    type Color = Rgb888;
    type Error = DebugGraphicsDrawError;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>
    {
        let slice = self.framebuffer_as_pixel_slice();
        let pitch = self.framebuffer.pitch() as usize;
        let height = self.framebuffer.height() as usize;
        let width = self.framebuffer.width() as usize;
        for Pixel(Point {x, y} , colour) in pixels.into_iter() {
            let ox = x as usize;
            let oy = y as usize;
            
            if ox > width || oy > height {
                return Err(DebugGraphicsDrawError::BoundsError)
            }
            
            slice[ox + oy * pitch] = self.colour_from_rgb888(colour).0;
        }

        Ok(())
    }
}

pub struct ColourHandler {
    red_size: u8,
    green_size: u8,
    blue_size: u8,
    red_shift: u8,
    green_shift: u8,
    blue_shift: u8,
}

impl ColourHandler {
    fn from_framebuffer(framebuffer: &Framebuffer) -> Self {
        Self {
            red_size: framebuffer.red_mask_size(),
            green_size: framebuffer.green_mask_size(),
            blue_size: framebuffer.blue_mask_size(),
            red_shift: framebuffer.red_mask_shift(),
            green_shift: framebuffer.green_mask_shift(),
            blue_shift: framebuffer.blue_mask_shift(),
        }
    }
}

fn mask_colour(value: u8, mask_size: u8) -> u32 {
    (value as u32) & ((1 << (mask_size as u32)) -1)
}

impl ColourHandler {
    pub fn make_colour(&self, red: u8, blue: u8, green:u8) -> Colour {
        return Colour( ((mask_colour(red, self.red_size) as u32) << self.red_shift as usize)
            | ((mask_colour(green, self.green_size) as u32) << self.green_shift as usize)
            | ((mask_colour(blue, self.blue_size) as u32) << self.blue_shift as usize)
            | 0xff000000 // This isn't general and undercuts all this but oh well
            )
    }

    pub fn colour_from_rgb888(&self, p: Rgb888) -> Colour {
        self.make_colour(p.r(), p.b(), p.g())
    }
}

pub struct Colour(pub u32);

pub fn fill_framebuffer(state: &mut DebugGraphicsState, colour: Colour) {
    for item in state.framebuffer_as_pixel_slice().iter_mut() {
        *item = colour.0;
    }
}
