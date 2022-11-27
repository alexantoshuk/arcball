//! An implementation of the [Shoemake Arcball Camera](https://www.talisman.org/~erlkonig/misc/shoemake92-arcball.pdf)
//! See the [cube example](https://github.com/Twinklebear/arcball/blob/master/examples/cube.rs) for an example
//! of use with [glium](https://crates.io/crates/glium).

use ultraviolet::{
    mat::{Mat3, Mat4},
    rotor::Rotor3,
    vec::{Vec2, Vec3, Vec4},
};

/// The Shoemake Arcball camera.
pub struct ArcballCamera {
    translation: Mat4,
    center_translation: Mat4,
    rotation: Rotor3,
    camera: Mat4,
    inv_camera: Mat4,
    zoom_speed: f32,
    inv_screen: [f32; 2],
}

impl ArcballCamera {
    /// Create a new Arcball camera focused at the `center` point, which will zoom at `zoom_speed`
    /// `screen` should be `[screen_width, screen_height]`.
    pub fn new(center: Vec3, zoom_speed: f32, screen: [f32; 2]) -> ArcballCamera {
        let mut cam = ArcballCamera {
            translation: Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            center_translation: Mat4::from_translation(center).inversed(),
            rotation: Rotor3::identity(),
            camera: Mat4::identity(),
            inv_camera: Mat4::identity(),
            zoom_speed,
            inv_screen: [1.0 / screen[0], 1.0 / screen[1]],
        };
        cam.update_camera();
        cam
    }
    /// Get the view matrix computed by the camera.
    pub fn get_mat4(&self) -> Mat4 {
        self.camera
    }
    /// Get the inverse view matrix
    pub fn get_inv_camera(&self) -> Mat4 {
        self.inv_camera
    }
    /// Get the camera eye position
    pub fn eye_pos(&self) -> Vec3 {
        Vec3::new(
            self.inv_camera[3].x,
            self.inv_camera[3].y,
            self.inv_camera[3].z,
        )
    }
    /// Get the camera view direction
    pub fn eye_dir(&self) -> Vec3 {
        let dir = self.inv_camera * Vec4::new(0.0, 0.0, -1.0, 0.0);
        Vec3::new(dir.x, dir.y, dir.z).normalized()
    }
    /// Get the camera view direction
    pub fn up_dir(&self) -> Vec3 {
        let dir = self.inv_camera * Vec4::new(0.0, 1.0, 0.0, 0.0);
        Vec3::new(dir.x, dir.y, dir.z).normalized()
    }
    /// Rotate the camera, mouse positions should be in pixel coordinates.
    ///
    /// Rotates from the orientation at the previous mouse position specified by `mouse_prev`
    /// to the orientation at the current mouse position, `mouse_cur`.
    pub fn rotate(&mut self, mouse_prev: Vec2, mouse_cur: Vec2) {
        let m_cur = Vec2::new(
            (mouse_cur.x * 2.0 * self.inv_screen[0] - 1.0).clamp(-1.0, 1.0),
            (1.0 - 2.0 * mouse_cur.y * self.inv_screen[1]).clamp(-1.0, 1.0),
        );
        let m_prev = Vec2::new(
            (mouse_prev.x * 2.0 * self.inv_screen[0] - 1.0).clamp(-1.0, 1.0),
            (1.0 - 2.0 * mouse_prev.y * self.inv_screen[1]).clamp(-1.0, 1.0),
        );
        let mouse_cur_ball = ArcballCamera::screen_to_arcball(m_cur);
        let mouse_prev_ball = ArcballCamera::screen_to_arcball(m_prev);
        self.rotation = mouse_cur_ball * mouse_prev_ball * self.rotation;
        self.update_camera();
    }
    /// Zoom the camera in by some amount. Positive values zoom in, negative zoom out.
    pub fn zoom(&mut self, amount: f32, elapsed: f32) {
        let motion = Vec3::new(0.0, 0.0, amount);
        self.translation =
            Mat4::from_translation(motion * self.zoom_speed * elapsed) * self.translation;
        self.update_camera();
    }
    /// Pan the camera following the motion of the mouse. The mouse delta should be in pixels.
    pub fn pan(&mut self, mouse_delta: Vec2) {
        let zoom_dist = self.translation[3][3].abs();
        let delta = Vec4::new(
            mouse_delta.x * self.inv_screen[0],
            -mouse_delta.y * self.inv_screen[1],
            0.0,
            0.0,
        ) * zoom_dist;
        let motion = self.inv_camera * delta;
        self.center_translation = Mat4::from_translation(Vec3::new(motion.x, motion.y, motion.z))
            * self.center_translation;
        self.update_camera();
    }
    /// Update the screen dimensions, e.g. if the window has resized.
    pub fn update_screen(&mut self, width: f32, height: f32) {
        self.inv_screen[0] = 1.0 / width;
        self.inv_screen[1] = 1.0 / height;
    }
    fn update_camera(&mut self) {
        self.camera = self.translation
            * Mat3::from(self.rotation).into_homogeneous()
            * self.center_translation;
        self.inv_camera = self.camera.inversed();
    }
    fn screen_to_arcball(p: Vec2) -> Rotor3 {
        let dist = p.mag_sq();
        // If we're on/in the sphere return the point on it
        if dist <= 1.0 {
            Rotor3::from_quaternion_array([p.x, p.y, f32::sqrt(1.0 - dist), 0.0])
        } else {
            let unit_p = p.normalized();
            Rotor3::from_quaternion_array([unit_p.x, unit_p.y, 0.0, 0.0])
        }
    }
}
