use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub target: Vec3,
    pub eye: Vec3,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        let position = Vec3::new(0.0, 8.0, 8.0); // Inside arena bounds
        let target = Vec3::new(0.0, 0.0, 0.0);
        let forward_direction = Vec3::new(1.0, 0.0, 0.0).normalize();
        let yaw = forward_direction.x.atan2(forward_direction.z);
        let pitch = forward_direction.y.asin();
        Self {
            position,
            yaw,
            pitch,
            up: Vec3::Y,
            aspect: width as f32 / height as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0, // Increased render distance
            target,
            eye: position,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let direction = Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
        .normalize();
        let view = Mat4::look_at_rh(self.position, self.position + direction, self.up);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        let sensitivity = 0.005;
        self.yaw += (dx as f32) * sensitivity;
        self.pitch -= (dy as f32) * sensitivity;
        self.pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }
}
