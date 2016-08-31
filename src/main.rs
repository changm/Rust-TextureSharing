extern crate glutin;
extern crate libc;
extern crate image;
extern crate gleam;
extern crate nix;
extern crate cgl;
extern crate core_foundation;
extern crate io_surface;
extern crate ipc_channel;

use ipc_channel::platform::{OsIpcChannel, OsIpcReceiverSet, OsIpcOneShotServer};
use ipc_channel::platform::{OsIpcSharedMemory, OsIpcSender};

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
fn draw_quad_to_screen(texture_id : gl::GLuint) {
    gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    gl::bind_texture(gl::TEXTURE_2D, texture_id);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
}

// Instead of drawing this to the back buffer direclty, let's draw it to an FBO
fn upload_texture(width: u32, height: u32, data: &[u8], device : &Device) -> u32 {
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

    gl::clear(gl::COLOR_BUFFER_BIT);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
    gl::flush();
    //device.end_frame();

    return texture_buffer;
}

fn upload_texture_rectangle(width: u32, height: u32, data: &[u8], device : &Device) -> u32 {
    println!("Uploading texture\n");
    // Buffers for our textures
    device.begin_frame();
    let texture_buffers = gl::gen_textures(1);
    let texture_buffer = texture_buffers[0];
    gl::enable(gl::TEXTURE_RECTANGLE);
    gl::bind_texture(gl::TEXTURE_RECTANGLE, texture_buffer);

    // Use linear filtering to scale down and up
    gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::GLint);

    // Clamp the image to border
    gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as gl::GLint);
    gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as gl::GLint);

    gl::tex_image_2d(gl::TEXTURE_RECTANGLE,
                     0,
                     gl::RGBA as gl::GLint,
                     width as gl::GLint,
                     height as gl::GLint,
                     0,
                     gl::RGBA,
                     gl::UNSIGNED_BYTE,
                     Some(data));

    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
    gl::flush();
    device.end_frame();
    return texture_buffer;
}

fn draw_image_to_screen() {
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
        gl::viewport(0, 0, 1024, 1024);
    }

    // Get the viewport size
    let viewport_size = gl::get_integer_v(gl::MAX_VIEWPORT_DIMS);

    let mut device = Device::new();
    device.setup_vao();
    device.setup_fbo_iosurface();

    // Have to do this after we create the window which loads all the symbols.
    //let texture_id = upload_texture(width, height, data.as_slice(), &device);
    let texture_id = upload_texture_rectangle(width, height, data.as_slice(), &device);

    // When we upload the texture, we have to invert the texture coordinates
    // since image data has the origin in the top left. However,
    // once we re-draw the fbo, the image is already corrected for
    // so change our vertices to not invert the texture coordinates to draw it
    // at the proper orientation.
    //device.setup_noninverting_vertices();
    //gl::bind_texture(gl::TEXTURE_RECTANGLE_ARB, device.m_shared_surface_id);
    //device.debug_shaders();

    device.setup_shared_texture_vertices();
    gl::bind_texture(gl::TEXTURE_RECTANGLE, device.m_shared_surface_id);

    for event in window.wait_events() {
        gl::clear(gl::COLOR_BUFFER_BIT);
        //draw_quad_to_screen(device.m_fbo_tex_id);
        //draw_quad_to_screen(device.m_shared_surface_id);
        //draw_quad_to_screen(texture_id);
        gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}

fn create_processes() {
    let (server, name) = OsIpcOneShotServer::new().unwrap();

    match fork().expect("fork failed") {
        ForkResult::Parent{child} => {
            let (rx, mut received_data, received_channels, received_shared_memory_regions) =
                server.accept().unwrap();
            println!("Recevived data is: {:?}", received_data);
            sleep(1);

            let (received_data, received_channels, received_shared_memory_regions) 
                = rx.recv().unwrap();
            println!("Recevived again data is: {:?}", received_data);

            println!("Parent alive, child is: {:?}", child);
            //kill(child, SIGKILL).expect("Could not kill child");
        }
        ForkResult::Child => {
            let data : &[u8] = b"HEllo from child";
            let tx = OsIpcSender::connect(name).unwrap();
            tx.send(data, vec![], vec![]).unwrap();

            let data : &[u8] = b"Try again";
            tx.send(data, vec![], vec![]).unwrap();
            unsafe { libc::exit(0); }
        }
    }
}

fn main() {
    create_processes();
}
