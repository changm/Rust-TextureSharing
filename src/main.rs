extern crate glutin;
extern crate libc;
extern crate image;
extern crate gleam;
extern crate nix;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::mem;
use gleam::gl;
use nix::sys::signal::*;
use nix::unistd::*;

use device::{Device};
mod device;

// Returns the raw pixel data
fn get_image_data() -> Vec<u8> {
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let img = image::open(&Path::new(image_path)).unwrap();
    return img.raw_pixels();
}

// Given the FBO, draw it to the screen
fn draw_quad_to_screen(device : &Device) {
    gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    gl::bind_texture(gl::TEXTURE_2D, device.m_fbo_tex_id);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
}

// Instead of drawing this to the back buffer direclty, let's draw it to an FBO
fn upload_texture(width: u32, height: u32, data: &[u8], device : &Device) {
    println!("Uploading texture\n");
    // Buffers for our textures
    //device.begin_frame();
    let texture_buffers = gl::gen_textures(1);
    let texture_buffer = texture_buffers[0];
    gl::bind_texture(gl::TEXTURE_2D, texture_buffer);

    // Use linear filtering to scale down and up
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::GLint);

    // Clamp the image to border
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as gl::GLint);

    gl::tex_image_2d(gl::TEXTURE_2D,
                     0,
                     gl::RGBA as gl::GLint,
                     width as gl::GLint,
                     height as gl::GLint,
                     0,
                     gl::RGBA,
                     gl::UNSIGNED_BYTE,
                     Some(data));

    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
    //gl::bind_texture(gl::TEXTURE_2D, 0);
    //device.end_frame();
}

fn create_processes() {
    match fork().expect("fork failed") {
        ForkResult::Parent{child} => {
            sleep(5);
            println!("Parent alive, child is: {:?}", child);
            kill(child, SIGKILL).expect("Could not kill child");
        }
        ForkResult::Child => {
            println!("Child forked");
            loop {};
        }
    }
}

fn main() {
    // let's upload the image
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let mut img = image::open(&Path::new(image_path)).unwrap();

    let rgba_image = img.as_mut_rgba8().unwrap();
    let width = rgba_image.width();
    let height = rgba_image.height();
    let data = rgba_image.to_vec();
    //println!("Data is: {:?}", data);

    let window = glutin::Window::new().unwrap();
    unsafe {
        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }

    let mut device = Device::new();
    device.setup_vao();

    // Have to do this after we create the window which loads all the symbols.
    upload_texture(width, height, data.as_slice(), &device);

    //gl::bind_framebuffer(gl::FRAMEBUFFER, 0);

    for event in window.wait_events() {
        //gl::clear(gl::COLOR_BUFFER_BIT);
        gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
        //draw_quad_to_screen(&device);

        /*
        gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
        // Lets just copy the blit
        gl::bind_framebuffer(gl::READ_FRAMEBUFFER, device.m_fbo);
        gl::bind_framebuffer(gl::DRAW_FRAMEBUFFER, 0);



        unsafe {
            // Hmm need to have the depth buffer bit?
            gl::BlitFramebuffer(0, 0, width as gl::GLint, height as gl::GLint,
                                0, 0, width as gl::GLint, height as gl::GLint,
                                gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }
        */

        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
