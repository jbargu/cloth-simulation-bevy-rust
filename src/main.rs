use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use simulation::{Params, Simulation};

mod simulation;

pub fn main() -> Result<(), String> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(Simulation::new(Params {
            node_size: 10.0,
            num_nodes_x: 40,
            num_nodes_y: 40,
            m: 1.0,
            g: 1000.0,
            r: Vec3::new(40.0, 0.0, 0.0),
            k: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        }))
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();

    Ok(())
}
