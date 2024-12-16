#![no_std]
#![no_main]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(generic_const_exprs)]
#![feature(str_from_raw_parts)]

mod debug_graphics;

use debug_graphics::{DebugGraphicsState, TextBuffer};

use spin::{Lazy, Mutex};

use core::fmt::Write;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: limine::request::FramebufferRequest =
    limine::request::FramebufferRequest::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn print(text: &str, state: &mut debug_graphics::DebugGraphicsState) {
    let mut lock = TEXT_BUFFER.lock();

    lock.write_str(unsafe {
        core::slice::from_raw_parts(text.as_ptr() as *const core::ascii::Char, text.len())
    });

    lock.print(state);
}

macro_rules! debug_write {
    ($($arg:tt)*) => {
        TEXT_BUFFER.lock().write_fmt(format_args!($($arg)*))
    };
}

macro_rules! debug_print {
    ($graphics_state:expr, $($arg:tt)*) => {
        let mut lock = TEXT_BUFFER.lock();
        lock.write_fmt(format_args!($($arg)*));
        lock.print($graphics_state);
        core::mem::drop(lock);
    };
}

// Size of the text buffer is chosen arbitrarily
static TEXT_BUFFER: Lazy<Mutex<TextBuffer<1500>>> =
    Lazy::new(|| spin::Mutex::new(TextBuffer::new()));

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    let mut graphics_state = debug_graphics::DebugGraphicsState::new(
        FRAMEBUFFER_REQUEST
            .get_response()
            .unwrap()
            .framebuffers()
            .next()
            .unwrap(),
    );

    for (index, val) in core::iter::repeat("Hello").zip((0..100).rev()) {
        debug_print!(&mut graphics_state, "{}, {}\n", val, index);
    }

    loop {}
}
