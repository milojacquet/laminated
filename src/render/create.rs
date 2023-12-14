use crate::puzzle::common::*;
use crate::render::common::*;
use crate::util::enum_map_clone;

use enum_map::EnumMap;

use std::iter;

use three_d::*;

fn make_top_viewport(abstract_viewports: &[AbstractViewport]) -> AbstractViewport {
    let min_abstract_x = abstract_viewports
        .iter()
        .map(|av| av.x)
        .reduce(f32::min)
        .expect("at least one viewport");
    let min_abstract_y = abstract_viewports
        .iter()
        .map(|av| av.y)
        .reduce(f32::min)
        .expect("at least one viewport");
    let max_abstract_x = abstract_viewports
        .iter()
        .map(|av| av.x + av.width)
        .reduce(f32::max)
        .expect("at least one viewport");
    let max_abstract_y = abstract_viewports
        .iter()
        .map(|av| av.y + av.height)
        .reduce(f32::max)
        .expect("at least one viewport");

    AbstractViewport {
        x: min_abstract_x,
        y: min_abstract_y,
        width: max_abstract_x - min_abstract_x,
        height: max_abstract_y - min_abstract_y,
    }
}

fn make_viewport(
    window_size: (u32, u32),
    top_viewport: &AbstractViewport,
    abstract_viewport: &AbstractViewport,
) -> Viewport {
    let (window_width, window_height) = window_size;
    let scale = f32::min(
        window_width as f32 / top_viewport.width,
        window_height as f32 / top_viewport.height,
    );

    let viewport_width = scale * top_viewport.width;
    let viewport_height = scale * top_viewport.height;
    let viewport_x0 = (window_width as f32 / 2.0 - viewport_width / 2.0).max(0.0);
    let viewport_y0 = (window_height as f32 / 2.0 - viewport_height / 2.0).max(0.0);

    Viewport {
        x: (viewport_x0 + abstract_viewport.x * scale).ceil() as i32,
        y: (viewport_y0 + abstract_viewport.y * scale).ceil() as i32,
        width: (abstract_viewport.width * scale).round() as u32,
        height: (abstract_viewport.height * scale).round() as u32,
    }
}

pub fn correct_angle<A: Angle<Unitless = f32>>(angle: A, height: f32) -> A {
    // dimensionless fov: Rad::cot(persp.fovy / two)

    A::atan(1.0 / (A::cot(angle / 2.0) / height)) * 2.0
}

pub fn create_sticker_gm<Ray: ConcreteRaySystem>(
    context: &Context,
    vertices: &Vec<Vec3>,
    color: Ray,
) -> Gm<Mesh, ColorMaterial> {
    let mut cpu_mesh = CpuMesh {
        positions: Positions::F32(vertices.clone()),
        indices: Indices::U8(
            polygon_inds(vertices.len())
                .into_iter()
                .flatten()
                .map(|e| e as u8)
                .collect(),
        ),
        ..Default::default()
    };
    cpu_mesh.compute_normals();
    Gm::new(
        Mesh::new(&context, &cpu_mesh),
        ColorMaterial {
            color: Ray::ray_to_color(&color).to_srgba(),
            render_states: RenderStates {
                cull: Cull::Back,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

pub fn make_concrete_puzzle<Ray: ConcreteRaySystem>(
    window_size: (u32, u32),
    context: &Context,
    mut puzzle_seed: PuzzleSeed<Ray>,
) -> ConcretePuzzle<Ray> {
    let puzzle: Puzzle<Ray> = Puzzle::make_solved(puzzle_seed.grips);

    let top_viewport = make_top_viewport(
        &puzzle_seed
            .viewports
            .iter()
            .map(|vp| vp.abstract_viewport)
            .collect::<Vec<_>>()[..],
    );

    let viewports = puzzle_seed
        .viewports
        .iter_mut()
        .map(|viewport_seed| {
            let viewport =
                make_viewport(window_size, &top_viewport, &viewport_seed.abstract_viewport);

            let mut stickers = vec![];
            for seed in viewport_seed.stickers.iter_mut() {
                for turn_m in iter::once(None).chain(Ray::CYCLE.iter().map(Some)) {
                    if let Some(&turn) = turn_m {
                        let (turn_ray, turn_order) = turn;
                        seed.layers = EnumMap::from_fn(|ray: Ray| {
                            seed.layers[ray.turn((turn_ray, -turn_order))]
                        });
                        seed.face = seed.face.turn(turn);
                        seed.color = seed.color.turn(turn);

                        let mat = Ray::axis_to_transform(turn, viewport_seed.conjugate);
                        for vert in seed.vertices.iter_mut() {
                            *vert = (mat * vert.extend(1.0)).truncate();
                        }
                    }
                    let seed_layers_clone = enum_map_clone(&seed.layers);
                    let piece_ind =
                        puzzle.piece_to_index(&Piece::make_solved_from_layers(seed_layers_clone));

                    let gm = create_sticker_gm(context, &seed.vertices, seed.color);

                    stickers.push(Sticker {
                        piece_ind,
                        face: seed.face.clone(),
                        color: seed.color.clone(),
                        vertices: seed.vertices.clone(),
                        gm,
                        animation: None,
                    });
                }
            }

            PuzzleViewport {
                abstract_viewport: viewport_seed.abstract_viewport.clone(),
                viewport,
                camera: Camera::new_perspective(
                    viewport,
                    vec3(5.0, -10.0, 4.0),
                    vec3(0.0, 0.0, 0.0),
                    vec3(0.0, 0.0, 1.0),
                    correct_angle(degrees(20.0), viewport_seed.abstract_viewport.height),
                    0.1,
                    1000.0,
                ),
                conjugate: viewport_seed.conjugate,
                stickers,
                default_layers: viewport_seed.default_layers.clone(),
            }
        })
        .collect();

    ConcretePuzzle {
        puzzle,
        viewports,
        key_layers: puzzle_seed.key_layers,
    }
}

pub fn update_viewports<Ray: ConcreteRaySystem>(
    window_size: (u32, u32),
    concrete_puzzle: &mut ConcretePuzzle<Ray>,
) {
    let top_viewport = make_top_viewport(
        &concrete_puzzle
            .viewports
            .iter()
            .map(|vp| vp.abstract_viewport)
            .collect::<Vec<_>>()[..],
    );

    for puzzle_viewport in concrete_puzzle.viewports.iter_mut() {
        let viewport = make_viewport(
            window_size,
            &top_viewport,
            &puzzle_viewport.abstract_viewport,
        );
        puzzle_viewport.viewport = viewport;

        puzzle_viewport.camera = Camera::new_perspective(
            viewport,
            vec3(5.0, -10.0, 4.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            correct_angle(degrees(20.0), puzzle_viewport.abstract_viewport.height),
            0.1,
            1000.0,
        )
    }
}
