use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use three_d::*;

pub mod puzzle;
pub mod render;
pub mod util;

const TURN_DISTANCE_THRESHOLD: f32 = 3.0;
//const ORBIT_SPEED: f32 = 0.3;
const ORBIT_SPEED: f32 = 0.007; // radians per pixel

fn orbit_camera(camera: &mut Camera, &(dx, dy): &(f32, f32)) {
    let pointing = -1.0 * camera.position();
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

fn main() {
    let window = Window::new(WindowSettings {
        title: "Laminated".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, -10.0, 4.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        degrees(22.0),
        0.1,
        1000.0,
    );

    /*let mut cube = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        ColorMaterial {
            color: Srgba::RED,
            ..Default::default()
        },
    );
    cube.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(1.0));*/
    let mut sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(16)),
        ColorMaterial {
            color: Srgba::BLACK,
            ..Default::default()
        },
    );
    sphere.set_transformation(Mat4::from_translation(vec3(1.3, 0.0, 0.0)) * Mat4::from_scale(0.2));

    let mut concrete_333 = render::make_concrete_puzzle();
    //concrete_333.puzzle.twist((CubeRay::U, 1), &[1, 0]);
    //concrete_333.puzzle.twist((CubeRay::R, 1), &[1, 0]);
    //println!("{:?}", concrete_333.stickers);

    // If the mouse is down, the time when it was first pressed.
    // It will be None if the mouse has moved farther than TURN_DISTANCE_THRESHOLD.
    let mut mouse_press_location: Option<(LogicalPoint, MouseButton)> = None;

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        let geometry = render::concrete_puzzle_gm(&context, &concrete_333);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, geometry.into_iter(), &[]);

        for event in frame_input.events {
            //println!("{:?}", event);
            match event {
                Event::MousePress {
                    button, position, ..
                } => {
                    mouse_press_location = Some((position, button));
                }
                Event::MouseMotion {
                    button: Some(MouseButton::Left),
                    position,
                    delta,
                    ..
                } => match mouse_press_location {
                    Some((press_position, _)) => {
                        let distance_moved = f32::hypot(
                            position.x - press_position.x,
                            position.y - press_position.y,
                        );
                        if distance_moved > TURN_DISTANCE_THRESHOLD {
                            mouse_press_location = None;
                            orbit_camera(
                                &mut camera,
                                &(position.x - press_position.x, position.y - press_position.y),
                            );
                        }
                    }
                    None => {
                        orbit_camera(&mut camera, &delta);
                    }
                },
                Event::MouseRelease {
                    button, position, ..
                } => {
                    let sticker_m = concrete_333.ray_intersect(
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
                                if CubeRay::AXIS_HEADS.contains(&sticker.face) {
                                    concrete_333
                                        .puzzle
                                        .twist((sticker.face, turn_direction), &[1, 0]);
                                } else {
                                    concrete_333
                                        .puzzle
                                        .twist((sticker.face, -turn_direction), &[0, 1]);
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        FrameOutput::default()
    });
}
