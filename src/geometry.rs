use bytemuck;
use wgpu;

#[repr(C)] //layout the struct in memory how a C compiler would ->
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3], //syntax for making arrays of type f32 with a compile length of 3
    color: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, //array_stride says how big the vertex is. When the shader goes to read the next vertex, it will skip over the array_stride number of bytes
            step_mode: wgpu::VertexStepMode::Vertex, //here for stepmode it defines that each element in array represents pre-vertex data and not pre-instance 
            //these attributes usually correspond to the member variables in the Vertex struct
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0, //for the first attribute, offset is zero, and later attributes will have sum of size_of of the previous' data
                    shader_location: 0, //saying that this attribute corresponds to location of 0 in shaders -> telling the gpu how position and color are mapped in shaders
                    format: wgpu::VertexFormat::Float32x3, // 6.tells the shader the shape of the attribute. Float32x3 corresponds to vec3<f32> in shader code. 
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    }
}

//data that make up the triangle
//vertex data laid out in ccw order bc earlier we talked about having the front_face to be ccw -> with this data we have the triangle facing us
pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];

//using index buffers to save on memory -> save us having to keep track of duplicate data. You only need to map the order that the vertices appear in. 
pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
]; 