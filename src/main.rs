use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use simulation::{Params, Simulation};

mod simulation;

pub fn main() -> Result<(), String> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(Simulation::new(Params {
            node_size: 20.0,
            num_nodes_x: 20,
            num_nodes_y: 10,
            r: Vec3::new(40.0, 0.0, 0.0),
            k: Vec3::new(0.1, 0.5, 0.5),
            ..Default::default()
        }))
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();

    Ok(())
}
