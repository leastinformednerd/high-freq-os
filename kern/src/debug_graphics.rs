use core::borrow::BorrowMut;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};
use limine::framebuffer::Framebuffer;

pub struct DebugGraphicsState {
    framebuffer: Framebuffer<'static>,
    colour_handler: ColourHandler,
}

impl DebugGraphicsState {
    pub fn framebuffer_as_pixel_slice(&mut self) -> &'static mut [u32] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.framebuffer.addr() as *mut u32,
                (self.framebuffer.pitch() * self.framebuffer.height()) as usize,
            )
        }
    }

    fn fill(&mut self, colour: Colour) {
        fill_framebuffer(self, colour);
    }

    pub fn new(framebuffer: Framebuffer<'static>) -> Self {
        Self {
            colour_handler: ColourHandler::from_framebuffer(&framebuffer),
            framebuffer,
        }
    }

    pub fn make_colour(&self, red: u8, green: u8, blue: u8) -> Colour {
        self.colour_handler.make_colour(red, green, blue)
    }

    // This probably gets inlined anyway but always good to hint :)
    #[inline]
    pub fn colour_from_rgb888(&self, p: Rgb888) -> Colour {
        self.colour_handler.colour_from_rgb888(p)
    }
}

impl OriginDimensions for DebugGraphicsState {
    fn size(&self) -> Size {
        Size {
            width: self.framebuffer.width() as u32,
            height: self.framebuffer.height() as u32,
        }
    }
}

#[derive(Debug)]
pub enum DebugGraphicsDrawError {
    BoundsError { x: usize, y: usize },
}

impl DrawTarget for DebugGraphicsState {
    type Color = Rgb888;
    type Error = DebugGraphicsDrawError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let slice = self.framebuffer_as_pixel_slice();
        let pitch = self.framebuffer.pitch() as usize / 4;
        let height = self.framebuffer.height() as usize;
        let width = self.framebuffer.width() as usize;

        for Pixel(Point { x, y }, colour) in pixels.into_iter() {
            let ox = x as usize;
            let oy = y as usize;

            if ox >= width || oy >= height {
                return Err(DebugGraphicsDrawError::BoundsError { x: ox, y: oy });
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
    (value as u32) & ((1 << (mask_size as u32)) - 1)
}

impl ColourHandler {
    #[inline]
    pub fn make_colour(&self, red: u8, blue: u8, green: u8) -> Colour {
        return Colour(
            ((mask_colour(red, self.red_size) as u32) << self.red_shift as usize)
                | ((mask_colour(green, self.green_size) as u32) << self.green_shift as usize)
                | ((mask_colour(blue, self.blue_size) as u32) << self.blue_shift as usize)
                | 0xff000000, // This isn't general and undercuts all this but oh well
        );
    }

    #[inline]
    pub fn colour_from_rgb888(&self, p: Rgb888) -> Colour {
        self.make_colour(p.r(), p.b(), p.g())
    }
}

#[derive(Clone)]
pub struct Colour(pub u32);

pub fn fill_framebuffer(state: &mut DebugGraphicsState, colour: Colour) {
    for item in state.framebuffer_as_pixel_slice().iter_mut() {
        *item = colour.0;
    }
}

/// A buffer of (ascii) characters that can be written to the screen
pub struct TextBuffer<const BufferSize: usize> {
    buffer: [u8; BufferSize],
    cursor: usize,
    shifted: bool,
}

#[derive(Debug)]
pub enum TextWritingError {
    TooLong,
}

const TEXT_STYLE: embedded_graphics::mono_font::MonoTextStyle<Rgb888> =
    embedded_graphics::mono_font::MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_10X20,
        Rgb888::new(255, 255, 255),
    );

impl<const BufferSize: usize> TextBuffer<BufferSize> {
    pub fn new() -> TextBuffer<BufferSize> {
        TextBuffer {
            buffer: [255; BufferSize],
            cursor: 0,
            shifted: false,
        }
    }

    pub fn write_str<'a>(
        &mut self,
        string: &[core::ascii::Char],
    ) -> Result<&'a str, TextWritingError> {
        let space_required = string.len() + 2;
        if space_required > BufferSize || space_required > u16::MAX as usize {
            return Err(TextWritingError::TooLong);
        }

        if space_required < 10 {
            gdb_pause(());
        }

        if self.cursor + space_required >= BufferSize {
            self.remove_to_fit(space_required);
        }

        self.buffer[self.cursor] = (space_required >> 8) as u8;
        self.buffer[self.cursor + 1] = (space_required & 0xff) as u8;
        for i in 0..string.len() {
            self.buffer[self.cursor + 2 + i] = string[i] as u8;
        }

        let val = unsafe { core::str::from_raw_parts(self.buffer.as_ptr().add(2), space_required) };

        self.cursor = self.cursor + space_required;

        Ok(val)
    }

    pub fn print(&mut self, state: &mut DebugGraphicsState) -> Result<(), DebugGraphicsDrawError> {
        use embedded_graphics::{
            mono_font::{ascii::FONT_9X18, MonoTextStyleBuilder},
            text::Text,
            Drawable,
        };
        /*
        if self.shifted {
            state.fill(Colour(0));
            self.shifted = false;
        }
        */
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_9X18)
            .text_color(Rgb888::WHITE)
            .background_color(Rgb888::BLACK)
            .build();

        let mut line_no = 0;
        let mut x = 9;
        for text in self.into_iter() {
            let to_draw = Text::new(
                text,
                Point {
                    x: x as i32,
                    y: ((line_no + 1) * 18) as i32,
                },
                text_style,
            );

            if text.as_bytes()[text.len() - 1] == b'\n' {
                line_no += 1;
                x = 9;
            } else {
                x += 9 * text.len();
                if x > 1000 {
                    line_no += 1;
                    x = 9;
                }
            }

            to_draw.draw(state)?;
        }

        Ok(())
    }

    /// Remove enough strings from the front of the buffer to fit space_required new bytes at the end
    ///
    /// Assumes BufferSize > space_required, else it will panic!
    fn remove_to_fit(&mut self, space_required: usize) {
        let mut bytes = 0;
        while bytes < space_required {
            bytes += ((self.buffer[bytes] as u16) << 8 | self.buffer[bytes + 1] as u16) as usize;
        }

        self.shifted = true;

        self.cursor -= bytes;

        //gdb_pause((&self, bytes));
        unsafe {
            core::ptr::copy(
                self.buffer.as_ptr().add(bytes),
                self.buffer.as_mut_ptr(),
                BufferSize - bytes,
            );

            core::intrinsics::write_bytes(
                self.buffer.as_mut_ptr().add(self.cursor),
                255,
                BufferSize - self.cursor,
            );
        }
    }
}

impl<const BufSize: usize> core::fmt::Write for TextBuffer<BufSize> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if !s.is_ascii() {
            return Err(core::fmt::Error);
        }

        unsafe {
            match self.write_str(core::slice::from_raw_parts(
                s.as_ptr() as *const core::ascii::Char,
                s.len(),
            )) {
                Ok(_) => Ok(()),
                Err(_) => Err(core::fmt::Error),
            }
        }
    }
}

impl<'a, const BufSize: usize> IntoIterator for &'a TextBuffer<BufSize> {
    type Item = &'a str;
    type IntoIter = TextBufferIter<'a, BufSize>;

    fn into_iter(self) -> Self::IntoIter {
        return TextBufferIter {
            strings: &self.buffer,
            cursor: 0,
        };
    }
}

pub struct TextBufferIter<'a, const BufSize: usize> {
    strings: &'a [u8; BufSize],
    cursor: usize,
}

fn gdb_pause<T>(i: T) -> T {
    i
}

static mut BLA: (usize, usize, usize) = (0, 0, 0);

impl<'a, const BufSize: usize> Iterator for TextBufferIter<'a, BufSize> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.cursor + 2 >= BufSize {
            return None;
        }

        let len = ((self.strings[self.cursor] as u16) << 8 | self.strings[self.cursor + 1] as u16)
            as usize
            - 2;

        // This means the data is unitialised (since it's impossible for this to occur otherwise)
        // and it's known that this will happen if it's uninited since the default guard value is
        // u8::MAX and so len will be u16::MAX-2 > BufSize on all reason buffer sizes for this
        // module
        if len > BufSize {
            return None;
        }

        let string = unsafe {
            core::str::from_raw_parts(
                self.strings.as_ptr().add(self.cursor + 2),
                if len + self.cursor + 2 > BufSize {
                    BLA = (len, self.cursor, BufSize - self.cursor);
                    BufSize - self.cursor
                } else {
                    len
                },
            )
        };

        self.cursor += 2 + len;

        Some(string)
    }
}
