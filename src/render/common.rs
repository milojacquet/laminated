use crate::puzzle::common::*;
use crate::puzzle::cube::*;
use crate::ANIMATION_INIT_V;
use crate::ANIMATION_LENGTH;
use enum_map::EnumMap;
use std::cmp;
use std::collections::HashMap;

use std::f32::consts::PI;
use three_d::*;

pub trait ConcreteRaySystem
where
    Self: RaySystem,
    Self::Conjugate: Default + Eq + Copy,
{
    type Conjugate;

    fn axis_to_transform(turn: &(Self, i8), conjugate: Self::Conjugate) -> Mat4;

    fn axis_to_vec(&self, conjugate: Self::Conjugate) -> Vec3;

    fn ray_to_color(&self) -> Srgba;
}

/// The initial data which will be symmetry-expanded into a sticker.
#[derive(Debug)]
pub struct StickerSeed<Ray>
where
    Ray: RaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    pub layers: EnumMap<Ray, i8>,
    /// The face the sticker is on. Controls what turn is done when clicked.
    pub face: Ray,
    /// The face that controls the color of the sticker.
    pub color: Ray,
    pub cpu_mesh: CpuMesh,
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
    Ray: RaySystem,
{
    /// The index of the piece this sticker is part of in `permutation` on `Puzzle`.
    pub piece_ind: usize,
    /// The face the sticker is on. Controls what turn is done when clicked.
    pub face: Ray,
    /// The face that controls the color of the sticker.
    pub color: Ray,
    /// The mesh of the sticker.
    pub cpu_mesh: CpuMesh,
    pub animation: Option<StickerAnimation>,
}

/// Smoothly maps 0 to 0 and 1 to 1, with derivative ANIMATION_INIT_V at 0 and 1.
pub fn cubic_interpolate(t: f32) -> f32 {
    ANIMATION_INIT_V * (2.0 * t * t * t - 3.0 * t * t + t) - (2.0 * t * t * t - 3.0 * t * t)
}

impl<Ray: RaySystem> Sticker<Ray> {
    fn ray_intersect(&self, position: &Vec3, direction: &Vec3) -> Option<f32> {
        let positions = self.cpu_mesh.positions.to_f32();
        let indices = self
            .cpu_mesh
            .indices
            .to_u32()
            .unwrap_or_else(|| (0..self.cpu_mesh.positions.len() as u32).collect());
        indices[..]
            .chunks_exact(3)
            .map(|inds| {
                let verts = &inds
                    .iter()
                    .map(|&i| positions[i as usize])
                    .collect::<Vec<_>>()[..];
                ray_triangle_intersect(position, direction, verts)
            })
            .filter_map(|x| x)
            .reduce(f32::min)
    }

    pub fn gm(
        &mut self,
        context: &Context,
        color: Srgba,
        elapsed_time: f32,
    ) -> Gm<Mesh, ColorMaterial> {
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
                * cubic_interpolate(animation.time_remaining / ANIMATION_LENGTH);
            sticker_mat = Mat4::from_axis_angle(animation.rotation_axis, Rad(sticker_angle));
        } else {
            sticker_mat = Mat4::identity();
        }

        let mut gm = Gm::new(
            Mesh::new(&context, &self.cpu_mesh),
            ColorMaterial {
                color,
                render_states: RenderStates {
                    cull: Cull::Back,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        gm.set_transformation(sticker_mat);
        gm
    }
}

fn ray_triangle_intersect(position: &Vec3, direction: &Vec3, verts: &[Vec3]) -> Option<f32> {
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

pub struct PuzzleViewport<Ray>
where
    Ray: ConcreteRaySystem,
{
    pub abstract_viewport: AbstractViewport,
    pub viewport: Viewport,
    pub camera: Camera,
    pub conjugate: Ray::Conjugate,
    pub stickers: Vec<Sticker<Ray>>,
    pub default_layers: Vec<Vec<i8>>,
}

pub struct ViewportSeed<Ray>
where
    Ray: ConcreteRaySystem,
{
    pub abstract_viewport: AbstractViewport,
    pub conjugate: Ray::Conjugate,
    pub stickers: Vec<StickerSeed<Ray>>,
    pub default_layers: Vec<Vec<i8>>,
}

impl<Ray: ConcreteRaySystem> PuzzleViewport<Ray> {
    pub fn ray_intersect(&self, position: &Vec3, direction: &Vec3) -> Option<&Sticker<Ray>> {
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
    pub fn twist(&mut self, &(ray, order): &(Ray, i8), grip: &[i8]) {
        let twisted = self.puzzle.twist((ray, order), grip);
        //println!("{:?}", twisted);
        for viewport in self.viewports.iter_mut() {
            for sticker in viewport.stickers.iter_mut() {
                if twisted.contains(&sticker.piece_ind) {
                    sticker.animation = Some(StickerAnimation {
                        //rotation_axis: Ray::axis_to_vec(&ray, Default::default()),
                        rotation_axis: ray.axis_to_vec(viewport.conjugate),
                        start_angle: (order as f32) * 2.0 * PI / (ray.order() as f32),
                        time_remaining: ANIMATION_LENGTH,
                    })
                }
            }
        }
    }
}

pub fn polygon(verts: Vec<Vec3>) -> CpuMesh {
    let indices = Indices::U8(
        (2..verts.len() as u8)
            .map(|i| vec![0, i - 1, i])
            .flatten()
            .collect(),
    );
    CpuMesh {
        positions: Positions::F32(verts),
        indices,
        ..Default::default()
    }
}
