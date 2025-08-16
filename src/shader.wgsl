// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

// Instance input data
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

//this struct will hold the output of the vertext shader
//basically the data that will be passed from the vertex shader to fragment shader
//the @ instructions provide special info to the gpu about how to handle to data
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //@builtin(position) tells the gpu this is supposed to be the final vertex position
    @location(0) tex_coords: vec2<f32>, // texture coordinates
    @location(1) normal: vec3<f32>, // normal for lighting
};

//marks it as an entry point for a vertex shader
@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput 
{
    // Reassemble the model matrix from the instance data
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.normal = model.normal;
    // Apply the model matrix before the camera view projection
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}