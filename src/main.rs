use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use crate::render::common::*;
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
            viewport
                .camera
                .as_mut()
                .map(|camera| orbit_camera(camera, delta));
        }
    }
}

fn make_viewports<Ray: ConcreteRaySystem>(
    window: &Window,
    concrete_puzzle: &mut ConcretePuzzle<Ray>,
) {
    let min_abstract_x = concrete_puzzle
        .viewports
        .iter()
        .map(|viewport| viewport.abstract_viewport.x)
        .reduce(f32::min)
        .expect("at least one viewport");
    let min_abstract_y = concrete_puzzle
        .viewports
        .iter()
        .map(|viewport| viewport.abstract_viewport.y)
        .reduce(f32::min)
        .expect("at least one viewport");
    let max_abstract_x = concrete_puzzle
        .viewports
        .iter()
        .map(|viewport| viewport.abstract_viewport.x + viewport.abstract_viewport.width)
        .reduce(f32::max)
        .expect("at least one viewport");
    let max_abstract_y = concrete_puzzle
        .viewports
        .iter()
        .map(|viewport| viewport.abstract_viewport.y + viewport.abstract_viewport.height)
        .reduce(f32::max)
        .expect("at least one viewport");
    let (window_width, window_height) = window.size();
    let abstract_width = max_abstract_x - min_abstract_x;
    let abstract_height = max_abstract_y - min_abstract_y;
    let scale = f32::min(
        abstract_width as f32 / abstract_width,
        window_height as f32 / abstract_height,
    );

    let viewport_width = scale * abstract_width;
    let viewport_height = scale * abstract_height;
    let viewport_x0 = (window_width as f32 / 2.0 - viewport_width / 2.0).max(0.0);
    let viewport_y0 = (window_height as f32 / 2.0 - viewport_height / 2.0).max(0.0);

    for puzzle_viewport in concrete_puzzle.viewports.iter_mut() {
        //puzzle_viewport.viewport = Some(Viewport {
        let viewport = Viewport {
            x: (viewport_x0 + puzzle_viewport.abstract_viewport.x * scale).ceil() as i32,
            y: (viewport_y0 + puzzle_viewport.abstract_viewport.y * scale).ceil() as i32,
            width: (puzzle_viewport.abstract_viewport.width * scale).round() as u32,
            height: (puzzle_viewport.abstract_viewport.height * scale).round() as u32,
        };
        let context = window.gl();
        context.set_cull(Cull::Back);
        context.set_viewport(viewport);
        puzzle_viewport.context = Some(context);
        puzzle_viewport.camera = Some(Camera::new_perspective(
            window.viewport(),
            vec3(5.0, -10.0, 4.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            degrees(22.0),
            0.1,
            1000.0,
        ));
    }
}

fn main() {
    let window = Window::new(WindowSettings {
        title: "Laminated".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let mut concrete_333 = make_concrete_puzzle();

    make_viewports(&window, &mut concrete_333);

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
        let geometry = concrete_puzzle_gm(&(frame_input.elapsed_time as f32), &mut concrete_333);
        //println!("{:?}", concrete_333.stickers[0].animation);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

        //println!("{:?} {:?}", concrete_333.viewports.len(), geometry.len());
        for (viewport, geom) in concrete_333.viewports.iter().zip(geometry) {
            if let Some(camera) = &viewport.camera {
                frame_input.screen().render(&camera, geom.into_iter(), &[]);
                //println!("here!");
            }
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
                                &mut concrete_333,
                                &conjugate,
                                &(position.x - press_position.x, position.y - press_position.y),
                            )
                        }
                    }
                    Some((conjugate, None)) => {
                        orbit_cameras(&mut concrete_333, &conjugate, &delta);
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
                    /*let sticker_m = concrete_333.ray_intersect(
                        &camera.position_at_pixel(position),
                        &camera.view_direction_at_pixel(position),
                    );
                    if let Some(sticker) = sticker_m {
                        if button == MouseButton::Middle {
                            println!(
                                "sticker: {:?}, face = {:?}, color = {:?}",
                                concrete_333
                                    .puzzle
                                    .index_to_solved_piece(sticker.piece_ind)
                                    .layers,
                                sticker.face,
                                sticker.color
                            );
                            println!(
                                "piece: {:?}",
                                concrete_333.puzzle.pieces
                                    [concrete_333.puzzle.permutation[sticker.piece_ind]]
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
                                    concrete_333.twist(
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
