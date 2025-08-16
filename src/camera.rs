use cgmath::{self, InnerSpace};
use bytemuck;
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

pub struct Camera {
    eye: cgmath::Point3<f32>, //position of camera in space
    target: cgmath::Point3<f32>, //where the camera should look at
    up: cgmath::Vector3<f32>, //upward direction for camera which should be [0,1,0] -> not sure why we need this
    aspect: f32, //aspect ratio of the screen width/height
    fovy: f32, 
    znear: f32, //clips
    zfar: f32,
}

//webgpu space ranges from 0 to 1 whereas opengl is -1 to 1 
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn new() -> Self {
        Self {
            //+1 unit up and 2 units back, with +z being out of the screen meaning towards me
            eye: (0.0, 1.0, 2.0).into(),
            //look at the origin
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: 1.0, //default to 1 -> if its NaN its the object wont render because the projection matrix wont render 
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!("Building matrix with eye: {:?}, target: {:?}, up: {:?}", 
            self.eye, self.target, self.up).into());

        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up); //Create a homogeneous transformation matrix that will cause a vector to point at target from eye, using up for orientation. rh means right handed coordinate system
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!("View matrix: {:?}", view).into());

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar); //have the screen setup with proper aspect ratio and depth without warping
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!("Projection matrix: {:?}", proj).into());

        let result = OPENGL_TO_WGPU_MATRIX * proj * view;
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!("Final matrix: {:?}", result).into());
        result
    }

    pub fn reset(&mut self) {
        #[cfg(target_arch = "wasm32")]
        console::log_1(&"reset being called".into());
        self.eye = (0.0, 1.0, 2.0).into();
        self.target = (0.0, 0.0, 0.0).into();
        self.up = cgmath::Vector3::unit_y();
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        if height > 0 {
            self.aspect = width as f32 / height as f32;
            #[cfg(target_arch = "wasm32")]
            console::log_1(&format!("Aspect ratio updated: {} / {} = {}", width, height, self.aspect).into());
        } else {
            #[cfg(target_arch = "wasm32")]
            console::log_1(&"Warning: Height is 0, keeping current aspect ratio".into());
        }
    }

    pub fn get_eye(&self) -> cgmath::Point3<f32> {
        self.eye
    }

    pub fn get_target(&self) -> cgmath::Point3<f32> {
        self.target
    }

    pub fn get_up(&self) -> cgmath::Vector3<f32> {
        self.up
    }

    pub fn set_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.eye = eye;
    }

    pub fn set_target(&mut self, target: cgmath::Point3<f32>) {
        self.target = target;
    }

    pub fn set_up(&mut self, up: cgmath::Vector3<f32>) {
        self.up = up;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        let matrix = camera.build_view_projection_matrix();
        self.view_proj = matrix.into();
        
        // Debug: Check matrix values
        #[cfg(target_arch = "wasm32")]
        console::log_1(&format!("View-projection matrix: {:?}", matrix).into());
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    // Camera orientation
    yaw: f32,   // Horizontal rotation (left/right)
    pitch: f32, // Vertical rotation (up/down)
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            yaw: -90.0, // Start looking along negative z-axis
            pitch: 0.0,
        }
    }

    pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state,
                        physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == winit::event::ElementState::Pressed;
                match keycode {
                    winit::keyboard::KeyCode::KeyW | winit::keyboard::KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    winit::keyboard::KeyCode::KeyA | winit::keyboard::KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    winit::keyboard::KeyCode::KeyS | winit::keyboard::KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    winit::keyboard::KeyCode::KeyD | winit::keyboard::KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    winit::keyboard::KeyCode::KeyR => {
                        if is_pressed {
                            self.reset_orientation();
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::{InnerSpace, Rad, Deg};
        
        // Calculate camera direction from yaw and pitch
        let yaw_rad = cgmath::Rad::from(cgmath::Deg(self.yaw));
        let pitch_rad = cgmath::Rad::from(cgmath::Deg(self.pitch));
        
        // Calculate forward direction
        let forward_x = yaw_rad.0.cos() * pitch_rad.0.cos();
        let forward_y = pitch_rad.0.sin();
        let forward_z = yaw_rad.0.sin() * pitch_rad.0.cos();
        
        let forward = cgmath::Vector3::new(forward_x, forward_y, forward_z).normalize();
        
        // Calculate right direction (perpendicular to forward and up)
        let up = cgmath::Vector3::unit_y();
        let right = forward.cross(up).normalize();
        
        // Calculate up direction (perpendicular to forward and right)
        let camera_up = right.cross(forward).normalize();
        
        // Update camera position based on input
        let mut new_eye = camera.get_eye();
        
        if self.is_forward_pressed {
            new_eye += forward * self.speed;
        }
        if self.is_backward_pressed {
            new_eye -= forward * self.speed;
        }
        if self.is_right_pressed {
            new_eye += right * self.speed;
        }
        if self.is_left_pressed {
            new_eye -= right * self.speed;
        }
        
        // Update camera
        camera.set_eye(new_eye);
        camera.set_target(new_eye + forward);
        camera.set_up(camera_up);
    }

    pub fn reset_orientation(&mut self) {
        self.yaw = -90.0;
        self.pitch = 0.0;
    }
}

/// Camera system that manages camera positioning, uniforms, and GPU resources
/// This encapsulates all camera-related functionality that was previously in the renderer
pub struct CameraSystem {
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl CameraSystem {
    /// Create a new camera system with default settings
    pub fn new(device: &wgpu::Device) -> Self {
        let mut camera_controller = CameraController::new(0.1); // Increased speed for better responsiveness
        
        // Initialize camera with proper orientation
        let mut camera = Camera::new();
        
        // Set initial camera position and orientation
        let initial_position = cgmath::Point3::new(-6.0, 37.0, -6.0);
        camera.set_eye(initial_position);
        
        // Calculate initial target based on yaw and pitch
        let yaw_rad = cgmath::Rad::from(cgmath::Deg(camera_controller.yaw));
        let pitch_rad = cgmath::Rad::from(cgmath::Deg(camera_controller.pitch));
        
        let forward_x = yaw_rad.0.cos() * pitch_rad.0.cos();
        let forward_y = pitch_rad.0.sin();
        let forward_z = yaw_rad.0.sin() * pitch_rad.0.cos();
        
        let forward = cgmath::Vector3::new(forward_x, forward_y, forward_z).normalize();
        let target_position = initial_position + forward;
        camera.set_target(target_position);

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

        Self {
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        }
    }

    /// Update camera aspect ratio when window is resized
    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.camera.update_aspect(width, height);
    }

    /// Update camera controller and uniform data
    pub fn update(&mut self, queue: &wgpu::Queue) {
        // Update camera based on controller input
        self.camera_controller.update_camera(&mut self.camera);
        
        // Update camera uniform with new view-projection matrix
        self.camera_uniform.update_view_proj(&self.camera);
        
        // Write updated uniform data to GPU buffer
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    /// Process window events for camera input
    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    /// Calculate the center of all instances for camera positioning
    /// This method helps position the camera to look at the center of all rendered objects
    pub fn calculate_instances_center(&self, instances: &[Instance]) -> cgmath::Point3<f32> {
        if instances.is_empty() {
            // If no instances, return origin
            return cgmath::Point3::new(0.0, 0.0, 0.0);
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        // Find the bounding box of all instances
        //Note that if there 
        for instance in instances {
            min_x = min_x.min(instance.position.x);
            max_x = max_x.max(instance.position.x);
            min_y = min_y.min(instance.position.y);
            max_y = max_y.max(instance.position.y);
            min_z = min_z.min(instance.position.z);
            max_z = max_z.max(instance.position.z);
        }

        // Calculate center (ignore z for camera positioning as requested)
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let center_z = 10.0; // Set to ground level as requested

        cgmath::Point3::new(center_x, center_y, center_z)
    }

    /// Position camera to look at the center of all instances
    /// This automatically calculates the optimal camera position based on instance distribution
    pub fn position_camera_at_instances_center(&mut self, instances: &[Instance], queue: &wgpu::Queue) {
        let center = self.calculate_instances_center(instances);
        
        // Calculate the largest magnitude for x,y to determine camera distance
        let max_magnitude = instances.iter()
            .map(|instance| {
                let dx = (instance.position.x - center.x).abs();
                let dy = (instance.position.y - center.y).abs();
                dx.max(dy)
            })
            .fold(0.0, f32::max);

        // Set camera position: offset from center with appropriate height
        let camera_distance = (max_magnitude * 3.0).max(5.0); // At least 5 units away
        let camera_height = 3.0; // Fixed height above ground
        
        self.camera.set_eye(cgmath::Point3::new(
            center.x,
            center.y + camera_height,
            center.z + camera_distance
        ));
        
        // Set target to the center
        self.camera.set_target(center);
        
        // Update camera uniform and GPU buffer
        self.camera_uniform.update_view_proj(&self.camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    /// Reset camera to default position and update GPU buffer
    pub fn reset(&mut self, queue: &wgpu::Queue) {
        #[cfg(target_arch = "wasm32")]
        console::log_1(&"RESETTING CAMERA".into());
        
        self.camera.reset();
        self.camera_uniform.update_view_proj(&self.camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    /// Get reference to camera bind group layout for pipeline creation
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    /// Get reference to camera bind group for rendering
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }
}

/// Instance struct to hold position and rotation data for camera calculations
/// This is moved here from renderer.rs since it's used by camera positioning logic
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
} 