use crate::puzzle::common::*;
use crate::puzzle::cube::*;
use crate::render::common::*;

use enum_map::{enum_map, EnumMap};

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
    window: &winit::window::Window,
    top_viewport: &AbstractViewport,
    abstract_viewport: &AbstractViewport,
) -> Viewport {
    let (window_width, window_height): (u32, u32) = window
        .inner_size()
        .to_logical::<f64>(window.scale_factor())
        .into(); // https://docs.rs/three-d/latest/src/three_d/window/winit_window.rs.html#289-294
    let scale = f32::min(
        window_width as f32 / top_viewport.width,
        window_height as f32 / top_viewport.height,
    );

    let viewport_width = scale * top_viewport.width;
    let viewport_height = scale * top_viewport.height;
    let viewport_x0 = (window_width as f32 / 2.0 - viewport_width / 2.0).max(0.0);
    let viewport_y0 = (window_height as f32 / 2.0 - viewport_height / 2.0).max(0.0);

    //dbg!(scale, &abstract_viewports[i]);

    Viewport {
        x: (viewport_x0 + abstract_viewport.x * scale).ceil() as i32,
        y: (viewport_y0 + abstract_viewport.y * scale).ceil() as i32,
        width: (abstract_viewport.width * scale).round() as u32,
        height: (abstract_viewport.height * scale).round() as u32,
    }
}

pub fn make_concrete_puzzle<Ray: ConcreteRaySystem>(
    window: &winit::window::Window,
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
            let viewport = make_viewport(&window, &top_viewport, &viewport_seed.abstract_viewport);

            let mut stickers = vec![];
            for seed in viewport_seed.stickers.iter_mut() {
                for turn_m in iter::once(None).chain(Ray::CYCLE.iter().map(Some)) {
                    if let Some(turn) = turn_m {
                        let &(turn_ray, turn_order) = turn;
                        seed.layers = EnumMap::from_fn(|ray: Ray| {
                            seed.layers[ray.turn(&(turn_ray, -turn_order))]
                        });
                        seed.face = seed.face.turn(turn);
                        seed.color = seed.color.turn(turn);
                        seed.cpu_mesh
                            .transform(&Ray::axis_to_transform(turn, Default::default()))
                            .expect("the axis transform matrices should be invertible");
                    }
                    // ad hoc clone
                    let seed_layers_clone = EnumMap::from_fn(|ray: Ray| seed.layers[ray]);
                    let piece_ind =
                        puzzle.piece_to_index(&Piece::make_solved_from_layers(seed_layers_clone));
                    let mut new_cpu_mesh = seed.cpu_mesh.clone();

                    new_cpu_mesh.compute_normals();
                    stickers.push(Sticker {
                        piece_ind,
                        face: seed.face.clone(),
                        color: seed.color.clone(),
                        cpu_mesh: new_cpu_mesh,
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
                    degrees(20.0),
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

/*pub fn update_concrete_puzzle_gm<Ray: ConcreteRaySystem>(
    //contexts: &Vec<Context>,
    elapsed_time: &f32,
    concrete_puzzle: &mut ConcretePuzzle<Ray>,
) {
    let puzzle = &concrete_puzzle.puzzle;

    for viewport in concrete_puzzle.viewports.iter_mut() {
        for sticker in viewport.stickers.iter_mut() {
            sticker.gm.material.color = Ray::ray_to_color(
                &puzzle.pieces[puzzle.permutation[sticker.piece_ind]].orientation[sticker.color],
            );

            // can this section be written better
            let remove_animation;
            let sticker_mat;
            if let Some(animation) = &mut sticker.animation {
                animation.time_remaining -= elapsed_time;
                remove_animation = animation.time_remaining < 0.0;
            } else {
                remove_animation = false;
            }
            if remove_animation {
                sticker.animation = None;
            }
            if let Some(animation) = &mut sticker.animation {
                let sticker_angle = animation.start_angle
                    * cubic_interpolate(animation.time_remaining / ANIMATION_LENGTH);
                sticker_mat = Mat4::from_axis_angle(animation.rotation_axis, Rad(sticker_angle));
            } else {
                sticker_mat = Mat4::identity();
            }
            sticker.gm.set_transformation(sticker_mat);

            //screen.render(camera, sticker_mesh.into_iter(), &[]);
        }
    }
}*/
