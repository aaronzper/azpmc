use cgmath::InnerSpace;

use crate::settings::CAMERA_SPEED;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[derive(Debug)]
pub struct Controller {
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
}

/// Scene camera
#[derive(Debug)]
pub struct Camera {
    pub(super) eye: cgmath::Point3<f32>,
    pub(super) target: cgmath::Point3<f32>,
    pub(super) up: cgmath::Vector3<f32>,
    pub(super) aspect: f32,
    pub(super) fovy: f32,
    pub(super) znear: f32,
    pub(super) zfar: f32,
    pub controller: Controller,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            // position the camera 0.8 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 0.8).into(),
            // have it look at the origin
            target: (0.0, 0.0, -0.25).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: width / height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            controller: Controller { 
                is_forward_pressed: false,
                is_backward_pressed: false,
                is_left_pressed: false,
                is_right_pressed: false,
            }
        }
    }

    /// **"Where the Magic Happens"** per https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/#a-perspective-camera
/// 1. The view matrix moves the world to be at the position and rotation of the camera. It's essentially an inverse of whatever the transform matrix of the camera would be.
/// 2. The proj matrix warps the scene to give the effect of depth. Without this, objects up close would be the same size as objects far away.
/// 3. The coordinate system in Wgpu is based on DirectX and Metal's coordinate systems. That means that in normalized device coordinates (opens new window), the x-axis and y-axis are in the range of -1.0 to +1.0, and the z-axis is 0.0 to +1.0. The cgmath crate (as well as most game math crates) is built for OpenGL's coordinate system. This matrix will scale and translate our scene from OpenGL's coordinate system to WGPU's. We'll define it as follows.
    fn view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn update_camera(&mut self) {
        let forward = self.target - self.eye;
        let right = forward.cross(self.up).normalize();

        if self.controller.is_forward_pressed {
            self.eye += forward * CAMERA_SPEED;
            self.target += forward * CAMERA_SPEED;
        }
        if self.controller.is_backward_pressed {
            self.eye -= forward * CAMERA_SPEED;
            self.target -= forward * CAMERA_SPEED;
        }

        if self.controller.is_right_pressed {
            self.eye += right * CAMERA_SPEED;
            self.target += right * CAMERA_SPEED;
        }
        if self.controller.is_left_pressed {
            self.eye -= right * CAMERA_SPEED;
            self.target -= right * CAMERA_SPEED;
        }
    }
}

// This is so we can store this in a buffer
#[repr(C)]
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
        self.view_proj = camera.view_projection_matrix().into();
    }
}
