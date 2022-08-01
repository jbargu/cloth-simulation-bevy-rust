mod simulation;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use simulation::{Params, Simulation};

pub fn main() -> Result<(), String> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(Simulation::new(Params {
            node_size: 10.0,
            num_nodes_x: 50,
            num_nodes_y: 30,
            dt: 0.025,
            m: 1.0,
            g: 1000.0,
            mouse_force: Vec3::new(8000.0, 0.0, 0.0),
            r: Vec3::new(20.0, 0.0, 0.0),
            k: Vec3::new(1200.0, 1.0, 1.0),
            dampen_factor: 0.99,
            enable_wind: false,
            side_panel_width: 300.0,
            ..Default::default()
        }))
        .insert_resource(WindowDescriptor {
            fit_canvas_to_parent: true,
            ..default()
        })
        .run();

    Ok(())
}
