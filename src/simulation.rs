use bevy::ecs::schedule::ShouldRun;
use bevy::render::camera::RenderTarget;
use bevy::{core::FixedTimestep, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

mod physics;
mod util;

use physics::{physics_update, Edge, Force, Index, Mass, Pinned, PreviousPosition};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

/// Array containing all nodes, addressable by inded
struct Grid(Vec<Vec<Entity>>);

#[derive(Component)]
struct MainCamera;

pub struct Simulation {
    pub params: Params,
}

#[derive(Component)]
pub struct WindWave {
    rect: Rect<f32>,
}

#[derive(Default, Clone, Copy)]
pub struct Params {
    pub node_size: f32,
    pub num_nodes_x: usize,
    pub num_nodes_y: usize,
    pub dt: f32,
    pub m: f32,  // default mass of the node
    pub g: f32,  // gravity constant
    pub r: Vec3, // rest lengths: structural, shear, flexion
    pub k: Vec3, // spring coefficients: structural, shear, flexion
    pub enable_wind: bool,
}

impl Params {
    /// Calculates spring rest lengths based on the structural rest length.
    fn calc_rest_lengths(&mut self, structural_rest_length: f32) {
        self.r[0] = structural_rest_length;
        self.r[1] = self.r[0] * (2.0 as f32).sqrt(); // diagonal shear spring
        self.r[2] = self.r[0] * 2.0; // flexion spring, double the rest length
    }
}

impl Simulation {
    pub fn new(mut params: Params) -> Self {
        params.calc_rest_lengths(params.r[0]);
        Simulation { params }
    }
}

impl Plugin for Simulation {
    fn build(&self, app: &mut App) {
        let mut grid: Vec<Vec<Entity>> = Vec::new();

        let shape = shapes::Circle {
            radius: self.params.node_size,
            ..shapes::Circle::default()
        };

        // Create nodes
        for k in 0..self.params.num_nodes_y {
            let mut vec: Vec<Entity> = Vec::new();

            for i in 0..self.params.num_nodes_x {
                let index = Index {
                    x: i as usize,
                    y: k as usize,
                };
                let pos = Transform::from_xyz(
                    i as f32 * self.params.r[0],
                    -(k as f32 * self.params.r[0]),
                    0.0,
                );

                let prev_pos = PreviousPosition(Vec3::new(
                    i as f32 * self.params.r[0],
                    -(k as f32 * self.params.r[0]),
                    0.0,
                ));
                let _shape_bundle = GeometryBuilder::build_as(
                    &shape,
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::WHITE),
                        outline_mode: StrokeMode::new(Color::BLACK, 1.0),
                    },
                    pos,
                );
                let mass = Mass(self.params.m);
                let force = Force(Vec3::default());

                let id;
                if k == 0 {
                    id = app
                        .world
                        .spawn()
                        .insert(index)
                        .insert_bundle(TransformBundle::from(pos))
                        .insert(prev_pos)
                        .insert(mass)
                        .insert(force)
                        .insert(Pinned {})
                        //.insert_bundle(shape_bundle)
                        .id();
                } else {
                    id = app
                        .world
                        .spawn()
                        .insert(index)
                        .insert_bundle(TransformBundle::from(pos))
                        .insert(prev_pos)
                        .insert(mass)
                        .insert(force)
                        //.insert_bundle(shape_bundle)
                        .id();
                }

                vec.push(id);
            }

            grid.push(vec);
        }

        // Create edges
        for k in 0..self.params.num_nodes_y {
            for i in 0..self.params.num_nodes_x {
                // Add top edge
                if k > 0 {
                    let line = shapes::Line(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

                    app.world
                        .spawn()
                        .insert(Edge {
                            a: grid[k - 1][i],
                            b: grid[k][i],
                        })
                        .insert_bundle(GeometryBuilder::build_as(
                            &line,
                            DrawMode::Stroke(StrokeMode::new(Color::WHITE, 1.0)),
                            Transform::default(),
                        ));
                }

                // Add left edge
                if i > 0 {
                    let line = shapes::Line(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

                    app.world
                        .spawn()
                        .insert(Edge {
                            a: grid[k][i - 1],
                            b: grid[k][i],
                        })
                        .insert_bundle(GeometryBuilder::build_as(
                            &line,
                            DrawMode::Stroke(StrokeMode::new(Color::WHITE, 1.0)),
                            Transform::default(),
                        ));
                }
            }
        }

        // Add camera
        let camera_bundle = OrthographicCameraBundle::new_2d();
        app.world
            .spawn()
            .insert_bundle(camera_bundle)
            .insert(MainCamera);

        app.insert_resource(self.params)
            .insert_resource(Grid(grid))
            .add_startup_system(setup_wind)
            .add_plugin(EguiPlugin)
            .add_system(ui_side_panel)
            .add_system(handle_keyboard_input)
            //.add_system(render_edges)
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(self.params.dt.into()))
                    .with_system(handle_mouse_interaction.label("handle_mouse_interaction"))
                    .with_system_set(
                        SystemSet::new()
                            .with_run_criteria(run_if_wind_enabled)
                            .with_system(apply_wind)
                            .label("apply_wind")
                            .after("handle_mouse_interaction"),
                    )
                    .with_system(physics_update.label("physics_update").after("apply_wind"))
                    .with_system(render_edges.after("physics_update")),
            );
    }
}

fn run_if_wind_enabled(params: Res<Params>) -> ShouldRun {
    if params.enable_wind {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn setup_wind(mut commands: Commands, windows: Res<Windows>) {
    let window = util::get_primary_window_size(&windows);

    println!("window size: {}", window);

    commands
        .spawn()
        .insert(WindWave {
            rect: Rect {
                top: 0.0,
                left: 0.0,
                right: window.x,
                bottom: -1000.0,
            },
        })
        .insert(Force(Vec3::new(20.0, 0.0, 0.0)));
}

fn render_edges(
    mut set: ParamSet<(
        Query<(&mut Path, &Edge)>,
        Query<(Entity, &Transform), With<Index>>,
    )>,
) {
    let map: HashMap<Entity, Transform> = set
        .p1()
        .iter()
        .map(|(key, value)| return (key, *value))
        .collect();

    for (mut path, edge) in set.p0().iter_mut() {
        if let Some(a_pos) = map.get(&edge.a) {
            if let Some(b_pos) = map.get(&edge.b) {
                let line = shapes::Line(a_pos.translation.truncate(), b_pos.translation.truncate());
                *path = ShapePath::build_as(&line);
            }
        }
    }
}

fn ui_side_panel(
    mut egui_ctx: ResMut<EguiContext>,
    mut params: ResMut<Params>,
    query: Query<(&Index, &mut Transform, &mut PreviousPosition)>,
) {
    egui::SidePanel::left("side_panel")
        .default_width(300.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.heading("Simulation controls");

            if ui.button("Reset").clicked() {
                reset_nodes_position(&params, query);
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

            ui.add(egui::Slider::new(&mut params.k[0], 1.0..=500.0).text("Structural k"));
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

fn apply_wind(
    windows: Res<Windows>,
    params: Res<Params>,
    mut wind_waves: Query<(&mut WindWave, &mut Force), Without<Index>>,
    mut nodes: Query<(&Transform, &mut Force), With<Index>>,
) {
    let dt = params.dt;

    let window = util::get_primary_window_size(&windows);
    for (mut wave, wave_force) in wind_waves.iter_mut() {
        wave.rect.left += wave_force.0.x * dt;
        wave.rect.right += wave_force.0.x * dt;

        if wave.rect.left >= window.x {
            wave.rect.left -= window.x;
            wave.rect.right -= window.x;
        }

        wave.rect.top += wave_force.0.y * dt;
        wave.rect.bottom += wave_force.0.y * dt;

        if wave.rect.top >= window.y {
            wave.rect.top += window.y;
            wave.rect.bottom += window.y;
        }

        for (pos, mut node_force) in nodes.iter_mut() {
            if pos.translation.x >= wave.rect.left
                && pos.translation.x <= wave.rect.right
                && pos.translation.y >= wave.rect.bottom
                && pos.translation.y <= wave.rect.top
            {
                node_force.0 += wave_force.0;
            }
        }
    }
}

fn handle_keyboard_input(
    keys: Res<Input<KeyCode>>,
    params: ResMut<Params>,
    query: Query<(&Index, &mut Transform, &mut PreviousPosition)>,
) {
    // Reset position of nodes
    if keys.just_released(KeyCode::R) {
        reset_nodes_position(&params, query);
    }
}

/// Resets nodes to initial position
fn reset_nodes_position(
    params: &ResMut<Params>,
    mut query: Query<(&Index, &mut Transform, &mut PreviousPosition)>,
) {
    for (index, mut pos, mut prev_pos) in query.iter_mut() {
        pos.translation = Vec3::new(
            index.x as f32 * params.r[0],
            -(index.y as f32 * params.r[0]),
            0.0,
        );

        prev_pos.0 = pos.translation.clone();
    }
}
fn handle_mouse_interaction(
    buttons: Res<Input<MouseButton>>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut nodes: Query<(&Transform, &mut Force), (With<Index>, Without<Pinned>)>,
) {
    if buttons.just_released(MouseButton::Left) {
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

            for (pos, mut force) in nodes.iter_mut() {
                if pos.translation.truncate().distance(world_pos) < 150.0 {
                    force.0 += Vec3::new(8000.0, 0.0, 0.0);

                    println!("{}", pos.translation.truncate().distance(world_pos));
                }
            }

            eprintln!("World coords: {}/{}", world_pos.x, world_pos.y);
        }
    }
}
