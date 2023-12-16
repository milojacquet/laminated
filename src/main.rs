use crate::key_label::*;
use crate::preferences::Preferences;
use crate::puzzle::common::RaySystem;
use crate::puzzle::cube::CubeRay;
use crate::puzzle::octa::OctaRay;
use crate::render::common::*;
use crate::render::create::*;
use crate::session::*;
use crate::util::enum_iter;
use eyre::eyre;

use std::collections::HashSet;

use three_d::*;

pub mod key_label;
pub mod preferences;
pub mod puzzle;
pub mod render;
pub mod session;
pub mod util;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
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
    concrete_puzzle: &mut ConcretePuzzle<Ray>,
    prefs: &Preferences,
) {
    screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

    let permutation = concrete_puzzle.puzzle.permutation();

    for viewport in &mut concrete_puzzle.viewports.iter_mut() {
        screen.render(
            &viewport.camera,
            viewport.stickers.iter_mut().map(|sticker| {
                let puzzle = &concrete_puzzle.puzzle;
                sticker.update_gm(
                    Ray::ray_to_color(&prefs)
                        [puzzle.pieces[permutation[sticker.piece_ind]].orientation[sticker.color]]
                        .to_srgba(),
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

fn file_dialog() -> rfd::FileDialog {
    rfd::FileDialog::new()
        .add_filter("Log files", &["log"])
        .add_filter("All files", &["*"])
}

pub fn reset_button_small<T: PartialEq>(
    ui: &mut egui::Ui,
    value: &mut T,
    reset_value: T,
) -> egui::Response {
    let r = ui
        .add_enabled(*value != reset_value, egui::Button::new("⟲"))
        .on_hover_text("Reset");
    if r.clicked() {
        *value = reset_value;
    }
    r
}

fn color_picker_grid<Ray: ConcreteRaySystem>(
    name: &'static str,
    ui: &mut egui::Ui,
    prefs: &mut Preferences,
    color_picker_open: &mut bool,
) {
    use egui::*;

    ui.label(name);
    Grid::new(format!("{name}_color_grid"))
        .min_col_width(0.0)
        .show(ui, |ui| {
            for ray in enum_iter::<Ray>() {
                let mut color = Ray::ray_to_color(prefs)[ray].as_array();
                reset_button_small(
                    ui,
                    &mut Ray::ray_to_color_mut(prefs)[ray],
                    Ray::ray_to_color(&Default::default())[ray],
                );
                let color_picker = ui.color_edit_button_srgb(&mut color);
                if color_picker.changed() {
                    Ray::ray_to_color_mut(prefs)[ray] = color.into();
                    *color_picker_open = true;
                } else if color_picker.clicked_elsewhere() {
                    *color_picker_open = false
                }
                ui.label(ray.name());
                ui.end_row();
            }
        });
}

// polyfill from egui 0.24.1
fn selected_button(button: egui::Button, ui: &egui::Ui, selected: bool) -> egui::Button {
    if selected {
        let selection = ui.visuals().selection;
        button.fill(selection.bg_fill).stroke(selection.stroke)
    } else {
        button
    }
}

/// Mutable objects that have to persist through making a new session
struct PersistentObjects {
    keys_down: HashSet<Key>,
    keys_clicked: HashSet<Key>,
    status_message: Option<String>,
    window_size: (u32, u32),
    gui: GUI,
    prefs: Preferences,
    settings_open: bool,
    color_picker_open: bool,
}

impl PersistentObjects {
    fn show_or<T, E: std::fmt::Display>(
        &mut self,
        result: &Result<T, E>,
        default: impl Fn(&T) -> String,
    ) {
        match result {
            Ok(ok) => self.status_message = Some(default(ok)),
            Err(err) => self.status_message = Some(err.to_string()),
        };
    }

    fn show_err<T, E: std::fmt::Display>(&mut self, result: Result<T, E>) {
        if let Err(err) = result {
            self.status_message = Some(err.to_string());
        };
    }

    fn show_only_err<T, E: std::fmt::Display>(&mut self, result: Result<T, E>) {
        if let Err(err) = result {
            self.status_message = Some(err.to_string());
        } else {
            self.status_message = None
        };
    }

    fn save_prefs(&mut self) {
        match self.prefs.save() {
            Ok(()) => {
                self.status_message = Some("Saved preferences".to_string());
            }
            Err(err) => {
                self.status_message =
                    Some(format!("Error saving preferences: {}", err.to_string()));
            }
        }
    }

    fn load_prefs(&mut self) {
        match Preferences::load() {
            Ok(prefs) => {
                self.prefs = prefs;
            }
            Err(err) => {
                self.status_message =
                    Some(format!("Error loading preferences: {}", err.to_string()));
            }
        }
    }
}

enum Save {
    SavePath,
    SaveDefault,
}

/// A response that gets sent out of the render loop and into the main loop.
#[derive(Default)]
struct RenderLoopResponse {
    new_session: Option<SessionEnum>,
    save: Option<Save>,
    load: bool,
    save_prefs: bool,
    load_prefs: bool,
    replace_concrete_puzzle: bool,
}

/// Does everything in the render loop, and if the puzzle changed, return the new puzzle.
fn run_render_loop<Ray: ConcreteRaySystem + std::fmt::Display>(
    frame_input: &mut FrameInput,
    session: &mut Session<Ray>,
    persistent: &mut PersistentObjects,
    context: &Context,
) -> RenderLoopResponse {
    //println!("new frame");

    let mut response: RenderLoopResponse = Default::default();

    let new_window_size = (
        (frame_input.window_width as f32 * frame_input.device_pixel_ratio) as u32,
        (frame_input.window_height as f32 * frame_input.device_pixel_ratio) as u32,
    );

    if new_window_size != persistent.window_size {
        persistent.window_size = new_window_size;
        println!("resized to {:?}", persistent.window_size);
        update_viewports(persistent.window_size, &mut session.concrete_puzzle);
    }

    persistent.gui.update(
        &mut frame_input.events,
        frame_input.accumulated_time,
        frame_input.viewport,
        frame_input.device_pixel_ratio,
        |gui_context| {
            use egui::*;
            #[allow(non_snake_case)]
            let COMMAND = Modifiers::COMMAND;
            #[allow(non_snake_case)]
            let COMMAND_SHIFT = Modifiers::COMMAND | Modifiers::SHIFT;

            TopBottomPanel::top("menu_bar").show(gui_context, |ui| {
                menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if shortcut_button(ui, gui_context, "Open...", COMMAND, Key::O).clicked() {
                            response.load = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if shortcut_button(ui, gui_context, "Save", COMMAND, Key::S).clicked() {
                            response.save = Some(Save::SaveDefault);
                            ui.close_menu();
                        }
                        if shortcut_button(ui, gui_context, "Save as...", COMMAND_SHIFT, Key::S)
                            .clicked()
                        {
                            response.save = Some(Save::SavePath);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Puzzle", |ui| {
                        ui.menu_button("Cube", |ui| {
                            for n in 2..=9 {
                                if ui.button(format!("{0} layers ({0}×{0}×{0})", n)).clicked() {
                                    response.new_session = Some(
                                        SessionType::Cube(CubePuzzle::Nnn(n)).make_session_enum(
                                            persistent.window_size,
                                            &context,
                                            &persistent.prefs,
                                        ),
                                    );
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.menu_button("Octahedron", |ui| {
                            // bugged and not useful
                            /*if ui.button(format!("Core (testing)")).clicked() {
                                response.new_session = Some(
                                    SessionType::Octa(OctaPuzzle::Core)
                                        .make_session_enum(persistent.window_size, &context),
                                );
                                ui.close_menu();
                            }*/
                            for n in 2..=5 {
                                if ui.button(format!("{0} layers", n)).clicked() {
                                    response.new_session = Some(
                                        SessionType::Octa(OctaPuzzle::Fto(n)).make_session_enum(
                                            persistent.window_size,
                                            &context,
                                            &persistent.prefs,
                                        ),
                                    );
                                    ui.close_menu();
                                }
                            }
                        });
                    });

                    ui.menu_button("Control", |ui| {
                        if ui.button("Scramble").clicked() {
                            session.scramble();
                            ui.close_menu();
                        }
                        if ui.button("Reset").clicked() {
                            session.reset();
                            ui.close_menu();
                        }
                        ui.separator();
                        if shortcut_button(ui, gui_context, "Undo", COMMAND, Key::Z).clicked() {
                            // cannot borrow persistent so can't use show_only_err
                            if let Err(err) = session.undo() {
                                persistent.status_message = Some(err.to_string());
                            } else {
                                persistent.status_message = None;
                            };
                        }
                        if shortcut_button(ui, gui_context, "Redo", COMMAND_SHIFT, Key::Z).clicked()
                        {
                            if let Err(err) = session.redo() {
                                persistent.status_message = Some(err.to_string());
                            } else {
                                persistent.status_message = None;
                            };
                        }
                        if shortcut_button(ui, gui_context, "Do inverse", COMMAND, Key::X).clicked()
                        {
                            if let Err(err) = session.do_inverse() {
                                persistent.status_message = Some(err.to_string());
                            } else {
                                persistent.status_message = None;
                            };
                        }
                    });

                    if ui
                        .add(selected_button(
                            Button::new("Settings"),
                            ui,
                            persistent.settings_open,
                        ))
                        .clicked()
                    {
                        persistent.settings_open = !persistent.settings_open;
                    }
                });
            });

            TopBottomPanel::bottom("status_bar").show(gui_context, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // this detects key presses one frame late
                    for i in (0..NUMBER_KEYS.len()).rev() {
                        let num_key = NUMBER_KEYS[i];
                        let status = if persistent.keys_clicked.contains(&num_key) {
                            KeyLabelStatus::Clicked
                        } else if persistent.keys_down.contains(&num_key) {
                            KeyLabelStatus::Pressed
                        } else {
                            KeyLabelStatus::Unpressed
                        };
                        if session.concrete_puzzle.key_layers[0].contains_key(&num_key) {
                            if ui.add(KeyLabel::new(status, (i + 1).to_string())).clicked() {
                                if persistent.keys_clicked.contains(&num_key) {
                                    persistent.keys_clicked.remove(&num_key);
                                } else {
                                    persistent.keys_clicked.insert(num_key);
                                }
                            };
                        }
                    }
                    ui.separator();

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if let Some(message) = &persistent.status_message {
                            ui.label(message.as_str());
                        } else if session.concrete_puzzle.puzzle.is_solved() {
                            // if the message is None, we display one of the weaker messages
                            ui.label("Solved!");
                        }
                    });
                });
            });

            if persistent.settings_open {
                let frame = Frame::side_top_panel(&gui_context.style())
                    .fill(Color32::from_rgba_premultiplied(0, 0, 0, 222));
                let settings_panel = SidePanel::left("Settings").frame(frame);
                settings_panel.show(gui_context, |ui| {
                    ui.collapsing("Colors", |ui| {
                        color_picker_grid::<CubeRay>(
                            "Cube",
                            ui,
                            &mut persistent.prefs,
                            &mut persistent.color_picker_open,
                        );
                        color_picker_grid::<OctaRay>(
                            "Octahedron",
                            ui,
                            &mut persistent.prefs,
                            &mut persistent.color_picker_open,
                        );
                    });

                    ui.collapsing("Controls", |ui| {
                        ui.checkbox(
                            &mut persistent.prefs.viewport_keys,
                            "Per-viewport layer keys",
                        );
                    });

                    ui.collapsing("Puzzle form", |ui| {
                        if ui
                            .checkbox(&mut persistent.prefs.concrete.octa_extend, "FTO extensions")
                            .clicked()
                        {
                            response.replace_concrete_puzzle = true;
                        }
                    });

                    ui.separator();

                    if ui.button("Save preferences").clicked() {
                        response.save_prefs = true;
                    }

                    if ui.button("Reload preferences").clicked() {
                        response.load_prefs = true;
                    }

                    if ui.button("Reset preferences").clicked() {
                        persistent.prefs = Default::default();
                    }
                });
            }
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
                    session.mouse_press_location =
                        Some((viewport_clicked.conjugate, Some((position, button))));
                }
            }
            Event::MouseMotion {
                button: Some(MouseButton::Left | MouseButton::Right),
                position,
                delta,
                ..
            } => {
                if !persistent.color_picker_open {
                    match session.mouse_press_location {
                        Some((conjugate, Some((press_position, _)))) => {
                            let distance_moved = f32::hypot(
                                position.x - press_position.x,
                                position.y - press_position.y,
                            );
                            if distance_moved > TURN_DISTANCE_THRESHOLD {
                                persistent.status_message = None;

                                orbit_cameras(
                                    &mut session.concrete_puzzle,
                                    conjugate,
                                    (position.x - press_position.x, position.y - press_position.y),
                                );
                                session.mouse_press_location = Some((conjugate, None));
                            }
                        }
                        Some((conjugate, None)) => {
                            orbit_cameras(&mut session.concrete_puzzle, conjugate, delta);
                            // change default
                        }
                        None => {
                            // do not orbit the camera
                        }
                    }
                }
            }
            Event::MouseRelease {
                button, position, ..
            } => {
                if !persistent.color_picker_open {
                    if let Some(viewport_clicked) =
                        get_viewport_from_pixel(&session.concrete_puzzle, position)
                    {
                        let sticker_m = viewport_clicked.ray_intersect(
                            viewport_clicked.camera.position_at_pixel(position),
                            viewport_clicked.camera.view_direction_at_pixel(position),
                        );

                        if let Some(sticker) = sticker_m {
                            if button == MouseButton::Middle {
                                persistent.status_message = Some(format!(
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
                                session.mouse_press_location
                            {
                                persistent.status_message = None;

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

                                    let keys = persistent.keys_down.union(&persistent.keys_clicked);

                                    // if an invalid key is pressed on a puzzle while in viewport key mode,
                                    // there should be no turn instead of doing the default turn
                                    let default_mode = !keys.clone().any(|key| {
                                        session.concrete_puzzle.key_layers[0].contains_key(&key)
                                    });

                                    let grips;
                                    if default_mode {
                                        // rustfmt? hewwo?
                                        grips = vec![viewport_clicked.key_layers[axis_index]
                                            .get(&NUMBER_KEYS[0])
                                            .expect("viewport grips should have a binding for first key")
                                            .clone()];
                                    } else {
                                        let key_layers = if persistent.prefs.viewport_keys {
                                            &viewport_clicked.key_layers
                                        } else {
                                            &session.concrete_puzzle.key_layers
                                        };
                                        let grips_: Vec<_> = keys
                                            .filter_map(|key| {
                                                key_layers[axis_index].get(&key).clone()
                                            })
                                            .collect();

                                        grips = grips_.into_iter().cloned().collect();
                                    }

                                    session.twist(turn, grips);
                                }
                            }
                        }
                    }

                    session.mouse_press_location = None;
                }
            }
            Event::KeyPress {
                kind, modifiers, ..
            } => {
                persistent.keys_down.insert(kind);

                let ctrl = modifiers.ctrl || modifiers.command;

                persistent.show_err(match (kind, modifiers.shift, ctrl) {
                    (Key::Z, false, true) => session.undo(),
                    (Key::Y, false, true) | (Key::Z, true, true) => session.redo(),
                    (Key::X, false, true) => session.do_inverse(),
                    (Key::O, false, true) => Ok(response.load = true),
                    (Key::S, false, true) => Ok(response.save = Some(Save::SaveDefault)),
                    (Key::S, true, true) => Ok(response.save = Some(Save::SavePath)),
                    _ => Ok(()),
                });
            }
            Event::KeyRelease { kind, .. } => {
                persistent.keys_down.remove(&kind);
            }
            _ => (),
        }
    }

    // these should go above the events loop, otherwise the first turns will lag
    // maybe not
    render_puzzle(
        &mut frame_input.screen(),
        frame_input.elapsed_time,
        &mut session.concrete_puzzle,
        &persistent.prefs,
    );
    /*render_axes(
        &mut frame_input.screen(),
        &context,
        &session.concrete_puzzle,
    );*/

    frame_input.screen().write(|| persistent.gui.render());

    response
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

    let mut persistent = PersistentObjects {
        keys_down: HashSet::new(),
        keys_clicked: HashSet::new(),
        status_message: None,
        window_size: window.size(),
        gui: GUI::new(&context),
        prefs: Default::default(),
        settings_open: false,
        color_picker_open: false,
    };

    persistent.load_prefs();

    let mut session = SessionType::Cube(CubePuzzle::Nnn(3)).make_session_enum(
        persistent.window_size,
        &context,
        &persistent.prefs,
    );

    window.render_loop(move |mut frame_input| {
        let response;
        match &mut session {
            SessionEnum::Cube(_, ref mut session) => {
                response = run_render_loop(&mut frame_input, session, &mut persistent, &context);
            }
            SessionEnum::Octa(_, ref mut session) => {
                response = run_render_loop(&mut frame_input, session, &mut persistent, &context);
            }
        }

        if let Some(new_session) = response.new_session {
            session = new_session;
        }

        if let Some(save_type) = response.save {
            let save_path: Result<std::path::PathBuf, eyre::Report> =
                match (save_type, session.save_path()) {
                    (Save::SaveDefault, Some(save_path)) => Ok(save_path.clone()),
                    _ => file_dialog()
                        .save_file()
                        .ok_or_else(|| eyre!("No file picked")),
                };

            persistent.show_or(&save_path.and_then(|path| session.save_as(&path)), |path| {
                format!("Saved to {}", path.display()).to_string()
            });
        }

        if response.load {
            let load_path = file_dialog()
                .pick_file()
                .ok_or_else(|| eyre!("No file picked"));

            let load_result = load_path.and_then(|path| {
                SessionEnum::load(path, persistent.window_size, &context, &persistent.prefs)
            });
            persistent.show_or(&load_result, |session| {
                let suffix = if session.version() == VERSION {
                    "".to_string()
                } else {
                    format!(" from earlier version {}", session.version())
                };

                format!(
                    "Loaded {}{suffix}",
                    session
                        .save_path()
                        .clone()
                        .expect("the path should have been set")
                        .display()
                )
                .to_string()
            });
            if let Ok(new_session) = load_result {
                session = new_session;
            }
        }

        if response.save_prefs {
            persistent.save_prefs();
        }

        if response.load_prefs {
            persistent.load_prefs();
        }

        if response.replace_concrete_puzzle {
            session.replace_concrete_puzzle_from(session.get_type().make_session_enum(
                persistent.window_size,
                &context,
                &persistent.prefs,
            ))
        }

        FrameOutput::default()
    });
}
