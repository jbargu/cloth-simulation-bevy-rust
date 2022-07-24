use bevy::prelude::Vec3;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

mod edge;
mod node;

#[derive(Default)]
struct Delta(i16);

struct Grid(Vec<Vec<Entity>>);

#[derive(Component, Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct PreviousPosition {
    x: f32,
    y: f32,
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Index {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct Pinned;

pub struct Simulation {
    num_nodes_x: i16,
    num_nodes_y: i16,
    default_width_diff: f32,
    default_height_diff: f32,
    width: u32,
    height: u32,
    pub world: World,
    schedule: Schedule,
}

impl Simulation {
    pub fn new(width: u32, height: u32) -> Simulation {
        // ECS vars
        let world = World::new();
        let schedule = Schedule::default();

        let mut sim = Simulation {
            num_nodes_x: 20,
            num_nodes_y: 10,
            default_width_diff: 40f32,
            default_height_diff: 40f32,
            width,
            height,
            world,
            schedule,
        };
        let mut grid: Vec<Vec<Entity>> = Vec::new();

        for k in 0..sim.num_nodes_y {
            let mut vec: Vec<Entity> = Vec::new();
            for i in 0..sim.num_nodes_x {
                let index = Index {
                    x: i as usize,
                    y: k as usize,
                };
                let pos = Position {
                    x: 10.0 + i as f32 * sim.default_width_diff,
                    y: 10.0 + k as f32 * sim.default_height_diff,
                };
                let prev_pos = PreviousPosition {
                    x: 10.0 + i as f32 * sim.default_width_diff,
                    y: 10.0 + k as f32 * sim.default_height_diff,
                };
                let id;
                if k == 0 {
                    id = sim
                        .world
                        .spawn()
                        .insert(index)
                        .insert(pos)
                        .insert(prev_pos)
                        .insert(Pinned {})
                        .id();
                } else {
                    id = sim
                        .world
                        .spawn()
                        .insert(index)
                        .insert(pos)
                        .insert(prev_pos)
                        .id();
                }

                vec.push(id);
            }

            grid.push(vec);
        }
        sim.schedule
            .add_stage("update", SystemStage::parallel().with_system(apply_gravity));

        sim.world.insert_resource(Delta::default());
        sim.world.insert_resource(Grid(grid));

        sim
    }

    pub fn step(&mut self, delta: i16) {
        let mut delta_resource = self.world.get_resource_mut::<Delta>().unwrap();
        delta_resource.0 = delta;

        self.schedule.run(&mut self.world);
    }
}

// This system applies gravity to Nodes without Pinned component
fn apply_gravity(
    delta: Res<Delta>,
    grid: Res<Grid>,
    mut set: ParamSet<(
        Query<(&Index, &Position)>,
        Query<(&Index, &mut Position, &mut PreviousPosition), Without<Pinned>>,
    )>,
) {
    let dt = delta.0 as f32;

    let map: HashMap<Index, Position> = set
        .p0()
        .iter()
        .map(|(key, value)| return (*key, *value))
        .collect();

    for (ind, mut pos, mut prev_pos) in set.p1().iter_mut() {
        println!("{:?}", map.get(ind));
        //let vy = pos.y - prev_pos.y + dt.0 * 10;
        let vy_curr = pos.y - prev_pos.y;
        let vx_curr = pos.x - prev_pos.x;

        // Gravity force, F = m * a, assume m = 1
        let mut f: Vec3 = Vec3::new(0.0, 10.0, 0.0);

        f *= dt;

        let next_y = pos.y + (vy_curr + f.y) * dt;

        // Update prev pos
        prev_pos.y = pos.y;

        // New pos
        pos.y = next_y;

        // Test
        //if ind.y == 1 {
        //pos.y -= map.get(ind).unwrap().y;
        //}
    }
}
