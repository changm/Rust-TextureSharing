extern crate libc;
extern crate gleam;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::mem;
use gleam::gl;

use cgl;
use core_foundation;
use io_surface;

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use cgl::{kCGLNoError, CGLGetCurrentContext, CGLTexImageIOSurface2D, CGLErrorString};
use std::ffi::CStr;

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
    debug_fragment_shader : Option<gl::GLuint>,
    pid : gl::GLuint,
    debug_pid : gl::GLuint,

    m_vao : gl::GLuint,
    // indices
    m_ibo : gl::GLuint,
    m_vbo : gl::GLuint,

    // For a shared surface
    pub m_shared_gl_texture_id : gl::GLuint,
    pub m_shared_iosurface_id : io_surface::IOSurfaceID,
    m_shared_surface : Option<io_surface::IOSurface>,
}

impl Drop for Device {
    fn drop (&mut self) {
        let frame_buffers = [self.m_fbo];
        gl::delete_framebuffers(&frame_buffers);

        gl::delete_shader(self.quad_vertex_shader.unwrap());
        gl::delete_shader(self.quad_fragment_shader.unwrap());
        gl::delete_shader(self.debug_fragment_shader.unwrap());

        gl::delete_program(self.pid);
        gl::delete_program(self.debug_pid);

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
                                  debug_fragment_shader : Some(0),
                                  pid : 0,
                                  debug_pid : 0,
                                  m_vao : 0,
                                  m_ibo : 0,
                                  m_vbo : 0,
                                  m_shared_gl_texture_id: 0,
                                  m_shared_iosurface_id : 0,
                                  m_shared_surface: None};

        let vertex_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/vertex.glsl");
        let fragment_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/fragment.glsl");
        let debug_shader = String::from("/Users/masonchang/Projects/Rust-TextureSharing/shaders/fragment_texture.glsl");

        // Compile our shaders
        device.quad_vertex_shader = compile_shader(&vertex_shader, gl::VERTEX_SHADER);
        device.quad_fragment_shader = compile_shader(&fragment_shader, gl::FRAGMENT_SHADER);
        device.debug_fragment_shader = compile_shader(&debug_shader, gl::FRAGMENT_SHADER);

        // Create our program.
        device.pid = gl::create_program();
        gl::attach_shader(device.pid, device.quad_vertex_shader.unwrap());
        gl::attach_shader(device.pid, device.quad_fragment_shader.unwrap());

        // Use the program
        gl::link_program(device.pid);
        gl::use_program(device.pid);

        // Create our program.
        device.debug_pid = gl::create_program();
        gl::attach_shader(device.debug_pid, device.quad_vertex_shader.unwrap());
        gl::attach_shader(device.debug_pid, device.debug_fragment_shader.unwrap());

        // Use the program
        gl::link_program(device.debug_pid);

        return device;
    }

    pub fn debug_shaders(&self) {
        gl::use_program(self.debug_pid);
    }

    pub fn release_shaders(&self) {
        gl::use_program(self.pid);
    }

    pub fn begin_frame(&self) {
        gl::bind_framebuffer(gl::FRAMEBUFFER, self.m_fbo);
    }

    pub fn end_frame(&self) {
        gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    }

    pub fn setup_iosurface(&mut self) {
        let width = 1024;
        let height = 1024;

        // Create an io surface
        unsafe {
            let width_key: CFString = TCFType::wrap_under_get_rule(io_surface::kIOSurfaceWidth);
            let width_value: CFNumber = CFNumber::from_i32(width);

            let height_key: CFString = TCFType::wrap_under_get_rule(io_surface::kIOSurfaceHeight);
            let height_value: CFNumber = CFNumber::from_i32(height);

            let bytes_per_row_key: CFString =
                TCFType::wrap_under_get_rule(io_surface::kIOSurfaceBytesPerRow);
            let bytes_per_row_value: CFNumber = CFNumber::from_i32(width * 4);

            let bytes_per_elem_key: CFString =
                TCFType::wrap_under_get_rule(io_surface::kIOSurfaceBytesPerElement);
            let bytes_per_elem_value: CFNumber = CFNumber::from_i32(4);

            let is_global_key: CFString =
                TCFType::wrap_under_get_rule(io_surface::kIOSurfaceIsGlobal);
            let is_global_value = CFBoolean::true_value();

            self.m_shared_surface = Some(io_surface::new(&CFDictionary::from_CFType_pairs(&[
                (width_key.as_CFType(), width_value.as_CFType()),
                (height_key.as_CFType(), height_value.as_CFType()),
                (bytes_per_row_key.as_CFType(), bytes_per_row_value.as_CFType()),
                (bytes_per_elem_key.as_CFType(), bytes_per_elem_value.as_CFType()),
                (is_global_key.as_CFType(), is_global_value.as_CFType()),
            ])));
        }

        self.bind_iosurface(width, height);
    }

    pub fn bind_iosurface(&mut self, width: i32, height: i32) {
        // Create our texture
        let texture_ids = gl::gen_textures(1);
        self.m_shared_gl_texture_id = texture_ids[0];

        // According to apple docs, ioshared surfaces only work for GL_TEXTURE_RECTANGLE
        // Which means fragment shader data has to be based on texels and not [-1..1].
        gl::bind_texture(gl::TEXTURE_RECTANGLE, self.m_shared_gl_texture_id);
        match self.m_shared_surface {
            Some(ref surface) => {
                surface.bind_to_gl_texture(width, height);
                self.m_shared_iosurface_id = surface.get_id();
            },
            None => { panic!("No surface created") },
        }

        // Use linear filtering to scale down and up
        gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::GLint);
        gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::GLint);

        // Clamp the image to border
        gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as gl::GLint);
        gl::tex_parameter_i(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as gl::GLint);
        gl::bind_texture(gl::TEXTURE_RECTANGLE, 0);
    }

    pub fn connect_iosurface(&mut self, iosurface_id : u8) {
        let width = 1024;
        let height = 1024;

        self.m_shared_iosurface_id = iosurface_id as u32;
        self.m_shared_surface = Some(io_surface::lookup(self.m_shared_iosurface_id));
        println!("Child got surface: {:?}", self.m_shared_surface);
        self.bind_iosurface(width, height);
    }

    pub fn setup_fbo_iosurface(&mut self) {
        // Now use this ios urface for an FBO
        // generate our FBO
        let fbos = gl::gen_framebuffers(1);
        self.m_fbo = fbos[0];
        gl::bind_framebuffer(gl::FRAMEBUFFER, self.m_fbo);

        // Bind this texture to the FBO
        gl::framebuffer_texture_2d(gl::FRAMEBUFFER,
                                   gl::COLOR_ATTACHMENT0,
                                   gl::TEXTURE_RECTANGLE,
                                   self.m_shared_gl_texture_id,
                                   0);

        // Check that its ok
        unsafe {
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Could not bind texture to fbo");
            }
        }

        // Make sure we clear everything
        gl::clear(gl::COLOR_BUFFER_BIT);
        gl::flush();

        // Go back to our old fbo
        gl::bind_framebuffer(gl::FRAMEBUFFER, 0);
    }

    pub fn setup_noninverting_vertices(&mut self) {
        let vertices: [f32; 16] =
        [
            // vertices     // Texture coordinates, origin is bottom left, but images decode top left origin
            // So we flip our texture coordinates here instead.
            -1.0, -1.0,     0.0, 1.0,  // Bottom left
            -1.0, 1.0,      0.0, 0.0, // Top Left
            1.0, 1.0,       1.0, 0.0,    // Top right
            1.0, -1.0,      1.0, 1.0,  // bottom right
        ];

/*
        let vertices: [f32; 16] =
        [
            // vertices     // Texture coordinates, origin is bottom left
            -1.0, -1.0,     0.0, 0.0,  // Bottom left
            -1.0, 1.0,      0.0, 1.0, // Top Left
            1.0, 1.0,       1.0, 1.0,    // Top right
            1.0, -1.0,      1.0, 0.0,  // bottom right
        ];
        */

        gl::bind_buffer(gl::ARRAY_BUFFER, self.m_vbo);
        gl::buffer_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
    }

    pub fn setup_shared_texture_vertices(&self) {
        let vertices: [f32; 16] =
        [
            // vertices
            // So we flip our texture coordinates here instead.
            -1.0, -1.0,     0.0, 0.0,  // Bottom left
            -1.0, 1.0,      0.0, 1024.0, // Top Left
            1.0, 1.0,       1024.0, 1024.0,    // Top right
            1.0, -1.0,      1024.0, 0.0,  // bottom right
        ];
        gl::bind_buffer(gl::ARRAY_BUFFER, self.m_vbo);
        gl::buffer_data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
    }

    pub fn setup_vao(&mut self) {
        // These coordinates are texture coordinates in the size of the image.
        let vertices: [f32; 16] =
        [
            // vertices     // Texture coordinates, origin is bottom left, but images decode top left origin
            // So we flip our texture coordinates here instead.
            -1.0, -1.0,     0.0, 256.0,  // Bottom left
            -1.0, 1.0,      0.0, 0.0, // Top Left
            1.0, 1.0,       256.0, 0.0,    // Top right
            1.0, -1.0,      256.0, 256.0,  // bottom right
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
