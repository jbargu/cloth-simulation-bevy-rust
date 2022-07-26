use bevy::ecs::schedule::ShouldRun;
use bevy::{core::FixedTimestep, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

mod util;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

const TIMESTEP: f64 = 0.1;

/// Array containing all nodes, addressable by inded
struct Grid(Vec<Vec<Entity>>);

#[derive(Component)]
pub struct PreviousPosition(Vec3);

#[derive(Component)]
pub struct Force(Vec3);

#[derive(Component)]
pub struct Mass(f32);

#[derive(Component)]
pub struct Edge {
    a: Entity,
    b: Entity,
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Index {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct Pinned;

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
        app.world.spawn().insert_bundle(camera_bundle);

        app.insert_resource(self.params)
            .insert_resource(Grid(grid))
            .add_startup_system(setup_wind)
            .add_plugin(EguiPlugin)
            .add_system(ui_side_panel)
            .add_system(handle_keyboard_input)
            .add_system(render_edges)
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(TIMESTEP))
                    .with_system(apply_gravity.label("apply_gravity"))
                    .with_system_set(
                        SystemSet::new()
                            .with_run_criteria(run_if_wind_enabled)
                            .with_system(apply_wind)
                            .after("apply_gravity")
                            .label("apply_wind"),
                    )
                    .with_system(
                        apply_spring_forces
                            .after("apply_wind")
                            .label("apply_spring_forces"),
                    )
                    .with_system(update_nodes.after("apply_spring_forces")),
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

            ui.add(egui::Slider::new(&mut params.k[0], 1.0..=20.0).text("Structural k"));
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

// This system applies gravity to Nodes without Pinned component
fn apply_gravity(params: Res<Params>, mut query: Query<(&mut Force, &Mass), Without<Pinned>>) {
    for (mut force, mass) in query.iter_mut() {
        force.0 = Vec3::new(0.0, -params.g, 0.0) * mass.0;
    }
}

fn apply_wind(
    windows: Res<Windows>,
    mut wind_waves: Query<(&mut WindWave, &mut Force), Without<Index>>,
    mut nodes: Query<(&Transform, &mut Force), With<Index>>,
) {
    let dt = TIMESTEP as f32;

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

fn apply_spring_forces(
    params: Res<Params>,
    edges: Query<&Edge>,
    mut nodes: Query<(&mut Transform, &mut Force, &Mass), With<Index>>,
) {
    for (i, edge) in edges.iter().enumerate() {
        let [(a_pos, mut a_force, a_mass), (b_pos, mut b_force, b_mass)] =
            nodes.many_mut([edge.a, edge.b]);

        let len = (a_pos.translation - b_pos.translation).length();
        let f = params.k[0] * -(params.r[0] - len) / (a_mass.0 + b_mass.0)
            * ((a_pos.translation - b_pos.translation) / len);

        a_force.0 -= f;
        b_force.0 += f;

        if i == 1 {
            //println!(
            //"{}, {}, {}, a_force: {}, {}, b_force: {}",
            //f, len, a_pos.translation, a_force.0, b_pos.translation, b_force.0
            //);

            println!(
                "trans: {}, b_force: {}, force: {}, len: {}",
                b_pos.translation, b_force.0, f, len
            );
        }
    }
}

// Calculates new node position based on Force component
fn update_nodes(
    mut set: ParamSet<(
        Query<(&Index, &Transform)>,
        Query<(&mut Transform, &mut PreviousPosition, &mut Force, &mut Mass), Without<Pinned>>,
    )>,
) {
    let dt = TIMESTEP as f32;

    for (mut pos, mut prev_pos, force, mass) in set.p1().iter_mut() {
        let v = (pos.translation - prev_pos.0) / dt;
        let v = v + (force.0 / mass.0) * dt;

        let new_pos = pos.translation + v * dt;
        prev_pos.0 = pos.translation;

        // New pos
        pos.translation = new_pos;
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
