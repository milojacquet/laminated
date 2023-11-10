use crate::render::common::ConcreteRaySystem;
use crate::CubeRay;
use std::f32::consts::PI;
use three_d::*;
use CubeRay::*;

impl ConcreteRaySystem for CubeRay {
    type Conjugate = ();

    fn axis_to_transform(&(ray, order): &(Self, i8), _conjugate: Self::Conjugate) -> Mat4 {
        match ray {
            U | D => Mat4::from_angle_z(Rad(PI / 2.0 * (order as f32))),
            R | L => Mat4::from_angle_x(Rad(PI / 2.0 * (order as f32))),
            F | B => Mat4::from_angle_y(Rad(-PI / 2.0 * (order as f32))),
        }
    }

    fn axis_to_vec(&self, _conjugate: Self::Conjugate) -> Vec3 {
        match self {
            U | D => Vec3::new(0.0, 0.0, 1.0),
            R | L => Vec3::new(1.0, 0.0, 0.0),
            F | B => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    fn ray_to_color(&self) -> Srgba {
        match self {
            U => Srgba::WHITE,
            F => Srgba::RED,
            R => Srgba::BLUE,
            B => Srgba::new_opaque(255, 128, 0),
            L => Srgba::GREEN,
            D => Srgba::new_opaque(255, 255, 0),
        }
    }
}
