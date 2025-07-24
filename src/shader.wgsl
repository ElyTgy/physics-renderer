// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

//this struct will hold the output of the vertext shader
//basically the data that will be passed from the vertex shader to fragment shader
//the @ instructions provide special info to the gpu about how to handle to data
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //@builtin(position) tells the gpu this is supposed to be the final vertex position
    @location(0) color: vec3<f32>, //custom attribute made to pass data from vertex to fragment shader
};

//marks it as an entry point for a vertex shader
@vertex
fn vs_main(model: VertexInput) -> VertexOutput 
{
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}