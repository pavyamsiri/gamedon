use core::mem;

pub(crate) const VERTICES: &[Vertex] = &[
    // Top left
    Vertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
    },
    // Bottom left
    Vertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0],
    },
    // Bottom right
    Vertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
    // Top right
    Vertex {
        position: [0.5, 0.5, 0.0],
        uv: [1.0, 0.0],
    },
];

pub(crate) const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3, 0];

#[repr(C)]
#[derive(
    Copy,
    Clone,
    Debug,
    zerocopy_derive::IntoBytes,
    zerocopy_derive::FromZeros,
    zerocopy_derive::Immutable,
)]
pub(crate) struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub(crate) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
