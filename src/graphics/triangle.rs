use anyhow::Ok;
use render_gl_derive::VertexAttribPointers;

use crate::{
    render_gl::{
        self,
        buffer::{self, ArrayBuffer, VertexArray},
        data,
    },
    resources::Resources,
};

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 1]
    clr: data::u2_u10_u10_u10_rev_float,
}

pub struct Triangle {
    program: render_gl::Program,
    _vbo: buffer::ArrayBuffer, // _ to disable warning about not used vbo
    vao: buffer::VertexArray,
}

impl Triangle {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Triangle, anyhow::Error> {
        let program = render_gl::Program::from_res(gl, res, "shaders/triangle")?;

        let vertices: Vec<Vertex> = vec![
            Vertex { pos: (0.5, -0.5, 0.0).into(), clr: (1.0, 0.0, 0.0, 1.0).into() }, // bottom right
            Vertex { pos: (-0.5, -0.5, 0.0).into(), clr: (0.0, 1.0, 0.0, 1.0).into() }, // bottom left
            Vertex { pos: (0.0, 0.5, 0.0).into(), clr: (0.0, 0.0, 1.0, 1.0).into() },   // top
        ];

        let vbo = ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();

        let vao = VertexArray::new(gl);
        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(&gl);
        vbo.unbind();
        vao.unbind();

        Ok(Triangle { program, _vbo: vbo, vao })
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.program.set_used();
        self.vao.bind();
        unsafe {
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0,             // starting index in the enabled arrays
                3,             // number of indices to be rendered
            );
        }
    }
}
