extern crate glutin;
extern crate libc;
extern crate image;
extern crate gleam;

use std::fs::File;
use std::path::Path;

use gleam::gl;

// Returns the raw pixel data
fn get_image_data() -> Vec<u8> {
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let img = image::open(&Path::new(image_path)).unwrap();
    return img.raw_pixels();
}

fn upload_texture(texture_data: Vec<u8>) {
    unsafe {
        let mut texture_id: gl::GLuint = 0;
        gl::GenBuffers(1, &mut texture_id);
        println!("Generated texture id is: {:?}", texture_id);
    }
}

fn upload_triangle() {
    let vertices: [f32; 6] = [
        0.0, 0.5,   // V1
        0.5, -0.5,  // V2
        -0.5, -0.5  // V3
    ];

    unsafe {
        let mut triangle_vbo: gl::GLuint = 0;
        gl::GenBuffers(1, &mut triangle_vbo);
        println!("Generated vertex id : {:?}", triangle_vbo);

        // Now let's upload the data
        gl::BindBuffer(gl::ARRAY_BUFFER, triangle_vbo);

        // Always want a triangle
        gl::buffer_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
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
    upload_triangle();

    for event in window.wait_events() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
