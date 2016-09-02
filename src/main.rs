extern crate glutin;
extern crate libc;
extern crate image;
extern crate gleam;
extern crate nix;
extern crate cgl;
extern crate core_foundation;
extern crate io_surface;
extern crate ipc_channel;
#[macro_use] extern crate enum_primitive;
use enum_primitive::FromPrimitive;

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

// Sad that I need a crate to convert a u8 to a message.
enum_from_primitive! {
#[derive(Debug,PartialEq)]
enum Message {
    QUIT,
    PARENT_RENDER,
    CHILD_RENDER,
}
}

// Returns the raw pixel data
fn get_image_data() -> Vec<u8> {
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let img = image::open(&Path::new(image_path)).unwrap();
    return img.raw_pixels();
}

fn upload_texture_rectangle(device : &Device) -> u32 {
    // let's upload the image
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let mut img = image::open(&Path::new(image_path)).unwrap();

    let rgba_image = img.as_mut_rgba8().unwrap();
    let width = rgba_image.width();
    let height = rgba_image.height();
    let data = rgba_image.to_vec();

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
                     Some(data.as_slice()));

    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
    gl::flush();
    device.end_frame();
    return texture_buffer;
}

fn create_glutin_window() -> glutin::Window {
    let window = glutin::Window::new().unwrap();
    println!("Created window\n");

    unsafe {
        println!("Making current \n");
        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::viewport(0, 0, 1024, 1024);
    }

    return window;
}

fn redraw_parent_from_shared_iosurface(window: &glutin::Window, device : &mut Device) {
    device.setup_shared_texture_vertices();
    gl::bind_texture(gl::TEXTURE_RECTANGLE, device.m_shared_gl_texture_id);

    gl::clear(gl::COLOR_BUFFER_BIT);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

    for event in window.wait_events() {
        gl::clear(gl::COLOR_BUFFER_BIT);
        gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            glutin::Event::Awakened => break,
            _ => ()
        }
    }
}

fn setup_parent(window : &glutin::Window, device : &mut Device) {
    device.setup_vao();
    device.setup_iosurface();
    device.setup_fbo_iosurface();

    // Have to do this after we create the window which loads all the symbols.
    //let texture_id = upload_texture_rectangle(&device);
    redraw_parent_from_shared_iosurface(window, device);
}

fn draw_image_to_screen(window : &glutin::Window, device : &mut Device) {
    // Get the viewport size
    let viewport_size = gl::get_integer_v(gl::MAX_VIEWPORT_DIMS);

    device.setup_vao();
    device.setup_iosurface();
    device.setup_fbo_iosurface();

    // Have to do this after we create the window which loads all the symbols.
    device.setup_shared_texture_vertices();
    gl::bind_texture(gl::TEXTURE_RECTANGLE, device.m_shared_gl_texture_id);

    gl::clear(gl::COLOR_BUFFER_BIT);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

    for event in window.wait_events() {
        gl::clear(gl::COLOR_BUFFER_BIT);
        gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            glutin::Event::Awakened => break,
            _ => ()
        }
    }
}

fn child_render(shared_surface_id : u8) {
    let window = create_glutin_window();
    let mut child_device = Device::new();
    child_device.setup_vao();
    child_device.connect_iosurface(shared_surface_id);
    child_device.setup_fbo_iosurface();

    let image_texture = upload_texture_rectangle(&child_device);
    child_device.setup_shared_texture_vertices();
    gl::bind_texture(gl::TEXTURE_RECTANGLE, child_device.m_shared_gl_texture_id);

    gl::clear(gl::COLOR_BUFFER_BIT);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
    gl::flush();
}

fn create_processes() {
    let (server, name) = OsIpcOneShotServer::new().unwrap();
    match fork().expect("fork failed") {
        ForkResult::Parent{child} => {
            let window = create_glutin_window();
            let mut device = Device::new();
            setup_parent(&window, &mut device);
            let iosurface_id = device.m_shared_iosurface_id;
            println!("Shared surface id parent: {:?}", iosurface_id);
            sleep(2);

            let (rx, mut received_data, mut received_channels, received_shared_memory_regions) =
                server.accept().unwrap();
            // Have to receive the tx channel from the child
            let tx = received_channels.pop().unwrap().to_sender();
            let data : &[u8] = &[Message::CHILD_RENDER as u8, iosurface_id as u8];
            tx.send(data, vec![], vec![]);

            loop {
                println!("Parent waiting to receive something\n");
                let (received_data, received_channels, received_shared_memory_regions)
                    = rx.recv().unwrap();
                println!("Parent received: {:?}", received_data);

                let message_data = received_data[0];
                let received_message : Message = Message::from_u8(message_data).unwrap();
                match received_message {
                    Message::QUIT => {
                        println!("Child received quite message");
                        unsafe { libc::exit(0); }
                    },
                    Message::PARENT_RENDER => println!("Parent:: received parent render"),
                    Message::CHILD_RENDER => println!("Parent needs to render"),
                }

                println!("Parent rendering to screen\n");
                redraw_parent_from_shared_iosurface(&window, &mut device);
            }
        }
        ForkResult::Child => {
            let data : &[u8] = b"HEllo from child";
            let super_tx = OsIpcSender::connect(name).unwrap();
            let (tx, rx) = ipc_channel::platform::channel().unwrap();
            // Send the channel to the parent, I don't know what tx is actually for
            super_tx.send(data, vec![OsIpcChannel::Sender(tx)], vec![]);

            loop {
                let (received_data, received_channels, received_shared_memory_regions)
                    = rx.recv().unwrap();
                println!("Child received: {:?}", received_data);

                let message_data = received_data[0];
                let received_message : Message = Message::from_u8(message_data).unwrap();
                match received_message {
                    Message::QUIT => {
                        println!("Child received quite message");
                        unsafe { libc::exit(0); }
                    },
                    Message::PARENT_RENDER => println!("Child:: received parent render"),
                    Message::CHILD_RENDER => {
                        let shared_surface_id = received_data[1];
                        println!("Child surface: {:?}", shared_surface_id);
                        child_render(shared_surface_id);

                        let data : &[u8] = &[Message::PARENT_RENDER as u8];
                        super_tx.send(data, vec![], vec![]);
                    },
                }
            } // end loop
        } // end child
    } // End fork
}

fn main() {
    //draw_image_to_screen();
    create_processes();
}
