use rapier3d::prelude::*;
use cgmath::{Vector3, Quaternion, Deg, Zero, Rotation3};
use std::collections::HashMap;

/// Physics body data that can be easily extracted for rendering
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub linear_velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub is_dynamic: bool,
}

/// Wrapper around Rapier3D physics world for easy integration
pub struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    // Mapping from Rapier handle to our physics body data
    body_data: HashMap<RigidBodyHandle, PhysicsBody>,
}

impl PhysicsWorld {
    /// Create a new physics world with default settings
    pub fn new() -> Self {
        //GUI: also have a slider where you can set the gravity
        let gravity = vector![0.0, -2.0, 0.0];
        let integration_parameters = IntegrationParameters::default();
        
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            gravity,
            integration_parameters,
            body_data: HashMap::new(),
        }
    }

    /// Add a static ground plane at y = 0
    pub fn add_ground(&mut self) -> ColliderHandle {
        let ground_collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0)
            .translation(vector![0.0, -0.1, 0.0])
            .build();
        
        self.collider_set.insert(ground_collider)
    }

    /// Add a dynamic cube at the specified position
    pub fn add_cube(&mut self, position: Vector3<f32>, size: f32) -> RigidBodyHandle {
        // Create rigid body
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x, position.y, position.z])
            .build();
        
        let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
        
        // Create collider
        let collider = ColliderBuilder::cuboid(size / 2.0, size / 2.0, size / 2.0)
            .build();
        
        self.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut self.rigid_body_set,
        );
        
        // Store initial physics body data
        self.body_data.insert(rigid_body_handle, PhysicsBody {
            position,
            rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0)),
            linear_velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            is_dynamic: true,
        });
        
        rigid_body_handle
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) {
        // Create a physics hooks object
        let physics_hooks = ();
        let event_handler = ();
        
        // Step the physics simulation
        let gravity = self.gravity;
        let integration_parameters = self.integration_parameters;
        
        let physics_pipeline = &mut self.physics_pipeline;
        let island_manager = &mut self.island_manager;
        let broad_phase = &mut self.broad_phase;
        let narrow_phase = &mut self.narrow_phase;
        let rigid_body_set = &mut self.rigid_body_set;
        let collider_set = &mut self.collider_set;
        
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            island_manager,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );
        
        // Update our cached physics body data from Rapier
        self.update_body_data();
    }

    /// Update our cached physics body data from Rapier
    fn update_body_data(&mut self) {
        for (handle, rigid_body) in self.rigid_body_set.iter() {
            let position = rigid_body.translation();
            let rotation = rigid_body.rotation();
            let linear_velocity = rigid_body.linvel();
            let angular_velocity = rigid_body.angvel();
            
            if let Some(body_data) = self.body_data.get_mut(&handle) {
                body_data.position = Vector3::new(position.x, position.y, position.z);
                body_data.rotation = Quaternion::new(rotation.w, rotation.i, rotation.j, rotation.k);
                body_data.linear_velocity = Vector3::new(linear_velocity.x, linear_velocity.y, linear_velocity.z);
                body_data.angular_velocity = Vector3::new(angular_velocity.x, angular_velocity.y, angular_velocity.z);
            }
        }
    }

    /// Get all physics bodies for rendering
    pub fn get_bodies(&self) -> &HashMap<RigidBodyHandle, PhysicsBody> {
        &self.body_data
    }

    /// Get a specific physics body by handle
    pub fn get_body(&self, handle: RigidBodyHandle) -> Option<&PhysicsBody> {
        self.body_data.get(&handle)
    }

    /// Apply a force to a rigid body
    pub fn apply_force(&mut self, handle: RigidBodyHandle, force: Vector3<f32>) {
        if let Some(rigid_body) = self.rigid_body_set.get_mut(handle) {
            rigid_body.add_force(vector![force.x, force.y, force.z], true);
        }
    }
}
