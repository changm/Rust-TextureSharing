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

/*
pub struct Device {
    m_fbo : gl::GLuint,
    quad_vertex_shader : Option<gl::GLuint>,
    quad_fragment_shader : Option<gl::GLuint>,
    pid : gl::GLuint,

    m_vao : gl::GLuint,
    // indices
    m_ibo : gl::GLuint,
    m_vbo : gl::GLuint,
}

impl Device {
    pub fn new() -> Device {
        let mut device = Device { m_fbo: 0,
                              quad_vertex_shader : Some(0),
                              quad_fragment_shader : Some(0),
                              pid : 0,
                              m_vao : 0,
                              m_ibo : 0,
                              m_vbo : 0};

        let vertex_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/vertex.glsl");
        let fragment_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/fragment.glsl");

        // Compile our shaders
        device.quad_vertex_shader = compile_shader(&vertex_shader, gl::VERTEX_SHADER);
        device.quad_fragment_shader = compile_shader(&fragment_shader, gl::FRAGMENT_SHADER);

        // Create our program.
        device.pid = gl::create_program();
        gl::attach_shader(device.pid, device.quad_vertex_shader.unwrap());
        gl::attach_shader(device.pid, device.quad_fragment_shader.unwrap());

        // Use the program
        gl::link_program(device.pid);
        gl::use_program(device.pid);

        return device;
    }

    pub fn begin_frame(&self) {
        gl::bind_framebuffer(gl::FRAMEBUFFER, self.m_fbo);
    }

    pub fn end_frame(&self) {
        gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    }

    pub fn setup_vao(&mut self) {
        let vertices: [f32; 16] =
        [
            // vertices     // Texture coordinates, origin is bottom left, but images decode top left origin
            // So we flip our texture coordinates here instead.
            -1.0, -1.0,     0.0, 1.0,  // Bottom left
            -1.0, 1.0,      0.0, 0.0, // Top Left
            1.0, 1.0,       1.0, 0.0,    // Top right
            1.0, -1.0,      1.0, 1.0,  // bottom right
        ];

        let indices : [u32 ; 6] =
        [
            0, 1, 2, // Actually have to connect the whole screen
            2, 3, 0,
        ];

        // VAOs have to be genreated fairly early then.
        let vaos = gl::gen_vertex_arrays(1);
        self.m_vao = vaos[0];
        gl::bind_vertex_array(self.m_vao);

        // generate our FBO
        let fbos = gl::gen_framebuffers(1);
        self.m_fbo = fbos[0];
        //gl::bind_framebuffer(gl::FRAMEBUFFER, self.m_fbo);

        // Buffers for our index array
        let ibo_buffers = gl::gen_buffers(1);
        self.m_ibo = ibo_buffers[0];

        // Upload vertex data
        let vbos = gl::gen_buffers(1);
        self.m_vbo = vbos[0];
        gl::bind_buffer(gl::ARRAY_BUFFER, self.m_vbo);
        gl::buffer_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);

        // Upload our index data
        gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.m_ibo);
        gl::buffer_data(gl::ELEMENT_ARRAY_BUFFER, &indices, gl::STATIC_DRAW);

        // Now make the link between vertex data and attributes
        let vertex_count = 2;
        let vertex_stride = (mem::size_of::<f32>() * 4) as i32;
        let f32_size = mem::size_of::<f32>();
        let pos_attribute = gl::get_attrib_location(self.pid, "position");
        gl::enable_vertex_attrib_array(pos_attribute as u32);
        gl::vertex_attrib_pointer(pos_attribute as u32, vertex_count, gl::FLOAT, false, vertex_stride, 0);

        let tex_attribute = gl::get_attrib_location(self.pid, "texture_coord");
        gl::enable_vertex_attrib_array(tex_attribute as u32);
        gl::vertex_attrib_pointer(tex_attribute as u32,
                vertex_count,
                gl::FLOAT,
                false,
                vertex_stride,
                (f32_size * 2) as u32);
    }

    pub fn get_program_id(&self) -> gl::GLuint {
        return self.pid;
    }
}
*/

// Given the FBO, draw it to the screen
fn draw_quad_to_screen(fbo_id : gl::GLuint) {
    gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    gl::clear_color(1.0, 1.0, 0.0, 1.0);

    let vertices: [f32; 16] = [
        // vertices     // Texture coordinates, origin is bottom left, but images decode top left origin
                        // So we flip our texture coordinates here instead.
        -1.0, -1.0,     0.0, 1.0,  // Bottom left
        -1.0, 1.0,      0.0, 0.0, // Top Left
        1.0, 1.0,       1.0, 0.0,    // Top right
        1.0, -1.0,      1.0, 1.0,  // bottom right
    ];

    let indices : [u32 ; 6] =
    [
        0, 1, 2, // Actually have to connect the whole screen
        2, 3, 0,
    ];
}

// Instead of drawing this to the back buffer direclty, let's draw it to an FBO
fn upload_texture(width: u32, height: u32, data: &[u8], device : &Device) {
    let vertices: [f32; 16] = [
        // vertices     // Texture coordinates, origin is bottom left, but images decode top left origin
                        // So we flip our texture coordinates here instead.
        -1.0, -1.0,     0.0, 1.0,  // Bottom left
        -1.0, 1.0,      0.0, 0.0, // Top Left
        1.0, 1.0,       1.0, 0.0,    // Top right
        1.0, -1.0,      1.0, 1.0,  // bottom right
    ];

    let indices : [u32 ; 6] =
    [
        0, 1, 2, // Actually have to connect the whole screen
        2, 3, 0,
    ];

    // Buffers for our textures
    device.begin_frame();
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
    device.end_frame();
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

    // Lets just copy the blit
    gl::bind_framebuffer(gl::READ_FRAMEBUFFER, device.m_fbo);
    gl::bind_framebuffer(gl::DRAW_FRAMEBUFFER, 0);


    for event in window.wait_events() {
        //gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

        unsafe {
            // Hmm need to have the depth buffer bit?
            gl::BlitFramebuffer(0, 0, width as gl::GLint, height as gl::GLint,
                                0, 0, width as gl::GLint, height as gl::GLint,
                                gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }

        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
