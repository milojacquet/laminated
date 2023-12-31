use crate::puzzle::common::*;
use crate::ANIMATION_INIT_V;
use enum_map::EnumMap;
use std::cmp;
use std::collections::HashMap;

use crate::preferences::Preferences;
use crate::util::{color, Mat4, Vec3};
use std::f32::consts::PI;
use three_d::*;

const DEFAULT_HEIGHT: Deg<f32> = Deg(20.0);

pub trait ConcreteRaySystem
where
    Self: RaySystem,
    Self::Conjugate: Default + Eq + Copy + enum_map::Enum + enum_map::EnumArray<CameraFacing>,
{
    type Conjugate;

    fn axis_to_transform(turn: (Self, i8), conjugate: Self::Conjugate) -> Mat4;

    /// Unit vector that points along a ray.
    fn ray_to_vec(&self, conjugate: Self::Conjugate) -> Vec3;

    /// Unit vector that points along the ray's axis head.
    fn axis_to_vec(&self, conjugate: Self::Conjugate) -> Vec3 {
        self.get_axis()[0].ray_to_vec(conjugate)
    }

    fn ray_to_color(prefs: &Preferences) -> &EnumMap<Self, color::Color>;
    fn ray_to_color_mut(prefs: &mut Preferences) -> &mut EnumMap<Self, color::Color>;
}

/// Simpler version of three_d::CpuMesh without the enums.
#[derive(Debug, Clone)]
pub struct SimpleMesh {
    pub positions: Vec<Vec3>,
    pub indices: Vec<u8>,
}

impl SimpleMesh {
    pub fn to_mesh(&self) -> CpuMesh {
        CpuMesh {
            positions: Positions::F32(self.positions.clone()),
            indices: Indices::U8(self.indices.clone()),
            ..Default::default()
        }
    }

    pub fn transform(&mut self, transform: &Mat4) {
        for pos in self.positions.iter_mut() {
            *pos = (transform * pos.extend(1.0)).truncate();
        }
    }
}

/// The initial data which will be symmetry-expanded into a sticker.
#[derive(Debug)]
pub struct StickerSeed<Ray>
where
    Ray: ConcreteRaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    pub layers: EnumMap<Ray, i8>,
    /// The face the sticker is on. Controls what turn is done when clicked.
    pub face: Ray,
    /// The face that controls the color of the sticker.
    pub color: Ray,
    /// The vertices of the polygon that makes up the sticker.
    pub vertices: Vec<Vec3>,
}

#[derive(Debug)]
pub struct StickerAnimation {
    /// the axis the sticker is turning around
    pub rotation_axis: Vec3,
    /// the angle the axis starts at
    pub start_angle: f32,
    /// the time remaining for the animation, in milliseconds
    pub time_remaining: f32,
}

//#[derive(Debug)]
pub struct Sticker<Ray>
where
    Ray: ConcreteRaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    pub piece_ind: usize,
    /// The face the sticker is on. Controls what turn is done when clicked.
    pub face: Ray,
    /// The face that controls the color of the sticker.
    pub color: Ray,
    /// The vertices of the polygon that makes up the sticker.
    pub vertices: Vec<Vec3>,
    pub gm: Gm<Mesh, ColorMaterial>,
    pub animation: Option<StickerAnimation>,
}

/// Smoothly maps 0 to 0 and 1 to 1, with derivative ANIMATION_INIT_V at 0 and 1.
pub fn cubic_interpolate(t: f32) -> f32 {
    ANIMATION_INIT_V * (2.0 * t * t * t - 3.0 * t * t + t) - (2.0 * t * t * t - 3.0 * t * t)
}

impl<Ray: ConcreteRaySystem> Sticker<Ray> {
    fn ray_intersect(&self, position: Vec3, direction: Vec3) -> Option<f32> {
        polygon_inds(self.vertices.len())
            .iter()
            .filter_map(|inds| {
                let verts = &inds.iter().map(|&i| self.vertices[i]).collect::<Vec<_>>()[..];
                ray_triangle_intersect(position, direction, verts)
            })
            .reduce(f32::min)
    }

    pub fn update_gm(&mut self, color: Srgba, elapsed_time: f32, animation_length: f32) {
        // can this section be written better
        let remove_animation;
        let sticker_mat;
        if let Some(animation) = &mut self.animation {
            animation.time_remaining -= elapsed_time;
            remove_animation = animation.time_remaining < 0.0;
        } else {
            remove_animation = false;
        }
        if remove_animation {
            self.animation = None;
        }
        if let Some(animation) = &mut self.animation {
            let sticker_angle = animation.start_angle
                * cubic_interpolate(animation.time_remaining / animation_length);
            sticker_mat = Mat4::from_axis_angle(animation.rotation_axis, Rad(sticker_angle));
        } else {
            sticker_mat = Mat4::identity();
        }

        self.gm.material.color = color;
        self.gm.set_transformation(sticker_mat);
    }
}

fn ray_triangle_intersect(position: Vec3, direction: Vec3, verts: &[Vec3]) -> Option<f32> {
    // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    const EPSILON: f32 = 0.0000001;
    let edge1 = verts[1] - verts[0];
    let edge2 = verts[2] - verts[0];
    let h = direction.cross(edge2);
    let a = edge1.dot(h);
    if a.abs() < EPSILON {
        return None;
    }
    let f = 1.0 / a;
    let s = position - verts[0];
    let u = f * s.dot(h);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = f * edge2.dot(q);

    if t > EPSILON {
        // ray intersection
        Some(t)
    } else {
        // This means that there is a line intersection but not a ray intersection.
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AbstractViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AbstractViewport {
    fn camera_height(&self) -> Deg<f32> {
        // dimensionless fov: Rad::cot(persp.fovy / two)
        Deg::atan(1.0 / (Deg::cot(DEFAULT_HEIGHT / 2.0) / self.height)) * 2.0
    }
}

/// A three_d::Camera without the viewport
pub struct CameraFacing {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

impl CameraFacing {
    pub fn orbit(&mut self, (dx, dy): (f32, f32)) {
        // (dx, dy) will never both be zero
        let pointing = -1.0 * self.position;
        // camera.up() does not have to be perpendicular to the view vector
        let local_x_axis = pointing.cross(self.up).normalize();
        let local_y_axis = pointing.cross(local_x_axis).normalize();
        let orbit_direction = dx * local_x_axis + dy * local_y_axis;
        let orbit_axis = orbit_direction.cross(pointing).normalize();
        let mat = Mat3::from_axis_angle(orbit_axis, Rad(-f32::hypot(dx, dy) * crate::ORBIT_SPEED));
        *self = CameraFacing {
            position: mat * (-1.0 * pointing),
            target: mat * self.target,
            up: mat * (-1.0 * local_y_axis),
        };
        /*
        // this has a weird bug where it slows down the more you rotate
        camera.rotate_around(
            &Vec3::new(0.0, 0.0, 0.0),
            dx * ORBIT_SPEED,
            dy * ORBIT_SPEED,
        )*/
    }
}

pub struct PuzzleViewport<Ray>
where
    Ray: ConcreteRaySystem,
{
    pub abstract_viewport: AbstractViewport,
    pub viewport: Viewport,
    pub conjugate: Ray::Conjugate,
    pub stickers: Vec<Sticker<Ray>>,
    pub key_layers: Vec<HashMap<Key, Vec<i8>>>,
}

pub struct ViewportSeed<Ray>
where
    Ray: ConcreteRaySystem,
{
    pub abstract_viewport: AbstractViewport,
    pub conjugate: Ray::Conjugate,
    pub stickers: Vec<StickerSeed<Ray>>,
    // This one only applies to the puzzle in this viewport, if the appropriate setting is chosen.
    pub key_layers: Vec<HashMap<Key, Vec<i8>>>,
}

impl<Ray: ConcreteRaySystem> PuzzleViewport<Ray> {
    pub fn make_camera(&self, cam: &CameraFacing) -> Camera {
        Camera::new_perspective(
            self.viewport,
            cam.position,
            cam.target,
            cam.up,
            self.abstract_viewport.camera_height(),
            0.1,
            1000.0,
        )
    }
}

impl<Ray: ConcreteRaySystem> PuzzleViewport<Ray> {
    pub fn ray_intersect(&self, position: Vec3, direction: Vec3) -> Option<&Sticker<Ray>> {
        self.stickers
            .iter()
            .filter_map(|sticker| {
                sticker
                    .ray_intersect(position, direction)
                    .map(|d| (d, sticker))
            })
            .reduce(|ds1, ds2| {
                cmp::min_by(ds1, ds2, |ds1, ds2| {
                    ds1.0.partial_cmp(&ds2.0).expect("not nan")
                })
            })
            .map(|ds| ds.1)
    }
}

pub struct PuzzleSeed<Ray>
where
    Ray: ConcreteRaySystem,
{
    pub grips: Vec<Vec<i8>>,
    pub viewports: Vec<ViewportSeed<Ray>>,
    pub key_layers: Vec<HashMap<Key, Vec<i8>>>,
}

pub struct ConcretePuzzle<Ray>
where
    Ray: ConcreteRaySystem,
{
    pub puzzle: Puzzle<Ray>,
    pub viewports: Vec<PuzzleViewport<Ray>>,
    /// The nth entry in here is a HashMap mapping keys to layers,
    /// for rays that are the nth in their axis.
    pub key_layers: Vec<HashMap<Key, Vec<i8>>>, // Key is a keyboard key and a HashMap key!
}

impl<Ray: ConcreteRaySystem> ConcretePuzzle<Ray> {
    pub fn twist(&mut self, (ray, order): (Ray, i8), grip: &[i8], animation_length: f32) {
        self.puzzle.twist((ray, order), grip);
        for viewport in self.viewports.iter_mut() {
            for sticker in viewport.stickers.iter_mut() {
                let piece_at_sticker = self.puzzle.index_to_solved_piece(sticker.piece_ind);
                if piece_at_sticker.grip_on_axis(ray) == grip {
                    sticker.animation = Some(StickerAnimation {
                        rotation_axis: ray.axis_to_vec(viewport.conjugate),
                        start_angle: (order as f32) * 2.0 * PI / (ray.order() as f32),
                        time_remaining: animation_length,
                    })
                }
            }
        }
    }

    pub fn reset_animations(&mut self) {
        for viewport in self.viewports.iter_mut() {
            for sticker in viewport.stickers.iter_mut() {
                sticker.animation = None;
            }
        }
    }
}

pub fn polygon_inds(verts: usize) -> Vec<[usize; 3]> {
    (2..verts).map(|i| [0, i - 1, i]).collect()
}
