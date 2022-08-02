mod physics;
mod ui;
mod util;

use bevy::sprite::Rect;
use bevy::{prelude::*, time::FixedTimestep};
use bevy_egui::EguiPlugin;
use bevy_prototype_debug_lines::*;
use bevy_prototype_lyon::prelude::*;
use physics::{
    apply_wind, physics_update, Edge, Force, Index, Mass, Pinned, PreviousPosition, WindWave,
};
use ui::{handle_mouse_interaction, run_if_wind_enabled, ui_side_panel, MainCamera};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

/// Array containing all nodes, addressable by inded
pub struct Grid(Vec<Vec<Entity>>);

pub struct Simulation {
    pub params: Params,
}

#[derive(Default, Clone, Copy)]
pub struct Params {
    /// if node shape is defined (circle), the circle is this big, off by default
    pub node_size: f32,
    pub num_nodes_x: usize,
    pub num_nodes_y: usize,
    /// timestep between two physics update
    pub dt: f32,
    /// default mass of the node
    pub m: f32,
    /// gravity constant
    pub g: f32,
    /// mouse click will cause so much force (increase +x)
    pub mouse_force: Vec3,
    /// rest lengths: structural, shear (unused), flexion (unused)
    pub r: Vec3,
    /// spring coefficients: structural, shear (unused), flexion(unused)
    pub k: Vec3,
    /// velocity dampen factor between constraint solving
    pub dampen_factor: f32,
    pub enable_wind: bool,

    // UI related params
    pub side_panel_width: f32,
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
                        //.insert_bundle(_shape_bundle)
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
                        //.insert_bundle(_shape_bundle)
                        .id();
                }

                vec.push(id);
            }

            grid.push(vec);
        }

        app.add_plugin(EguiPlugin)
            .add_plugin(DebugLinesPlugin::default())
            .insert_resource(self.params)
            .insert_resource(Grid(grid))
            .add_startup_system(setup_edges_system)
            .add_startup_system(setup_camera)
            .add_startup_system(setup_wind)
            .add_startup_system(update_canvas_size)
            .add_system(ui_side_panel)
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
                    .with_system(physics_update.label("physics_update").after("apply_wind")),
            )
            .add_system(render_edges.after("physics_update"));
    }
}

fn setup_edges_system(commands: Commands, params: ResMut<Params>, grid: Res<Grid>) {
    setup_edges(commands, grid, params.num_nodes_x, params.num_nodes_y);
}

/// Creates edges between the nodes in Grid
fn setup_edges(mut commands: Commands, grid: Res<Grid>, num_nodes_x: usize, num_nodes_y: usize) {
    for k in 0..num_nodes_y {
        for i in 0..num_nodes_x {
            // Add top edge
            if k > 0 {
                let line = shapes::Line(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));

                commands
                    .spawn()
                    .insert(Edge {
                        a: grid.0[k - 1][i],
                        b: grid.0[k][i],
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

                commands
                    .spawn()
                    .insert(Edge {
                        a: grid.0[k][i - 1],
                        b: grid.0[k][i],
                    })
                    .insert_bundle(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Stroke(StrokeMode::new(Color::WHITE, 1.0)),
                        Transform::default(),
                    ));
            }
        }
    }
}

fn setup_camera(mut commands: Commands, windows: Res<Windows>) {
    let window = util::get_primary_window_size(&windows);
    println!("window size: {}", window);

    let camera_bundle = Camera2dBundle::default();

    commands
        .spawn()
        .insert_bundle(camera_bundle)
        .insert(MainCamera)
        .insert(Transform::from_translation(Vec3::new(
            window.x / 2.0 - 100.0,
            -window.y / 2.0 + 40.0,
            0.0,
        )));
}

fn setup_wind(mut commands: Commands, windows: Res<Windows>) {
    let window = util::get_primary_window_size(&windows);

    println!("window size: {}", window);

    commands
        .spawn()
        .insert(WindWave {
            rect: Rect {
                min: Vec2::new(0.0, -1000.0),
                max: Vec2::new(window.x, 0.0),
            },
        })
        .insert(Force(Vec3::new(1000.0, 300.0, 0.0)));
}

fn render_edges(
    mut lines: ResMut<DebugLines>,
    mut edges: Query<&Edge>,
    mut nodes: Query<(Entity, &Transform), With<Index>>,
) {
    for edge in edges.iter_mut() {
        let [(_, a_pos), (_, b_pos)] = nodes.many_mut([edge.a, edge.b]);
        lines.line(a_pos.translation, b_pos.translation, 0.0);
    }
}

/// Resets nodes to initial position
pub fn reset_nodes_position(
    mut commands: Commands,
    params: &ResMut<Params>,
    grid: Res<Grid>,
    mut edges: Query<Entity, With<Edge>>,
    mut nodes: Query<(&Index, &mut Transform, &mut PreviousPosition)>,
) {
    for (index, mut pos, mut prev_pos) in nodes.iter_mut() {
        pos.translation = Vec3::new(
            index.x as f32 * params.r[0],
            -(index.y as f32 * params.r[0]),
            0.0,
        );

        prev_pos.0 = pos.translation.clone();
    }

    for entity in edges.iter_mut() {
        commands.entity(entity).despawn();
    }
    setup_edges(commands, grid, params.num_nodes_x, params.num_nodes_y);
}

/// Make sure the canvas is full screen on web
#[cfg(target_arch = "wasm32")]
fn update_canvas_size(mut windows: ResMut<Windows>) {
    let mut update_window = || {
        let browser_window = web_sys::window()?;
        let window_width = browser_window.inner_width().ok()?.as_f64()?;
        let window_height = browser_window.inner_height().ok()?.as_f64()?;
        let window = windows.get_primary_mut()?;
        window.set_resolution(window_width as f32, window_height as f32);
        Some(())
    };
    update_window();
}

#[cfg(not(target_arch = "wasm32"))]
fn update_canvas_size() {}
