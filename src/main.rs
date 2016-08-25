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

// Returns the raw pixel data
fn get_image_data() -> Vec<u8> {
    let image_path = "/Users/masonchang/Projects/Rust-TextureSharing/assets/firefox-256.png";
    let img = image::open(&Path::new(image_path)).unwrap();
    return img.raw_pixels();
}

// Given the FBO, draw it to the screen
fn draw_quad_to_screen(fbo_id : gl::GLuint) {
    gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    gl::clear_color(1.0, 1.0, 0.0, 1.0);

}

// Instead of drawing this to the back buffer direclty, let's draw it to an FBO
fn upload_texture(width: u32, height: u32, data: &[u8]) {
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

    let vertex_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/vertex.glsl");
    let fragment_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/fragment.glsl");

    // VAOs have to be genreated fairly early then.
    let vaos = gl::gen_vertex_arrays(1);
    let vao = vaos[0];
    gl::bind_vertex_array(vao);

    // Let's render to an FBO instead
    let fbos = gl::gen_framebuffers(1);
    let fbo = fbos[0];
    gl::bind_framebuffer(gl::FRAMEBUFFER, fbo);

    // Buffers for our index array
    let ibo_buffers = gl::gen_buffers(1);
    let ibo_buffer = ibo_buffers[0];

    // Buffers for our textures
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

    // Upload vertex data
    let vbos = gl::gen_buffers(1);
    let image_vbo = vbos[0];
    gl::bind_buffer(gl::ARRAY_BUFFER, image_vbo);
    gl::buffer_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);

    // Upload our index data
    gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, ibo_buffer);
    gl::buffer_data(gl::ELEMENT_ARRAY_BUFFER, &indices, gl::STATIC_DRAW);

    // Compile our shaders
    let vertex_shader_id = compile_shader(&vertex_shader, gl::VERTEX_SHADER);
    let fragment_shader_id = compile_shader(&fragment_shader, gl::FRAGMENT_SHADER);

    // Create our program.
    let pid = gl::create_program();
    gl::attach_shader(pid, vertex_shader_id.unwrap());
    gl::attach_shader(pid, fragment_shader_id.unwrap());

    // Bind our output, oColor is outColor defined in the fragment shader
    //gl::bind_frag_data_location(pid, 0, "oColor");

    // Use the program
    gl::link_program(pid);
    gl::use_program(pid);

    // Now make the link between vertex data and attributes
    let vertex_count = 2;
    let vertex_stride = (mem::size_of::<f32>() * 4) as i32;
    let f32_size = mem::size_of::<f32>();
    let pos_attribute = gl::get_attrib_location(pid, "position");
    gl::enable_vertex_attrib_array(pos_attribute as u32);
    gl::vertex_attrib_pointer(pos_attribute as u32, vertex_count, gl::FLOAT, false, vertex_stride, 0);

    let tex_attribute = gl::get_attrib_location(pid, "texture_coord");
    gl::enable_vertex_attrib_array(tex_attribute as u32);
    gl::vertex_attrib_pointer(tex_attribute as u32,
                              vertex_count,
                              gl::FLOAT,
                              false,
                              vertex_stride,
                              (f32_size * 2) as u32);

    // What is a VAO for again, it just remembers everything we did here?
    //gl::draw_arrays(gl::TRIANGLES, 0, 6);
    gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

    // Lets just copy teh blit
    gl::bind_framebuffer(gl::READ_FRAMEBUFFER, fbo);
    gl::bind_framebuffer(gl::DRAW_FRAMEBUFFER, 0);

/*
    unsafe {
            gl::BlitFramebuffer(0, 0, width as gl::GLint, height as gl::GLint,
                                0, 0, width as gl::GLint, height as gl::GLint,
                                gl::COLOR_BUFFER_BIT, gl::NEAREST);
    }
    */


    // Now let's draw the FBO into the normal backbuffer?
    /*
    gl::framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                               gl::TEXTURE_2D, texture_buffer, 0);
    //let draw_buffers : [gl::GLenum; 1] = [gl::COLOR_ATTACHMENT0];
    unsafe { gl::DrawBuffer(gl::COLOR_ATTACHMENT0); }
    */
}

pub fn compile_shader(shader_path: &String,
                      shader_type: gl::GLenum) -> Option<gl::GLuint> {
    let mut shader_file = File::open(&Path::new(shader_path)).unwrap();
    let mut shader_string= String::new();
    shader_file.read_to_string(&mut shader_string).unwrap();

    let id = gl::create_shader(shader_type);
    let mut source = Vec::new();
    source.extend_from_slice(shader_string.as_bytes());
    gl::shader_source(id, &[&source[..]]);
    gl::compile_shader(id);
    if gl::get_shader_iv(id, gl::COMPILE_STATUS) == (0 as gl::GLint) {
        println!("Failed to compile shader: {}", gl::get_shader_info_log(id));
        panic!("-- Shader compile failed - exiting --");
        None
    } else {
        println!("Compiled shader {}", gl::get_shader_info_log(id));
        Some(id)
    }
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

    // Have to do this after we create the window which loads all the symbols.
    upload_texture(width, height, data.as_slice());

    //upload_triangle();

    for event in window.wait_events() {
        //unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        //unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); };
        // Draw a rectangle instead.
        //gl::draw_elements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0);

        unsafe {
            gl::BlitFramebuffer(0, 0, width as gl::GLint, height as gl::GLint,
                                0, 0, width as gl::GLint, height as gl::GLint,
                                gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT, gl::NEAREST);
        }


/*
        unsafe {
            gl::BlitFramebuffer(0, 0, width as gl::GLint, height as gl::GLint,
                                0, 0, width as gl::GLint, height as gl::GLint,
                                gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }
        */

        //unsafe { gl::DrawBuffer(gl::COLOR_ATTACHMENT0); }
        //draw_quad_to_screen(0);

        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}
