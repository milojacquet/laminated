use crate::key_label::*;
use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use crate::render::common::*;
use crate::render::create::*;
use crate::render::cube::nnn_seeds;
use crate::session::*;
use itertools::Itertools;

use std::collections::HashSet;

use three_d::*;

pub mod key_label;
pub mod puzzle;
pub mod render;
pub mod session;
pub mod util;

const TURN_DISTANCE_THRESHOLD: f32 = 3.0;
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
]; // has to be an array?

fn orbit_camera(camera: &mut Camera, (dx, dy): (f32, f32)) {
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
    conjugate: Ray::Conjugate,
    delta: (f32, f32),
) {
    if delta == (0.0f32, 0.0f32) {
        return;
    }
    for viewport in puzzle.viewports.iter_mut() {
        if viewport.conjugate == conjugate {
            orbit_camera(&mut viewport.camera, delta);
        }
    }
}

fn get_viewport_from_pixel<Ray: ConcreteRaySystem>(
    concrete_puzzle: &ConcretePuzzle<Ray>,
    pixel: impl Into<PhysicalPoint>,
) -> Option<&PuzzleViewport<Ray>> {
    if concrete_puzzle.viewports.len() == 1 {
        // if there's only one viewport, clicking anywhere should trigger that viewport
        return Some(&concrete_puzzle.viewports[0]);
    }
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
    screen: &mut RenderTarget,
    elapsed_time: f64,
    context: &Context,
    concrete_puzzle: &mut ConcretePuzzle<Ray>,
) {
    screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

    let permutation = concrete_puzzle.puzzle.permutation();

    for viewport in &mut concrete_puzzle.viewports.iter_mut() {
        screen.render(
            &viewport.camera,
            viewport.stickers.iter_mut().map(|sticker| {
                let puzzle = &concrete_puzzle.puzzle;
                sticker.update_gm(
                    &context,
                    Ray::ray_to_color(
                        &puzzle.pieces[permutation[sticker.piece_ind]].orientation[sticker.color],
                    ),
                    elapsed_time as f32,
                );

                &sticker.gm
            }),
            &[],
        );
    }
}

#[allow(dead_code)]
fn render_axes<Ray: ConcreteRaySystem>(
    screen: &mut RenderTarget,
    context: &Context,
    concrete_puzzle: &ConcretePuzzle<Ray>,
) {
    for viewport in &concrete_puzzle.viewports {
        screen.render(&viewport.camera, &Axes::new(context, 0.1, 1.3), &[]);
    }
}

fn shortcut_button(
    ui: &mut egui::Ui,
    gui_context: &egui::Context,
    text: impl Into<egui::widget_text::WidgetText>,
    modifiers: egui::Modifiers,
    key: egui::Key,
) -> egui::Response {
    let mut button = egui::Button::new(text);
    button = button
        .shortcut_text(gui_context.format_shortcut(&egui::KeyboardShortcut { modifiers, key }));

    ui.add(button)
}

/// Does everything in the render loop, and if the puzzle changed, return the new puzzle.
fn run_render_loop<Ray: ConcreteRaySystem + std::fmt::Display>(
    frame_input: &mut FrameInput,
    session: &mut Session<Ray>,
    mouse_press_location: &mut Option<(Ray::Conjugate, Option<(LogicalPoint, MouseButton)>)>,
    keys_down: &mut HashSet<Key>,
    status_message: &mut Option<String>,
    context: &Context,
    window_size: &mut (u32, u32),
    gui: &mut GUI,
) -> Option<SessionEnum> {
    //println!("new frame");

    let mut new_session: Option<SessionEnum> = None;

    let new_window_size = (
        (frame_input.window_width as f32 * frame_input.device_pixel_ratio) as u32,
        (frame_input.window_height as f32 * frame_input.device_pixel_ratio) as u32,
    );

    if &new_window_size != window_size {
        *window_size = new_window_size;
        println!("resized to {:?}", window_size);
        update_viewports(*window_size, &mut session.concrete_puzzle);
    }

    gui.update(
        &mut frame_input.events,
        frame_input.accumulated_time,
        frame_input.viewport,
        frame_input.device_pixel_ratio,
        |gui_context| {
            use three_d::egui::*;
            TopBottomPanel::top("menu_bar").show(gui_context, |ui| {
                menu::bar(ui, |ui| {
                    //use egui::Modifiers::*;
                    /*ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            // …
                        }
                    });*/
                    ui.menu_button("Puzzle", |ui| {
                        ui.menu_button("Cube", |ui| {
                            for n in 2..=9 {
                                if ui.button(format!("{0} layers ({0}×{0}×{0})", n)).clicked() {
                                    new_session = Some(SessionEnum::Cube(
                                        CubePuzzle::Nnn(n),
                                        Session::from_concrete(make_concrete_puzzle(
                                            *window_size,
                                            &context,
                                            nnn_seeds(n),
                                        )),
                                    ));
                                }
                            }
                        });
                    });

                    ui.menu_button("Control", |ui| {
                        if ui.button("Scramble").clicked() {
                            session.scramble();
                        }
                        if ui.button("Reset").clicked() {
                            session.reset();
                        }
                        ui.separator();
                        if shortcut_button(ui, gui_context, "Undo", Modifiers::COMMAND, Key::Z)
                            .clicked()
                        {
                            session.undo();
                        }
                        if shortcut_button(
                            ui,
                            gui_context,
                            "Redo",
                            Modifiers::COMMAND | Modifiers::SHIFT,
                            Key::Z,
                        )
                        .clicked()
                        {
                            session.redo();
                        }
                        if shortcut_button(
                            ui,
                            gui_context,
                            "Do inverse",
                            Modifiers::COMMAND,
                            Key::X,
                        )
                        .clicked()
                        {
                            session.do_inverse();
                        }
                    });
                });
            });
            TopBottomPanel::bottom("status_bar").show(gui_context, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // this detects key presses one frame late
                    for i in (0..NUMBER_KEYS.len()).rev() {
                        let num_key = NUMBER_KEYS[i];
                        if session.concrete_puzzle.key_layers[0].contains_key(&num_key) {
                            ui.add(KeyLabel::new(
                                keys_down.contains(&num_key),
                                (i + 1).to_string(),
                            ));
                        }
                    }
                    ui.separator();

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if let Some(message) = &status_message {
                            ui.label(message.as_str());
                        } else if session.concrete_puzzle.puzzle.is_solved() {
                            ui.label("Solved!");
                        }
                    });
                });
            });
        },
    );

    for event in &frame_input.events {
        match *event {
            Event::MousePress {
                button, position, ..
            } => {
                if let Some(viewport_clicked) =
                    get_viewport_from_pixel(&session.concrete_puzzle, position)
                {
                    *mouse_press_location =
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
                    let distance_moved =
                        f32::hypot(position.x - press_position.x, position.y - press_position.y);
                    if distance_moved > TURN_DISTANCE_THRESHOLD {
                        orbit_cameras(
                            &mut session.concrete_puzzle,
                            *conjugate,
                            (position.x - press_position.x, position.y - press_position.y),
                        );
                        *mouse_press_location = Some((*conjugate, None));
                    }
                }
                Some((conjugate, None)) => {
                    orbit_cameras(&mut session.concrete_puzzle, *conjugate, delta);
                    // change default
                }
                None => {
                    // do not orbit the camera
                }
            },
            Event::MouseRelease {
                button, position, ..
            } => {
                *status_message = None;

                if let Some(viewport_clicked) =
                    get_viewport_from_pixel(&session.concrete_puzzle, position)
                {
                    let sticker_m = viewport_clicked.ray_intersect(
                        viewport_clicked.camera.position_at_pixel(position),
                        viewport_clicked.camera.view_direction_at_pixel(position),
                    );

                    if let Some(sticker) = sticker_m {
                        if button == MouseButton::Middle {
                            /*println!(
                                "sticker: {:?}, face = {:?}, color = {:?}",
                                session
                                    .concrete_puzzle
                                    .puzzle
                                    .index_to_solved_piece(sticker.piece_ind)
                                    .layers,
                                sticker.face,
                                sticker.color
                            );
                            println!(
                                "piece: {:?}",
                                session.concrete_puzzle.puzzle.pieces[session
                                    .concrete_puzzle
                                    .puzzle
                                    .permutation()[sticker.piece_ind]]
                            );*/
                            /*status_message = Some(format!(
                                "sticker: {:?}, face: {:?}, color: {:?}, piece: {:?}",
                                session
                                    .concrete_puzzle
                                    .puzzle
                                    .index_to_solved_piece(sticker.piece_ind)
                                    .layers,
                                sticker.face,
                                sticker.color,
                                session.concrete_puzzle.puzzle.pieces[session
                                    .concrete_puzzle
                                    .puzzle
                                    .permutation()[sticker.piece_ind]]
                            ));*/
                            *status_message = Some(format!(
                                "position: {}, face: {}, color: {}, piece: {}",
                                session
                                    .concrete_puzzle
                                    .puzzle
                                    .index_to_solved_piece(sticker.piece_ind),
                                sticker.face,
                                sticker.color,
                                session.concrete_puzzle.puzzle.pieces[session
                                    .concrete_puzzle
                                    .puzzle
                                    .permutation()[sticker.piece_ind]]
                            ));
                        } else if let Some((_conjugate, Some((_, press_button)))) =
                            mouse_press_location
                        {
                            if press_button == &button {
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
                                        session.concrete_puzzle.key_layers[axis_index]
                                            .get(&key)
                                            .clone()
                                    })
                                    .collect();
                                session.twist(
                                    turn,
                                    if grips.is_empty() {
                                        vec![viewport_clicked.default_layers[axis_index].clone()]
                                    } else {
                                        grips.into_iter().cloned().collect()
                                    },
                                );
                            }
                        }
                    }
                }

                *mouse_press_location = None;
            }
            Event::KeyPress {
                kind, modifiers, ..
            } => {
                //println!("pressed {:?}", kind);
                keys_down.insert(kind);

                let ctrl = modifiers.ctrl || modifiers.command;

                match (kind, modifiers.shift, ctrl) {
                    (Key::Z, false, true) => session.undo(),
                    (Key::Y, false, true) | (Key::Z, true, true) => session.redo(),
                    (Key::X, false, true) => session.do_inverse(),
                    _ => (),
                }
            }
            Event::KeyRelease { kind, .. } => {
                //println!("released {:?}", kind);
                keys_down.remove(&kind);
            }
            _ => (),
        }
    }

    // these should go above the events loop, otherwise the first turns will lag
    // maybe not
    render_puzzle(
        &mut frame_input.screen(),
        frame_input.elapsed_time,
        &context,
        &mut session.concrete_puzzle,
    );
    /*render_axes(
        &mut frame_input.screen(),
        &context,
        &session.concrete_puzzle,
    );*/

    frame_input.screen().write(|| gui.render());

    new_session
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
    let mut gui = GUI::new(&context);

    //let mut concrete_puzzle = make_concrete_puzzle(window.size(), &context, nnn_seeds(3));
    let mut session =
        SessionSeed::Cube(CubePuzzle::Nnn(3)).make_session_enum(window.size(), &context);

    // If the mouse is down, the time when it was first pressed.
    // It will be None if the mouse has moved farther than TURN_DISTANCE_THRESHOLD.
    // None: the mouse is not pressed.
    // Some((conj, None)): the mouse is being held from a viewport with conjugation conj, and camera orbiting has started.
    // Some((conj, Some((loc, button)))): the mouse is being held from a viewport with conjugation conj, and camera orbiting has not yet started. the mouse was pressed at loc with button.
    let mut mouse_press_location: Option<((), Option<(LogicalPoint, MouseButton)>)> = None;
    let mut keys_down: HashSet<Key> = HashSet::new();

    let mut window_size = window.size();
    let mut status_message: Option<String> = None;

    window.render_loop(move |mut frame_input| {
        let new_session;
        match &mut session {
            SessionEnum::Cube(_, ref mut session) => {
                new_session = run_render_loop(
                    &mut frame_input,
                    session,
                    &mut mouse_press_location,
                    &mut keys_down,
                    &mut status_message,
                    &context,
                    &mut window_size,
                    &mut gui,
                );
            }
        }

        if let Some(new_session) = new_session {
            session = new_session;
        }

        FrameOutput::default()
    });
}
