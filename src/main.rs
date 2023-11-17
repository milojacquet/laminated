use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use crate::render::common::*;
use crate::render::create::*;
use crate::render::cube::nnn_seeds;
use std::cmp::Ordering;

use std::collections::HashSet;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use three_d::*;

pub mod puzzle;
pub mod render;
pub mod util;

const TURN_DISTANCE_THRESHOLD: f32 = 3.0;
//const ORBIT_SPEED: f32 = 0.3;
const ORBIT_SPEED: f32 = 0.007; // radians per pixel
const ANIMATION_LENGTH: f32 = 150.0;
const ANIMATION_INIT_V: f32 = 0.1;
const NUMBER_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

fn orbit_camera(camera: &mut Camera, &(dx, dy): &(f32, f32)) {
    // (dx, dy) will never both be zero
    let pointing = -1.0 * camera.position();
    // camera.up() does not have to be perpendicular to the view vector
    let local_x_axis = pointing.cross(*camera.up()).normalize();
    let local_y_axis = pointing.cross(local_x_axis).normalize();
    let orbit_direction = dx * local_x_axis + dy * local_y_axis;
    let orbit_axis = orbit_direction.cross(pointing).normalize();
    let mat = Mat3::from_axis_angle(orbit_axis, Rad(-f32::hypot(dx, dy) * ORBIT_SPEED));
    camera.set_view(
        mat * (-1.0 * pointing),
        Vec3::new(0.0, 0.0, 0.0),
        mat * (-1.0 * local_y_axis),
    );
    /*
    // this has a weird bug where it slows down the more you rotate
    camera.rotate_around(
        &Vec3::new(0.0, 0.0, 0.0),
        dx * ORBIT_SPEED,
        dy * ORBIT_SPEED,
    )*/
}

fn orbit_cameras<Ray: ConcreteRaySystem>(
    puzzle: &mut ConcretePuzzle<Ray>,
    conjugate: &Ray::Conjugate,
    delta: &(f32, f32),
) {
    if delta == &(0.0f32, 0.0f32) {
        return;
    }
    for viewport in puzzle.viewports.iter_mut() {
        if viewport.conjugate == *conjugate {
            orbit_camera(&mut viewport.camera, delta);
        }
    }
}

fn get_viewport_from_pixel<'a, Ray: ConcreteRaySystem>(
    concrete_puzzle: &'a ConcretePuzzle<Ray>,
    pixel: impl Into<PhysicalPoint>,
) -> Option<&'a PuzzleViewport<Ray>> {
    let phys_pixel = pixel.into();
    for viewport in &concrete_puzzle.viewports {
        let vp = viewport.viewport;
        //dbg!(phys_pixel, vp);
        if (vp.x..vp.x + vp.width as i32).contains(&(phys_pixel.x as i32))
            && (vp.y..vp.y + vp.height as i32).contains(&(phys_pixel.y as i32))
        {
            return Some(viewport);
        }
    }
    None
}

fn render_puzzle<Ray: ConcreteRaySystem>(
    frame_input: &mut FrameInput,
    context: &Context,
    concrete_puzzle: &mut ConcretePuzzle<Ray>,
) {
    //sleep(Duration::from_millis(100));
    //return;

    frame_input
        .screen()
        .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

    for viewport in &mut concrete_puzzle.viewports.iter_mut() {
        frame_input.screen().render(
            //viewport.viewport.into(),
            &viewport.camera,
            viewport.stickers.iter_mut().map(|sticker| {
                let puzzle = &concrete_puzzle.puzzle;
                sticker.gm(
                    &context,
                    Ray::ray_to_color(
                        &puzzle.pieces[puzzle.permutation[sticker.piece_ind]].orientation
                            [sticker.color],
                    ),
                    frame_input.elapsed_time as f32,
                )
            }),
            &[],
        );
    }
}

fn main() {
    let window = Window::new(WindowSettings {
        title: "Laminated".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();
    context.set_cull(Cull::Back);

    let mut concrete_puzzle = make_concrete_puzzle(&window, nnn_seeds(7));

    // If the mouse is down, the time when it was first pressed.
    // It will be None if the mouse has moved farther than TURN_DISTANCE_THRESHOLD.
    // None: the mouse is not pressed.
    // Some((conj, None)): the mouse is being held from a viewport with conjugation conj, and camera orbiting has started.
    // Some((conj, Some((loc, button)))): the mouse is being held from a viewport with conjugation conj, and camera orbiting has not yet started. the moouse was pressed at loc with button.
    let mut mouse_press_location: Option<((), Option<(LogicalPoint, MouseButton)>)> = None;
    let mut keys_down: HashSet<Key> = HashSet::new();

    let mut window_size = window.size();

    window.render_loop(move |mut frame_input| {
        println!("new frame");
        //println!("size: {:?}", window_size);
        /*dbg!(concrete_puzzle
        .viewports
        .iter()
        .map(|v| v.viewport)
        .collect::<Vec<_>>());*/

        let new_window_size = (
            (frame_input.window_width as f32 * frame_input.device_pixel_ratio) as u32,
            (frame_input.window_height as f32 * frame_input.device_pixel_ratio) as u32,
        );
        if new_window_size != window_size {
            window_size = new_window_size;
            println!("resized to {:?}", window_size);
            update_viewports(window_size, &mut concrete_puzzle);
        }

        render_puzzle(&mut frame_input, &context, &mut concrete_puzzle);

        for event in frame_input.events {
            match event {
                Event::MousePress {
                    button, position, ..
                } => {
                    if let Some(viewport_clicked) =
                        get_viewport_from_pixel(&concrete_puzzle, position)
                    {
                        mouse_press_location =
                            Some((viewport_clicked.conjugate, Some((position, button))));
                    }
                }
                Event::MouseMotion {
                    button: Some(MouseButton::Left | MouseButton::Right),
                    position,
                    delta,
                    ..
                } => match mouse_press_location {
                    Some((conjugate, Some((press_position, _)))) => {
                        let distance_moved = f32::hypot(
                            position.x - press_position.x,
                            position.y - press_position.y,
                        );
                        if distance_moved > TURN_DISTANCE_THRESHOLD {
                            mouse_press_location = Some((conjugate, None));

                            orbit_cameras(
                                &mut concrete_puzzle,
                                &conjugate,
                                &(position.x - press_position.x, position.y - press_position.y),
                            )
                        }
                    }
                    Some((conjugate, None)) => {
                        orbit_cameras(&mut concrete_puzzle, &conjugate, &delta);
                        // change default
                    }
                    None => {
                        // do not orbit the camera
                    }
                },
                Event::MouseRelease {
                    button, position, ..
                } => {
                    if let Some(viewport_clicked) =
                        get_viewport_from_pixel(&concrete_puzzle, position)
                    {
                        let sticker_m = viewport_clicked.ray_intersect(
                            &viewport_clicked.camera.position_at_pixel(position),
                            &viewport_clicked.camera.view_direction_at_pixel(position),
                        );

                        if let Some(sticker) = sticker_m {
                            if button == MouseButton::Middle {
                                println!(
                                    "sticker: {:?}, face = {:?}, color = {:?}",
                                    concrete_puzzle
                                        .puzzle
                                        .index_to_solved_piece(sticker.piece_ind)
                                        .layers,
                                    sticker.face,
                                    sticker.color
                                );
                                println!(
                                    "piece: {:?}",
                                    concrete_puzzle.puzzle.pieces
                                        [concrete_puzzle.puzzle.permutation[sticker.piece_ind]]
                                );
                            } else if let Some((_conjugate, Some((_, press_button)))) =
                                mouse_press_location
                            {
                                if press_button == button {
                                    // TODO revise for conjugate
                                    let turn_direction = match button {
                                        three_d::MouseButton::Left => -1,
                                        three_d::MouseButton::Right => 1,
                                        _ => 0, // should never happen
                                    };

                                    let turn_face = sticker.face;
                                    let axis_index = turn_face
                                        .get_axis()
                                        .iter()
                                        .position(|&r| r == turn_face)
                                        .expect("rays are always in their axes");
                                    let opposite_axis = (-1i8).pow(axis_index as u32);
                                    let turn = (turn_face, opposite_axis * turn_direction);
                                    let grips: Vec<_> = keys_down
                                        .iter()
                                        .filter_map(|key| {
                                            concrete_puzzle.key_layers[axis_index].get(&key).clone()
                                        })
                                        .collect();
                                    for grip in if grips.is_empty() {
                                        vec![viewport_clicked.default_layers[axis_index].clone()]
                                    } else {
                                        grips.into_iter().cloned().collect()
                                    } {
                                        concrete_puzzle.twist(&turn, &grip.clone()[..]);
                                    }
                                }
                            }
                        }
                    }

                    mouse_press_location = None;
                }
                Event::KeyPress { kind, .. } => {
                    println!("pressed {:?}", kind);
                    keys_down.insert(kind);
                    //sleep(Duration::from_millis(20));
                }
                Event::KeyRelease { kind, .. } => {
                    println!("released {:?}", kind);
                    keys_down.remove(&kind);
                }
                _ => (),
            }
        }

        FrameOutput::default()
    });
}
