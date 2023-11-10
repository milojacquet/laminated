use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use crate::render::common::*;
use crate::render::create::*;
use crate::render::cube::*;
use std::collections::HashSet;
use three_d::*;

pub mod puzzle;
pub mod render;
pub mod util;

const TURN_DISTANCE_THRESHOLD: f32 = 3.0;
//const ORBIT_SPEED: f32 = 0.3;
const ORBIT_SPEED: f32 = 0.007; // radians per pixel
const ANIMATION_LENGTH: f32 = 150.0;
const ANIMATION_INIT_V: f32 = 0.1;

fn orbit_camera(camera: &mut Camera, &(dx, dy): &(f32, f32)) {
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
    for viewport in puzzle.viewports.iter_mut() {
        if viewport.conjugate == *conjugate {
            orbit_camera(&mut viewport.camera, delta);
        }
    }
}

fn main() {
    let window = Window::new(WindowSettings {
        title: "Laminated".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let mut concrete_puzzle = make_concrete_puzzle(&window);

    //make_viewports(&window, &mut concrete_puzzle);

    /*let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, -10.0, 4.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        degrees(22.0),
        0.1,
        1000.0,
    );*/

    /*let mut cube = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        ColorMaterial {
            color: Srgba::RED,
            ..Default::default()
        },
    );
    cube.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1.0));*/
    /*let mut sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(16)),
        ColorMaterial {
            color: Srgba::BLACK,
            ..Default::default()
        },
    );
    sphere.set_transformation(Mat4::from_translation(vec3(1.3, 0.0, 0.0)) * Mat4::from_scale(0.2));
    */

    // If the mouse is down, the time when it was first pressed.
    // It will be None if the mouse has moved farther than TURN_DISTANCE_THRESHOLD.
    // None: the mouse is not pressed.
    // Some((conj, None)): the mouse is being held from a viewport with conjugation conj, and camera orbiting has started.
    // Some((conj, Some((loc, button)))): the mouse is being held from a viewport with conjugation conj, and camera orbiting has not yet started. the moouse was pressed at loc with button.
    let mut mouse_press_location: Option<((), Option<(LogicalPoint, MouseButton)>)> = None;
    let mut keys_down: HashSet<Key> = HashSet::new();

    window.render_loop(move |frame_input| {
        //camera.set_viewport(frame_input.viewport);
        //let geometry = concrete_puzzle_gm(&(frame_input.elapsed_time as f32), &mut concrete_puzzle);
        //update_concrete_puzzle_gm(&(frame_input.elapsed_time as f32), &mut concrete_puzzle);
        //println!("{:?}", concrete_puzzle.stickers[0].animation);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

        //println!("{:?} {:?}", concrete_puzzle.viewports.len(), geometry.len());
        for viewport in &mut concrete_puzzle.viewports {
            frame_input.screen().render(
                /**/
                &viewport.camera,
                viewport.stickers.iter_mut().map(|sticker| {
                    let puzzle = &concrete_puzzle.puzzle;
                    sticker.gm(
                        &viewport.context,
                        CubeRay::ray_to_color(
                            &puzzle.pieces[puzzle.permutation[sticker.piece_ind]].orientation
                                [sticker.color],
                        ),
                        frame_input.elapsed_time as f32,
                    )
                }),
                &[],
            );
            //println!("here!");
        }

        for event in frame_input.events {
            //println!("{:?}", event);
            match event {
                Event::MousePress {
                    button, position, ..
                } => {
                    mouse_press_location = Some((Default::default(), Some((position, button))));
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
                    mouse_press_location = None;
                    /*let sticker_m = concrete_puzzle.ray_intersect(
                        &camera.position_at_pixel(position),
                        &camera.view_direction_at_pixel(position),
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
                        } else if let Some((_, press_button)) = mouse_press_location {
                            if press_button == button {
                                let turn_direction = match button {
                                    three_d::MouseButton::Left => -1,
                                    three_d::MouseButton::Right => 1,
                                    _ => 0, // should never happen
                                };

                                let mut layer_offsets = Vec::new();
                                if keys_down.contains(&Key::Num1) {
                                    layer_offsets.push(0);
                                }
                                if keys_down.contains(&Key::Num2) {
                                    layer_offsets.push(1);
                                }
                                if keys_down.contains(&Key::Num3) {
                                    layer_offsets.push(2);
                                }
                                if layer_offsets.is_empty() {
                                    layer_offsets.push(0);
                                }

                                let opposite_axis = if CubeRay::AXIS_HEADS.contains(&sticker.face) {
                                    1
                                } else {
                                    -1
                                };

                                let turn_face = sticker.face;
                                for layer_offset in layer_offsets {
                                    concrete_puzzle.twist(
                                        &(turn_face, opposite_axis * turn_direction),
                                        &[
                                            opposite_axis * (1 - layer_offset),
                                            -opposite_axis * (1 - layer_offset),
                                        ],
                                    );
                                }
                            }
                        }
                    }*/
                }
                Event::KeyPress { kind, .. } => {
                    keys_down.insert(kind);
                }
                Event::KeyRelease { kind, .. } => {
                    keys_down.remove(&kind);
                }
                _ => (),
            }
        }

        FrameOutput::default()
    });
}
