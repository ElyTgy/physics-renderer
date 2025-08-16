use cgmath;

use bytemuck;

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
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
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
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        
        //console::log_1(&format!("Camera position before: {:?}", camera.eye).into());
        
        // Calculate the forward direction (from eye to target)
        let forward = camera.get_target() - camera.get_eye();
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Calculate the right direction (perpendicular to forward and up)
        let right = forward_norm.cross(camera.get_up());

        // Handle forward/backward movement
        if self.is_forward_pressed {
            camera.set_eye(camera.get_eye() + forward_norm * self.speed);
            #[cfg(target_arch = "wasm32")]
            console::log_1(&"MOVING FORWARD".into());
        }
        if self.is_backward_pressed {
            camera.set_eye(camera.get_eye() - forward_norm * self.speed);
            #[cfg(target_arch = "wasm32")]
            console::log_1(&"MOVING BACKWARD".into());
        }

        // Handle left/right movement (strafe)
        if self.is_right_pressed {
            camera.set_eye(camera.get_eye() + right * self.speed);
            #[cfg(target_arch = "wasm32")]
            console::log_1(&"MOVING RIGHT".into());
        }
        if self.is_left_pressed {
            camera.set_eye(camera.get_eye() - right * self.speed);
            #[cfg(target_arch = "wasm32")]
            console::log_1(&"MOVING LEFT".into());
        }

        //console::log_1(&format!("Camera position after: {:?}", camera.eye).into());
    }
} 