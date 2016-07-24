extern crate glfw;
extern crate gl;

use self::gl::types::*;

// use glfw;
// use gl;

use std::ffi::CString;
use std::ptr;
use std::os::raw;

use self::glfw::Context;

use std::sync::mpsc::Receiver;

use super::math::*;
use super::world::World;

pub struct WorldRenderer {
  pub world: World,
  glfw: glfw::Glfw,
  window: glfw::Window,
  // events: Receiver<(f32, glfw::WindowEvent)>,
}

macro_rules! c_str {
    ($s:expr) => { {
        concat!($s, "\0").as_ptr() as *const i8
    } }
}

#[derive(Clone, Copy, Default)]
struct color {
  r: GLubyte,
  g: GLubyte,
  b: GLubyte,
  a: GLubyte,
}

impl color {
  fn from_bytes(bytes: &[u8]) -> color {
    color {
      r: bytes[0],
      g: bytes[1],
      b: bytes[2],
      a: bytes[3],
    }
  }
}

macro_rules! color {
    ($s:expr) => { {
      color::from_bytes($s)
    } }
}

#[derive(Clone, Copy, Default)]
struct vertex {
  x: GLfloat,
  y: GLfloat,
  color: color,
}

#[derive(Clone, Copy, Default)]
struct particle {
  vertices: [vertex; 6],
}

impl BB {
  fn into_particle(self, color: color) -> particle {
    particle {
      vertices: [
        vertex {x: self.l as f32, y: self.t as f32, color: color},
        vertex {x: self.r as f32, y: self.t as f32, color: color},
        vertex {x: self.r as f32, y: self.b as f32, color: color},
        vertex {x: self.l as f32, y: self.t as f32, color: color},
        vertex {x: self.l as f32, y: self.b as f32, color: color},
        vertex {x: self.r as f32, y: self.b as f32, color: color},
      ],
    }
  }
}

impl WorldRenderer {
  pub fn new(world: World, step: &mut FnMut(&mut World)) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(640, 640, "Hello this is window", glfw::WindowMode::Windowed)
      .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    let mut renderer = WorldRenderer {
      world: world,
      glfw: glfw,
      window: window,
      // events: events,
    };

    // the supplied function must be of the type:
    // `&fn(symbol: &str) -> Option<extern "C" fn()>`
    // `window` is a glfw::Window
    gl::load_with(|s| unsafe {
      renderer.window.get_proc_address(s) as *const _
    });

    // loading a specific function pointer
    gl::Viewport::load_with(|s| renderer.window.get_proc_address(s) as *const _);

    let shader_fragment_text = "varying vec4 v_color;\nvoid main() {gl_FragColor = v_color;}";
    let shader_vertex_text = "uniform mat4 modelview_projection;\nattribute vec2 a_position;\nattribute vec4 a_color;\nvarying vec4 v_color;\nvoid main() {\n  v_color = a_color;\n  gl_Position = modelview_projection * vec4(a_position, 0, 1);\n}\n";

    let shader_program = unsafe { renderer.compile_program(shader_fragment_text, shader_vertex_text) };
    unsafe {
      gl::UseProgram(shader_program);
    }
    let position_attribute = unsafe {
      let position_attribute = gl::GetAttribLocation(shader_program, c_str!("a_position")) as u32;
      gl::EnableVertexAttribArray(position_attribute);
      position_attribute
    };
    let color_attribute = unsafe {
      let color_attribute = gl::GetAttribLocation(shader_program, c_str!("a_color")) as u32;
      gl::EnableVertexAttribArray(color_attribute);
      color_attribute
    };
    let modelview_projection = unsafe {
      let modelview_projection = gl::GetUniformLocation(shader_program, c_str!("modelview_projection"));
      // matrixtAttribute = modelview_projection;
      let identity = [
        1f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 1f32
      ];
      gl::UniformMatrix4fv(modelview_projection, 1, gl::TRUE, &identity as *const f32);
      modelview_projection
    };

    let buffer = unsafe {
      let mut buffer = [0 as u32; 1];
      gl::GenBuffers(1, buffer.as_mut_ptr() as *mut _);
      buffer
    };

    while !renderer.window.should_close() {
      renderer.glfw.poll_events();
      for (_, event) in glfw::flush_messages(&events) {
        renderer.handle_window_event(event);
      }
      step(&mut renderer.world);
      unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::DST_ALPHA);

        gl::UseProgram(shader_program);

        // GLfloat vertices[2 * 4] = {
        //   160, 120,
        //   -160, 120,
        //   160, -120,
        //   -160, -120
        // };

        // phv size = phv(640, 640);
        // float right = world->aabb.right,
        let (right, left, top, bottom, z_far, z_near) = (640f32, 0f32, 640f32, 0f32, 1000f32, 0f32);
        // let right = size.x,
        // left = 0,
        // // top = world->aabb.top,
        // top = size.y,
        // bottom = 0,
        // zFar = 1000,
        // zNear = 0;
        let tx : f32 = -(right + left) / (right - left);
        let ty : f32 = -(top + bottom) / (top - bottom);
        let tz : f32 = -(z_far + z_near) / (z_far - z_near);

        let mut matrix = [
          1f32, 0f32, 0f32, 0f32,
          0f32, 1f32, 0f32, 0f32,
          0f32, 0f32, 1f32, 0f32,
          0f32, 0f32, 0f32, 1f32
        ];
        matrix[0] = 2f32 / (right - left);
        matrix[5] = 2f32 / (top - bottom);
        matrix[10] = -2f32 / (z_far - z_near);
        matrix[12] = tx;
        matrix[13] = ty;
        matrix[14] = tz;

        gl::UniformMatrix4fv(modelview_projection, 1, gl::FALSE, matrix.as_mut_ptr() as *mut _);

        let mut data : [particle; 65536] = [Default::default(); 65536];
        let mut num_particles : isize = 0;
        // data[0] = bb!(160, 160, 480, 480)
        // .into_particle(color!(b"\x00\x00\xff\xff"));
        let blue = color!(b"\x00\x00\xff\x88");
        let red = color!(b"\xff\x00\x00\x88");
        let yellow = color!(b"\xff\xff\x00\x88");
        let orange = color!(b"\xff\x88\x00\x88");
        for (i, particle) in renderer.world.iter_particles().enumerate() {
          num_particles = (i + 1) as isize;
          let c = if particle.uncontained {
            if i % 100 == 0 {orange} else {red}
          } else {
            if i % 100 == 0 {yellow} else {blue}
          };
          data[i] = particle.bbox.into_particle(c);
        }
        // data[0] = particle {
        //   vertices: [
        //     vertex {x: 160f32, y: 480f32, color: color {r: 0u8, g: 0u8, b: 255u8, a: 255u8}},
        //     vertex {x: 480f32, y: 480f32, color: color {r: 0u8, g: 0u8, b: 255u8, a: 255u8}},
        //     vertex {x: 480f32, y: 160f32, color: color {r: 0u8, g: 0u8, b: 255u8, a: 255u8}},
        //     vertex {x: 160f32, y: 480f32, color: color {r: 0u8, g: 0u8, b: 255u8, a: 255u8}},
        //     vertex {x: 160f32, y: 160f32, color: color {r: 0u8, g: 0u8, b: 255u8, a: 255u8}},
        //     vertex {x: 480f32, y: 160f32, color: color {r: 0u8, g: 0u8, b: 255u8, a: 255u8}},
        //   ],
        // };
        // struct gldata data;
        // data.index = 0;
        // reset_water_mesh(&data);
        // phworldparticleiterator _wpitr;
        // phWorldParticleIterator(world, &_wpitr);
        // while (phWorldParticleNext(&_wpitr)) {
        //   // set_particle_vertices(&data, phWorldParticleDeref(&_wpitr));
        //   toggle_water_mesh_cells(&data, phWorldParticleDeref(&_wpitr));
        // }

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer[0]);
        let mut data_ptr : *const raw::c_void = ptr::null();
        data_ptr = data.as_ptr() as *const raw::c_void;
        gl::BufferData(
          gl::ARRAY_BUFFER,
          72 * num_particles,
          data_ptr,
          gl::DYNAMIC_DRAW
        );

        let position_offset : *const raw::c_void = ptr::null();
        gl::VertexAttribPointer(position_attribute, 2, gl::FLOAT, gl::FALSE, 12, position_offset);
        let color_offset : *const raw::c_void = ptr::null();
        // ptr::write(&mut color_offset, 8);
        gl::VertexAttribPointer(color_attribute, 4, gl::UNSIGNED_BYTE, gl::TRUE, 12, color_offset.offset(8));

        gl::DrawArrays(gl::TRIANGLES, 0, (6 * num_particles) as i32 );
      }
      renderer.window.swap_buffers();
    }
  }

  unsafe fn compile_program(&mut self, frag_source: &'static str, vert_source: &'static str) -> gl::types::GLuint {
    let shader_program = gl::CreateProgram();
    let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
    self.compile_shader(fragment, frag_source);
    gl::AttachShader(shader_program, fragment);

    let vertex = gl::CreateShader(gl::VERTEX_SHADER);
    self.compile_shader(vertex, vert_source);
    gl::AttachShader(shader_program, vertex);

    gl::LinkProgram(shader_program);
    let mut programStatus : gl::types::GLint = 0;
    gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut programStatus as *mut _);
    if programStatus as gl::types::GLboolean == gl::FALSE {
      println!("program failed compilation");
    }

    shader_program
  }

  unsafe fn compile_shader(&mut self, shader: gl::types::GLuint, source: &'static str) {
    let source_cstr = CString::new(source).unwrap();
    // let source_bytes = source_cstr.as_bytes_with_nul();
    let source_ptr = &source_cstr.as_ptr() as *const *const i8;
    let source_len : i32 = source.len() as i32;
    gl::ShaderSource(shader, 1, source_ptr, &source_len as *const _);
    gl::CompileShader(shader);

    let mut compile_status : gl::types::GLint = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status as *mut _);

    if compile_status as gl::types::GLboolean == gl::FALSE {
      let mut info_log_length : gl::types::GLint = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_log_length as *mut i32);
      println!("shader: failed compilation");
      let mut info_log_buffer = Vec::<u8>::with_capacity(info_log_length as usize + 1);
      for i in 0..(info_log_length as usize + 1) {
        info_log_buffer.push(0);
      }
      if let Ok(info_log) = CString::new(info_log_buffer) {
        gl::GetShaderInfoLog(shader, info_log_length, &mut 0 as *mut _, info_log.as_ptr() as *mut _);
        if let Ok(info_log_str) = info_log.into_string() {
          println!("shader(info): {}\n", info_log_str);
        }
      }
    }
  }

  fn handle_window_event(&mut self, event: glfw::WindowEvent) {
    match event {
      glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
        self.window.set_should_close(true)
      }
      _ => {}
    }
  }
}
