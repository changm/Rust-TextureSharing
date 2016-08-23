extern crate glutin;
extern crate libc;
extern crate image;
extern crate gleam;

use std::fs::File;
use std::path::Path;
use std::io::Read;

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

pub fn compile_shader(shader_path: &String,
                      shader_type: gl::GLenum) -> Option<gl::GLuint> {
    let mut shader_file = File::open(&Path::new(shader_path)).unwrap();
    let mut shader_string= String::new();
    shader_file.read_to_string(&mut shader_string).unwrap();

    // Odd that gleam::gl requires us to compile shaders as bytes and not string
    /*
    let mut source = Vec::new();
    source.extend_from_slice(vertex_string.as_bytes());
    let id = gl::create_shader(shader_type);

    let mut fragment_file = File::open(&Path::new(fragment_shader)).unwrap();
    let mut fragment_string = String::new();
    fragment_file.read_to_string(&mut vertex_string).unwrap();
    */


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

fn upload_triangle() {
    let vertices: [f32; 6] = [
        0.0, 0.5,   // V1
        0.5, -0.5,  // V2
        -0.5, -0.5  // V3
    ];

    let vertex_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/vertex.glsl");
    let fragment_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/fragment.glsl");

    let vaos = gl::gen_vertex_arrays(1);
    let vao = vaos[0];
    gl::bind_vertex_array(vao);

    let vbos = gl::gen_buffers(1);
    let triangle_vbo = vbos[0];
    println!("Generated vertex id : {:?}", triangle_vbo);

    // Now let's upload the data
    gl::bind_buffer(gl::ARRAY_BUFFER, triangle_vbo);

    // Always want a triangle
    gl::buffer_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);

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
    let pos_attribute = gl::get_attrib_location(pid, "position");
    gl::enable_vertex_attrib_array(pos_attribute as u32);
    gl::vertex_attrib_pointer(pos_attribute as u32, 2, gl::FLOAT, false, 0, 0);

    // What is a VAO for again, it just remembers everything we did here?
    gl::draw_arrays(gl::TRIANGLES, 0, 3);
}

fn main() {
    let texture_data = get_image_data();

    let window = glutin::Window::new().unwrap();
    unsafe {
        window.make_current();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    // Have to do this after we create the window which loads all the symbols.
    //upload_texture(texture_data);

    upload_triangle();
    
    for event in window.wait_events() {
        //unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); };
        gl::draw_arrays(gl::TRIANGLES, 0, 3);
        window.swap_buffers();

        match event {
            glutin::Event::Closed => break,
            _ => ()
        }
    }
}