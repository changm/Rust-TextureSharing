extern crate gl;
extern crate glutin;
extern crate libc;
extern crate image;

use std::fs::File;
use std::path::Path;

use gl::types::*;

// Returns the raw pixel data
fn get_image_data() -> Vec<u8> {
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let img = image::open(&Path::new(image_path)).unwrap();
    return img.raw_pixels();
}

fn upload_texture(texture_data: Vec<u8>) {
    unsafe {
        let mut texture_id: GLuint = 0;
        gl::GenBuffers(1, &mut texture_id);
        println!("Generated texture id is: {:?}", texture_id);
    }
}

fn main() {
    let texture_data = get_image_data();

    let window = glutin::Window::new().unwrap();
    unsafe {
        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    // Have to do this after we create the window which loads all the symbols.
    upload_texture(texture_data);

    for event in window.wait_events() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
