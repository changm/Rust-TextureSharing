extern crate gl;
extern crate glutin;
extern crate libc;
extern crate image;

use std::fs::File;
use std::path::Path;

// Returns the raw pixel data
fn get_image_data() -> Vec<u8> {
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let img = image::open(&Path::new(image_path)).unwrap();
    return img.raw_pixels();
}

fn main() {
    get_image_data();

    let window = glutin::Window::new().unwrap();

    unsafe {
        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    for event in window.wait_events() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
