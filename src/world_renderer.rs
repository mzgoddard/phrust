extern crate gl;

use self::gl::types::*;

// use glfw;
// use gl;

use std::ffi::CString;
use std::ptr;
use std::os::raw;
use std::mem;

use super::math::*;
use super::world::World;

pub struct WorldRenderer {
  // data: Vec<particle>,
  render_data: RenderData,
  driver: Box<Driver>,
  // pub world: World,
  // glfw: glfw::Glfw,
  // window: glfw::Window,
  // events: Receiver<(f32, glfw::WindowEvent)>,
}

macro_rules! c_str {
    ($s:expr) => { {
      ($s).as_ptr() as *const i8
    } }
}

#[derive(Clone, Copy, Default)]
struct GLColor {
  r: GLubyte,
  g: GLubyte,
  b: GLubyte,
  a: GLubyte,
}

impl GLColor {
  fn from_bytes(bytes: &[u8]) -> GLColor {
    GLColor {
      r: bytes[0],
      g: bytes[1],
      b: bytes[2],
      a: bytes[3],
    }
  }
}

macro_rules! color {
    ($s:expr) => { {
      GLColor::from_bytes($s)
    } }
}

#[derive(Clone, Copy, Default)]
struct GLVertex {
  x: GLfloat,
  y: GLfloat,
  color: GLColor,
}

#[derive(Clone, Copy, Default)]
struct GLParticle {
  vertices: [GLVertex; 6],
}

#[repr(usize)]
enum RenderPropertyType {
  Matrix,
}

impl RenderPropertyType {
  fn as_usize(&self) -> usize {
    unsafe { *mem::transmute::<&RenderPropertyType, &usize>(self) }
  }
}

struct RenderProperty {
  property_type: RenderPropertyType,
  values: Vec<u8>,
}

impl RenderProperty {
  fn new<T>(property_type: RenderPropertyType, values: T) -> RenderProperty {
    let mut buffer = Vec::<u8>::with_capacity(mem::size_of::<T>());
    unsafe {
      buffer.set_len(mem::size_of::<T>());
      *(mem::transmute::<*mut u8, *mut T>(buffer.as_mut_ptr())) = values;
    }
    RenderProperty {
      property_type: property_type,
      values: buffer,
    }
  }
}

#[repr(usize)]
enum MeshPropertyType {
  Vertex,
  Color,
}

impl MeshPropertyType {
  fn as_usize(&self) -> usize {
    unsafe { *mem::transmute::<&MeshPropertyType, &usize>(self) }
  }
}

struct MeshProperty {
  property_type: MeshPropertyType,
  values: Vec<u8>,
}

impl MeshProperty {
  fn new<T>(property_type: MeshPropertyType, values: T) -> MeshProperty {
    let mut buffer = Vec::<u8>::with_capacity(mem::size_of::<T>());
    unsafe {
      buffer.set_len(mem::size_of::<T>());
      *(mem::transmute::<*mut u8, *mut T>(buffer.as_mut_ptr())) = values;
    }
    MeshProperty {
      property_type: property_type,
      values: buffer,
    }
  }
}

#[repr(usize)]
enum RenderProgram {
  VertexColor,
}

impl RenderProgram {
  fn as_usize(&self) -> usize {
    unsafe { *mem::transmute::<&RenderProgram, &usize>(self) }
  }
}

struct RenderDraw {
  program: RenderProgram,
  properties: Vec<RenderProperty>,
  mesh_properties: Vec<MeshProperty>,
  mesh_triangles: usize,
  mesh: Vec<u8>,
}

struct RenderPass {
  draws: Vec<RenderDraw>,
}

struct RenderData {
  passes: Vec<RenderPass>,
}

trait Driver {
  fn draw(&mut self, &RenderData);
}

struct GLUniformMatrixData {
  count: GLint,
  transpose: GLboolean,
  value: [f32; 16],
}

// impl GLUniformMatrixData {
//   fn new(count: GLint, transpose: GLboolean, value: [f32; 16]) -> GLUniformMatrixData {
//     GLUniformMatrixData {
//       count: count,
//       transpose: transpose,
//       value: value,
//     }
//   }
// }

struct GLAttribData {
  size: GLint,
  data_type: GLuint,
  normalized: GLboolean,
  stride: GLint,
  pointer: GLint,
}

// impl GLAttribData {
//   fn new(size: GLint, data_type: GLuint, normalized: GLboolean, stride: GLint, pointer: GLint) -> GLAttribData {
//     GLAttribData {
//       size: size,
//       data_type: data_type,
//       normalized: normalized,
//       stride: stride,
//       pointer: pointer,
//     }
//   }
// }

const GL_RENDER_PROPERTY_NAMES : [&'static str; 1] = [
  "modelview_projection\0",
];

const GL_MESH_PROPERTY_NAMES : [&'static str; 2] = [
  "a_position\0",
  "a_color\0",
];

struct GLProgram {
  id: u32,
  render_properties: Vec<i32>,
  mesh_properties: Vec<i32>,
}

impl GLProgram {
  fn new(frag_source: &'static str, vert_source: &'static str) -> GLProgram {
    unsafe {
      let id = GLProgram::compile_program(frag_source, vert_source);
      let render_properties = GLProgram::load_render_properties(id);
      let mesh_properties = GLProgram::load_mesh_properties(id);
      GLProgram {
        id: id,
        render_properties: render_properties,
        mesh_properties: mesh_properties,
      }
    }
  }

  unsafe fn compile_program(frag_source: &'static str, vert_source: &'static str) -> GLuint {
    let shader_program = gl::CreateProgram();

    let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
    GLProgram::compile_shader(fragment, frag_source);
    gl::AttachShader(shader_program, fragment);

    let vertex = gl::CreateShader(gl::VERTEX_SHADER);
    GLProgram::compile_shader(vertex, vert_source);
    gl::AttachShader(shader_program, vertex);

    gl::LinkProgram(shader_program);
    let mut program_status : gl::types::GLint = 0;
    gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut program_status as *mut _);
    if program_status as gl::types::GLboolean == gl::FALSE {
      let mut len = 0;
      gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut len);
      let mut log_buffer = Vec::<u8>::with_capacity(len as usize);
      log_buffer.set_len(len as usize);
      let mut buffer_used = 0;
      gl::GetProgramInfoLog(shader_program, len, &mut buffer_used, log_buffer.as_mut_ptr() as *mut _);
      panic!("program failed compilation: {}", String::from_utf8(log_buffer).unwrap().as_str());
    }

    shader_program
  }

  unsafe fn compile_shader(shader: GLuint, source: &'static str) {
    let source_cstr = CString::new(source).unwrap();
    // let source_bytes = source_cstr.as_bytes_with_nul();
    let source_ptr = &source_cstr.as_ptr() as *const *const i8;
    let source_len : i32 = source.len() as i32;
    gl::ShaderSource(shader, 1, source_ptr, &source_len as *const _);
    gl::CompileShader(shader);

    let mut compile_status : gl::types::GLint = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);

    if compile_status as gl::types::GLboolean == gl::FALSE {
      let mut info_log_length : gl::types::GLint = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_log_length);
      if info_log_length == 0 {
        info_log_length = 256;
      }
      println!("shader: failed compilation");
      let mut info_log_buffer = Vec::<u8>::with_capacity(info_log_length as usize + 1);
      // info_log_length.set_len(info_log_length);
      for _ in 0..(info_log_length as usize + 1) {
        info_log_buffer.push(0);
      }
      let mut returned_len = 0;
      gl::GetShaderInfoLog(shader, info_log_length, &mut returned_len, info_log_buffer.as_mut_ptr() as *mut _);
      println!("shader(info): {} {}\n", returned_len, String::from_utf8(info_log_buffer).unwrap().as_str());
      // if let Ok(info_log_str) = info_log.into_string() {
      //   panic!("shader(info): {}\n", &info_log_str as &str);
      // }
      // match CString::new(info_log_buffer) {
      //    Ok(info_log) => {
      //     gl::GetShaderInfoLog(shader, info_log_length, &mut 0 as *mut _, info_log.as_ptr() as *mut _);
      //     if let Ok(info_log_str) = info_log.into_string() {
      //       panic!("shader(info): {}\n", info_log_str);
      //     }
      //   },
      //   _ => {
      //     panic!("failed to get shader compilation failure");
      //   },
      // }
    }
  }

  fn load_render_properties(shader_program: GLuint) -> Vec<i32> {
    let mut properties = Vec::<i32>::new();
    for name in GL_RENDER_PROPERTY_NAMES.iter() {
      properties.push(unsafe {
        gl::GetUniformLocation(shader_program, c_str!(name))
      });
    }
    properties
  }

  fn load_mesh_properties(shader_program: GLuint) -> Vec<i32> {
    let mut properties = Vec::<i32>::new();
    for name in GL_MESH_PROPERTY_NAMES.iter() {
      properties.push(unsafe {
        gl::GetAttribLocation(shader_program, c_str!(name))
      });
    }
    properties
  }

  fn use_program(&mut self) {
    unsafe { gl::UseProgram(self.id); }
  }

  fn set_properties(&mut self, properties: &[RenderProperty]) {
    for property in properties.iter() {
      match property.property_type {
        RenderPropertyType::Matrix => {
          unsafe { 
            let gl_uniform_data = &*mem::transmute::<*const u8, *const GLUniformMatrixData>(property.values.as_ptr());
            gl::UniformMatrix4fv(
              self.render_properties[property.property_type.as_usize()],
              gl_uniform_data.count,
              gl_uniform_data.transpose,
              gl_uniform_data.value.as_ptr(),
            );
          }
        },
      }
    }
  }

  fn set_mesh_properties(&mut self, properties: &[MeshProperty]) {
    for property in properties.iter() {
      unsafe {
        let gl_attrib_data = &*mem::transmute::<*const u8, *const GLAttribData>(property.values.as_ptr());
        gl::EnableVertexAttribArray(self.mesh_properties[property.property_type.as_usize()] as u32);
        gl::VertexAttribPointer(
          self.mesh_properties[property.property_type.as_usize()] as u32,
          gl_attrib_data.size,
          gl_attrib_data.data_type,
          gl_attrib_data.normalized,
          gl_attrib_data.stride,
          ptr::null().offset(gl_attrib_data.pointer as isize),
        );
      }
    }
  }
}

#[cfg(target_os="ios")]
const FRAGMENT_TEXT : &'static str = "varying lowp vec4 v_color;\nvoid main() {gl_FragColor = v_color;}";

#[cfg(not(target_os="ios"))]
const FRAGMENT_TEXT : &'static str = "varying vec4 v_color;\nvoid main() {gl_FragColor = v_color;}";

const SHADER_TEXT : [(&'static str, &'static str); 1] = [
  (
    FRAGMENT_TEXT,
    "uniform mat4 modelview_projection;\nattribute vec2 a_position;\nattribute vec4 a_color;\nvarying vec4 v_color;\nvoid main() {\n  v_color = a_color;\n  gl_Position = modelview_projection * vec4(a_position, 0, 1);\n}\n"
  ),
];

pub struct GLDriver {
  programs: Vec<GLProgram>,
  buffers: Vec<u32>,
}

impl GLDriver {
  pub fn init<F>(mut get_proc_address: F) where F: FnMut(&str) -> *const raw::c_void {
    // the supplied function must be of the type:
    // `&fn(symbol: &str) -> Option<extern "C" fn()>`
    // `window` is a glfw::Window
    gl::load_with(|s| get_proc_address(s) as *const _);

    // loading a specific function pointer
    gl::Viewport::load_with(|s| get_proc_address(s) as *const _);
  }

  pub fn new() -> GLDriver {
    GLDriver {
      programs: GLDriver::create_programs(),
      buffers: GLDriver::create_buffers(),
    }
  }

  fn create_programs() -> Vec<GLProgram> {
    SHADER_TEXT.iter()
    .map(|&(frag_source, vert_source)| GLProgram::new(frag_source, vert_source))
    .collect()
  }

  fn create_buffers() -> Vec<u32> {
    let mut buffers = Vec::<u32>::with_capacity(1);
    unsafe { 
      buffers.set_len(1);
      gl::GenBuffers(1, buffers.as_mut_ptr() as *mut _);
    }
    buffers
  }

  fn buffer_data(&mut self, buffer_index: usize, mesh: &[u8]) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.buffers[buffer_index]);
      {
        gl::BufferData(
          gl::ARRAY_BUFFER,
          mesh.len() as isize,
          mesh.as_ptr() as *const raw::c_void,
          gl::DYNAMIC_DRAW
        );
      }
    }
  }

  fn inner_draw(&mut self, draw: &RenderDraw) {
    unsafe {
      gl::DrawArrays(gl::TRIANGLES, 0, draw.mesh_triangles as i32);
    }
  }
}

impl Driver for GLDriver {
  fn draw(&mut self, data: &RenderData) {
    unsafe {
      gl::ClearColor(0.0, 0.0, 0.0, 0.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::DST_ALPHA);
    }

    for pass in data.passes.iter() {
      for draw in pass.draws.iter() {
        self.buffer_data(0, &draw.mesh);
        {
          let mut program = &mut self.programs[draw.program.as_usize()];
          program.use_program();
          program.set_properties(&draw.properties);
          program.set_mesh_properties(&draw.mesh_properties);
        }
        self.inner_draw(draw);
      }
    }
  }
}

struct HeadlessDriver {}

impl HeadlessDriver {
  fn new() -> HeadlessDriver {
    HeadlessDriver {}
  }
}

impl Driver for HeadlessDriver {
  fn draw(&mut self, _: &RenderData) {}
}

impl BB {
  fn into_particle(self, color: GLColor) -> GLParticle {
    GLParticle {
      vertices: [
        GLVertex {x: self.l as f32, y: self.t as f32, color: color},
        GLVertex {x: self.r as f32, y: self.t as f32, color: color},
        GLVertex {x: self.r as f32, y: self.b as f32, color: color},
        GLVertex {x: self.l as f32, y: self.t as f32, color: color},
        GLVertex {x: self.l as f32, y: self.b as f32, color: color},
        GLVertex {x: self.r as f32, y: self.b as f32, color: color},
      ],
    }
  }
}

impl WorldRenderer {
  fn render_data() -> RenderData {
    RenderData {
      passes: vec!(
        RenderPass {
          draws: vec!(RenderDraw {
            program: RenderProgram::VertexColor,
            properties: Vec::<RenderProperty>::new(),
            mesh_properties: Vec::<MeshProperty>::new(),
            mesh_triangles: 0,
            mesh: Vec::<u8>::new(),
          }),
        },
      ),
    }
  }

  pub fn new() -> WorldRenderer {
    WorldRenderer {
      render_data: WorldRenderer::render_data(),
      driver: Box::new(GLDriver::new()),
    }
  }

  pub fn new_headless() -> WorldRenderer {
    WorldRenderer {
      render_data: WorldRenderer::render_data(),
      driver: Box::new(HeadlessDriver::new()),
    }
  }

  pub fn update(&mut self, world: &World) {
    unsafe {
      // gl::ClearColor(0.0, 0.0, 0.0, 0.0);
      // gl::Clear(gl::COLOR_BUFFER_BIT);
      // gl::Enable(gl::BLEND);
      // gl::BlendFunc(gl::SRC_ALPHA, gl::DST_ALPHA);

      // gl::UseProgram(self.shader_program);

      // GLfloat vertices[2 * 4] = {
      //   160, 120,
      //   -160, 120,
      //   160, -120,
      //   -160, -120
      // };

      // phv size = phv(640, 640);
      // float right = world->aabb.right,
      let bb = bb!(-640, -640, 640, 640);
      let (right, left, top, bottom, z_far, z_near) = (bb.r, bb.l, bb.t, bb.b, 1000f32, 0f32);
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

      {
        let draw = &mut self.render_data.passes[0].draws[0];

        draw.properties.clear();
        draw.properties.push(RenderProperty::new(
          RenderPropertyType::Matrix,
          GLUniformMatrixData {
            count: 1,
            transpose: gl::FALSE,
            value: matrix,
          },
        ));

        draw.mesh_properties.clear();
        draw.mesh_properties.push(MeshProperty::new(
          MeshPropertyType::Vertex,
          GLAttribData {
            size: 2,
            data_type: gl::FLOAT,
            normalized: gl::FALSE,
            stride: 12,
            pointer: 0,
          },
        ));
        draw.mesh_properties.push(MeshProperty::new(
          MeshPropertyType::Color,
          GLAttribData {
            size: 4,
            data_type: gl::UNSIGNED_BYTE,
            normalized: gl::TRUE,
            stride: 12,
            pointer: 8,
          },
        ));

        let num_particles : isize = world.iter_particles().count() as isize;
        draw.mesh_triangles = (num_particles * 6) as usize;
        {
          let mut data = &mut draw.mesh;
          data.reserve((num_particles * 72) as usize);
          data.set_len((num_particles * 72) as usize);

          let blue = color!(b"\x00\x00\xff\x88");
          let red = color!(b"\xff\x00\x00\x88");
          let yellow = color!(b"\xff\xff\x00\x88");
          let orange = color!(b"\xff\x88\x00\x88");
          let white = color!(b"\xff\xff\xff\x88");
          for (i, particle) in world.iter_particles().enumerate() {
            let c = if particle.is_trigger() {
              white
            }
            else if particle.uncontained {
              // red
              if i % 100 == 0 {orange} else {red}
            }
            else {
              // blue
              if i % 100 == 0 {yellow} else {blue}
            };
            *mem::transmute::<*mut u8, *mut GLParticle>(data.as_mut_ptr().offset((i * 72) as isize)) = particle.bbox.into_particle(c);
          }
        }
      }
    }
  }

  pub fn draw(&mut self) {
    self.driver.draw(&self.render_data);
  }
}
