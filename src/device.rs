extern crate libc;
extern crate gleam;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::mem;
use gleam::gl;

fn compile_shader(shader_path: &String,
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

pub struct Device {
    pub m_fbo : gl::GLuint,
    pub m_fbo_tex_id : gl::GLuint,
    quad_vertex_shader : Option<gl::GLuint>,
    quad_fragment_shader : Option<gl::GLuint>,
    pid : gl::GLuint,

    m_vao : gl::GLuint,
    // indices
    m_ibo : gl::GLuint,
    m_vbo : gl::GLuint,
}

impl Drop for Device {
    fn drop (&mut self) {
        let frame_buffers = [self.m_fbo];
        gl::delete_framebuffers(&frame_buffers);

        gl::delete_shader(self.quad_vertex_shader.unwrap());
        gl::delete_shader(self.quad_fragment_shader.unwrap());
        gl::delete_program(self.pid);

        let vertex_arrays = [self.m_vao];
        gl::delete_vertex_arrays(&vertex_arrays);

        let vbo_ibo = [self.m_ibo, self.m_vbo];
        gl::delete_buffers(&vbo_ibo);
    }
}

impl Device {
    pub fn new() -> Device {
        println!("Creating new device\n");
        let mut device = Device { m_fbo: 0,
                                  m_fbo_tex_id : 0,
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
        gl::bind_framebuffer(gl::FRAMEBUFFER, self.m_fbo);

        // Generate a texture for our FBO
        let texture_ids = gl::gen_textures(1);
        self.m_fbo_tex_id = texture_ids[0];
        gl::bind_texture(gl::TEXTURE_2D, self.m_fbo_tex_id);

        let width = 256;
        let height = 256;
        // Use linear filtering to scale down and up
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::GLint);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::GLint);

        // Clamp the image to border
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as gl::GLint);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as gl::GLint);
        // 0 for the data pointer means allocate some data for me
        gl::tex_image_2d(gl::TEXTURE_2D,
                         0,
                         gl::RGBA as gl::GLint,
                         width as gl::GLint,
                         height as gl::GLint,
                         0,
                         gl::RGBA,
                         gl::UNSIGNED_BYTE,
                         None);

        // Bind this texture to the FBO
        gl::framebuffer_texture_2d(gl::FRAMEBUFFER,
                                   gl::COLOR_ATTACHMENT0,
                                   gl::TEXTURE_2D,
                                   self.m_fbo_tex_id,
                                   0);

        // Check that its ok
        unsafe {
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Could not bind texture to fbo");
            }
        }

        // Go back to our old fbo
        gl::bind_framebuffer(gl::FRAMEBUFFER, 0);

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
