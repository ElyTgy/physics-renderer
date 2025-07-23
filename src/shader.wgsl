// Vertex shader

//this struct will hold the output of the vertext shader
//basically the data that will be passed from the vertex shader to fragment shader
//the @ instructions provide special info to the gpu about how to handle to data
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //@builtin(position) tells the gpu this is supposed to be the final vertex position
    @location(0) vert_pos: vec3<f32>, //custom attribute made to pass data from vertex to fragment shader
};

//marks it as an entry point for a vertex shader
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32,) -> VertexOutput 
{
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}