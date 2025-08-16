use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    event::*, event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window
};
use cgmath::prelude::*;

use crate::camera::{Camera, CameraController, CameraUniform};
use crate::texture::Texture;
use crate::model::{Model, ModelVertex, DrawModel, Vertex as ModelVertexTrait};
use crate::resources;
use crate::physics::PhysicsWorld;
use rapier3d::prelude::RigidBodyHandle;


// Instance struct to hold position and rotation data
struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

// Raw instance data that goes into the GPU buffer
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// Constants for instancing
const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN: f32 = 5.0;

// This will store the state of our game
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    obj_model: Model,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,
    depth_texture: Texture,
    pub window: Arc<Window>,
    physics_world: PhysicsWorld,
    physics_bodies: Vec<RigidBodyHandle>, // Store handles to physics bodies
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let camera_controller = CameraController::new(0.01); // Much smaller speed

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Load texture
        let diffuse_bytes = include_bytes!("../assets/texture.jpg");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "texture.jpg").unwrap();

        // Create depth texture
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        });

        //TODO: change this so that the camera's initial target is towards the center of all instances (i.e. get the largest magnitude of x,y,z which would make an imaginery cube, and then set the camera to look at the center of that BUT ignore the z that comes out of this, and set the z an appropriate height above the ground)
        // Initialize camera to look at the first physics instance 
        let mut camera = Camera::new();
        // Point camera towards the first physics instance at (-4, 35, -4)
        let target_position = cgmath::Point3::new(-4.0, 35.0, -4.0);
        camera.set_target(target_position);
        // Position camera slightly offset from the target
        camera.set_eye(cgmath::Point3::new(-6.0, 37.0, -6.0));

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        // Create texture bind group layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("texture_bind_group_layout"),
        });

        // Create texture bind group
        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    ModelVertex::desc(),
                    InstanceRaw::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            //this field describes how to interpret the vertices when converting them to triangles
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1. every three vertices will become a triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2. tells when a triangle is facing forward: orientation of the vertices are counter clockwise
                cull_mode: None, // Disable face culling so all faces are visible
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4. for anti aliasing
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        // Load the cube model
        let mut obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();
        
        // Update all materials to use our loaded texture
        for material in &mut obj_model.materials {
            material.diffuse_texture = Some(diffuse_texture.clone());
            material.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("material_diffuse_bind_group"),
            });
        }

        // Create instances based on physics bodies (initially empty)
        let instances = Vec::new();

        // Create instance buffer (initially empty)
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice::<InstanceRaw, u8>(&[]), // Empty initially
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        // Initialize physics world
        let mut physics_world = PhysicsWorld::new();
        
        // Add ground plane
        physics_world.add_ground();
        
        // GUI: Add some physics cubes -> replace with gui functionality later to user can add these and create seperate file and functions for handling addition of objects via the gui
        //GUI: modify this and have it as a button to add cubes, and under another panel that has a list of all the pbject, drop down for each cube and be able to modify its x,y,z and its rotations
        let mut physics_bodies = Vec::new();
        for z in 0..2 {
            for x in 0..2 {
                let position = cgmath::Vector3::new(
                    x as f32 * 2.0 - 4.0,
                    35.0, // Start above ground
                    z as f32 * 2.0 - 4.0
                );
                let handle = physics_world.add_cube(position, 1.0);
                physics_bodies.push(handle);
            }
        }

        // Configure the surface initially
        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: true,
            render_pipeline,
            instances,
            instance_buffer,
            obj_model,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            diffuse_bind_group,
            diffuse_texture,
            depth_texture,
            window,
            physics_world,
            physics_bodies,
        })
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::KeyR, true) => {
                // Reset camera when R is pressed
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&"RESETTING CAMERA".into());
                self.reset_camera();
            },
            //GUI: also move this to gui, and have it under the button "apply upward force"
            (KeyCode::Space, true) => {
                // Apply force to all bodies
                for handle in &self.physics_bodies {
                    self.physics_world.apply_force(*handle, cgmath::Vector3::new(0.0, 10.0, 0.0));
                }
            },
            _ => {}
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }



    pub fn resize(&mut self, width: u32, height: u32) {
        let max_dim = 800;
        let width = width.min(max_dim);
        let height = height.min(max_dim);

        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.camera.update_aspect(width, height);
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            
            // Recreate depth texture with new dimensions
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            

        }
    }
    
    pub fn update(&mut self) {
        // Step physics simulation (assuming 60 FPS = 1/60 seconds)
        let delta_time = 1.0 / 60.0;
        self.physics_world.step(delta_time);
        
        // Update instances based on physics bodies
        self.update_instances_from_physics();
        
        // Update camera
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }   
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }
        
        //asks surface to give a new surfacetexture that we render to
        let output = self.surface.get_current_texture()?;
        
        //honestly not sure wtf this is but you should apparently get a textureview to control how the renderer interacts w texture
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        //create a command buffer to send data to the GPU
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { 
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            //for working with the shaders and the pipeline
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_model_instanced(&self.obj_model, 0..self.instances.len() as u32, &self.camera_bind_group);
        }

        //encoder.finish() ends the CommandEncoder and returns a CommandBuffer, ready to be passed on to the GPU
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


    // Add this method to State
    fn reset_camera(&mut self) {
        self.camera.reset();
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn update_instances_from_physics(&mut self) {
        let bodies = self.physics_world.get_bodies();
        
        // Clear existing instances and create new ones from physics bodies
        self.instances.clear();
        
        for (_handle, body_data) in bodies {
            // Only add dynamic bodies to rendering (skip ground plane)
            if body_data.is_dynamic {
                self.instances.push(Instance {
                    position: body_data.position,
                    rotation: body_data.rotation,
                });
            }
        }
        
        // Update GPU buffer with new instance data
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        
        // Recreate buffer if size changed
        if instance_data.len() * std::mem::size_of::<InstanceRaw>() != self.instance_buffer.size() as usize {
            self.instance_buffer = self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }
            );
        } else {
            self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
        }
    }




} 