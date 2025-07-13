use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub const CUBE_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.5, 0.5, 0.5] },
];

pub const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,
    1, 5, 6, 6, 2, 1,
    5, 4, 7, 7, 6, 5,
    4, 0, 3, 3, 7, 4,
    3, 2, 6, 6, 7, 3,
    4, 5, 1, 1, 0, 4,
];