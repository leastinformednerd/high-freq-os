#![no_std]
#![no_main]

mod debug_graphics;

use debug_graphics::Colour;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: limine::request::FramebufferRequest = limine::request::FramebufferRequest::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    let mut graphics_state = debug_graphics::DebugGraphicsState::new(
        FRAMEBUFFER_REQUEST.get_response().unwrap().framebuffers().next().unwrap()
    );

    graphics_state.framebuffer_as_pixel_slice()[100] = 0xffff0000;

    let style = embedded_graphics::mono_font::MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_10X20,
        <embedded_graphics::pixelcolor::Rgb888 as embedded_graphics::pixelcolor::WebColors>::CSS_WHITE
    );

    let text = embedded_graphics::text::Text::new(
        "Hello world",
        embedded_graphics::geometry::Point::new(500,500),
        style
    );

    let rect = (0..800).zip(0..800).map(|(x, y)| embedded_graphics::Pixel(
            embedded_graphics::geometry::Point{x, y}, 
            <embedded_graphics::pixelcolor::Rgb888 as embedded_graphics::pixelcolor::WebColors>::CSS_WHITE
        ));


    use embedded_graphics::draw_target::DrawTarget;
    use embedded_graphics::Drawable;
    text.draw(&mut graphics_state);

    for _ in 0..1u64<<32 {};

    panic!();
}
