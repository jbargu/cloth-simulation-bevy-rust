use bevy::ecs::schedule::ShouldRun;
use bevy_egui::{egui, EguiContext};

use super::physics::{Edge, Force, Index, Pinned, PreviousPosition};
use super::Params;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

#[derive(Component)]
pub struct MainCamera;

pub fn ui_side_panel(
    mut egui_ctx: ResMut<EguiContext>,
    mut params: ResMut<Params>,
    query: Query<(&Index, &mut Transform, &mut PreviousPosition)>,
) {
    egui::SidePanel::right("side_panel")
        .default_width(params.side_panel_width)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.heading("Simulation controls");

            if ui.button("Reset").clicked() {
                super::reset_nodes_position(&params, query);
            }

            ui.add(egui::Slider::new(&mut params.g, 0.0..=20000.0).text("gravity"));

            ui.separator();
            ui.heading("Rest lengths");

            if ui
                .add(
                    egui::Slider::new(&mut params.r[0], 10.0..=100.0)
                        .text("Structural rest length"),
                )
                .drag_released()
            {
                let rest_length = params.r[0];
                params.calc_rest_lengths(rest_length);
            }

            ui.label(format!("Shear rest length: {}", params.r[1]));
            ui.label(format!("Flexion rest length: {}", params.r[2]));

            ui.separator();
            ui.heading("Spring coefficients");

            ui.add(egui::Slider::new(&mut params.k[0], 1.0..=5000.0).text("Structural k"));
            ui.add(egui::Slider::new(&mut params.k[1], 1.0..=20.0).text("Shear k"));
            ui.add(egui::Slider::new(&mut params.k[2], 1.0..=20.0).text("Flexion k"));

            ui.separator();
            ui.heading("Wind");
            ui.checkbox(&mut params.enable_wind, "Enable wind");

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "created by jbargu",
                    "https://github.com/jbargu",
                ));
            });
        });
}

pub fn handle_keyboard_input(
    keys: Res<Input<KeyCode>>,
    params: ResMut<Params>,
    query: Query<(&Index, &mut Transform, &mut PreviousPosition)>,
) {
    // Reset position of nodes
    if keys.just_released(KeyCode::R) {
        super::reset_nodes_position(&params, query);
    }
}

pub fn handle_mouse_interaction(
    mut commands: Commands,
    params: Res<Params>,
    buttons: Res<Input<MouseButton>>,
    wnds: Res<Windows>,
    mut edges: Query<(Entity, &Edge)>,
    mut q_camera: Query<(&Camera, &mut GlobalTransform), With<MainCamera>>,
    mut nodes: Query<(&Transform, &mut Force, Option<&Pinned>), With<Index>>,
) {
    if buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Right) {
        // get the camera info and transform
        // assuming there is exactly one main camera entity, so query::single() is OK
        let (camera, camera_transform) = q_camera.single();

        // get the window that the camera is displaying to (or the primary window)
        let wnd = if let RenderTarget::Window(id) = camera.target {
            wnds.get(id).unwrap()
        } else {
            wnds.get_primary().unwrap()
        };

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = wnd.cursor_position() {
            // get the size of the window
            let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix.inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            let world_pos: Vec2 = world_pos.truncate();

            eprintln!("World coords: {}/{}", world_pos.x, world_pos.y);

            if buttons.pressed(MouseButton::Left) {
                for (pos, mut force, pinned) in nodes.iter_mut() {
                    if pos.translation.truncate().distance(world_pos) < 150.0 {
                        if let None = pinned {
                            force.0 += params.mouse_force;
                        }
                    }
                }
            }
            if buttons.pressed(MouseButton::Right) {
                for (entity, edge) in edges.iter_mut() {
                    let [(a_pos, _, _), (b_pos, _, _)] = nodes.many_mut([edge.a, edge.b]);

                    if a_pos.translation.truncate().distance(world_pos) <= params.r[0]
                        || b_pos.translation.truncate().distance(world_pos) <= params.r[0]
                    {
                        // Remove the first matching edge - to avoid having big holes
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }
        }
    }
    if buttons.pressed(MouseButton::Middle) {
        let (_, mut camera_transform) = q_camera.single_mut();

        camera_transform.translation.y -= 10.0;
    }
}

pub fn run_if_wind_enabled(params: Res<Params>) -> ShouldRun {
    if params.enable_wind {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
