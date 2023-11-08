use crate::puzzle::cube::CubeRay;
use three_d::*;

pub mod puzzle;
pub mod render;
pub mod util;

fn main() {
    // all this is testing
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
    concrete_333.puzzle.twist((CubeRay::U, 1), &[1, 0]);
    concrete_333.puzzle.twist((CubeRay::R, 1), &[1, 0]);
    //println!("{:?}", concrete_333.stickers);

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        let geometry = render::concrete_puzzle_gm(&context, &concrete_333);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, geometry.into_iter(), &[]);

        for event in frame_input.events {
            match event {
                Event::MousePress {
                    button: MouseButton::Middle,
                    position,
                    ..
                } => {
                    // asdf
                    // pick(&context, &camera, position, concrete_333.stickers)
                    let sticker_m = concrete_333.ray_intersect(
                        &camera.position_at_pixel(position),
                        &camera.view_direction_at_pixel(position),
                    );
                    if let Some(sticker) = sticker_m {
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
                    }
                }
                _ => (),
            }
        }

        FrameOutput::default()
    });
}
