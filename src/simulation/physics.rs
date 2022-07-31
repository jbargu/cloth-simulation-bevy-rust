use super::Params;
use bevy::prelude::*;

#[derive(Component)]
pub struct PreviousPosition(pub Vec3);

#[derive(Component)]
pub struct Force(pub Vec3);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Edge {
    pub a: Entity,
    pub b: Entity,
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Index {
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct Pinned;

pub fn physics_update(
    params: Res<Params>,
    edges: Query<&Edge>,
    mut nodes: Query<
        (
            &mut Transform,
            &mut PreviousPosition,
            &mut Force,
            &Mass,
            Option<&Pinned>,
        ),
        With<Index>,
    >,
) {
    let num_steps = 5;
    let step_dt = params.dt / num_steps as f32;

    for _ in 0..num_steps {
        apply_gravity(&params, &mut nodes);

        update_nodes(step_dt, &mut nodes);

        for _ in 0..3 {
            apply_spring_forces(step_dt, &params, &edges, &mut nodes);
        }
    }
}

// This system applies gravity to Nodes without Pinned component
fn apply_gravity(
    params: &Res<Params>,
    nodes: &mut Query<
        (
            &mut Transform,
            &mut PreviousPosition,
            &mut Force,
            &Mass,
            Option<&Pinned>,
        ),
        With<Index>,
    >,
) {
    for (_, _, mut force, mass, pinned) in nodes.iter_mut() {
        if let None = pinned {
            force.0 += Vec3::new(0.0, -params.g, 0.0) * mass.0;
        }
    }
}

fn apply_spring_forces(
    dt: f32,
    params: &Res<Params>,
    edges: &Query<&Edge>,
    nodes: &mut Query<
        (
            &mut Transform,
            &mut PreviousPosition,
            &mut Force,
            &Mass,
            Option<&Pinned>,
        ),
        With<Index>,
    >,
) {
    for (i, edge) in edges.iter().enumerate() {
        let [(mut a_pos, _, mut a_force, a_mass, a_pinned), (mut b_pos, _, mut b_force, b_mass, b_pinned)] =
            nodes.many_mut([edge.a, edge.b]);

        let difference = a_pos.translation - b_pos.translation;
        let distance = difference.length();

        let tension = params.r[0] - distance;

        let f = -(params.k[0] * tension);

        //println!(
        //"{} {} {} {} {}",
        //a_pos.translation, difference, distance, tension, f
        //);
        if let None = a_pinned {
            //a_pos.translation += (f / a_mass.0) * dt * dt;
            a_pos.translation += 0.5 * -((difference / distance) * f / a_mass.0) * dt * dt;
        }

        if let None = b_pinned {
            //b_force.0 -= (f / b_mass.0) * dt * dt;
            b_pos.translation += 0.5 * ((difference / distance) * f / b_mass.0) * dt * dt;
        }

        if i == params.num_nodes_x - 1 {
            //println!(
            //"{} {}, diff: {}, distance: {}, tension: {}, force: {}",
            //a_pos.translation, b_pos.translation, difference, distance, tension, f
            //);
            //println!("{} | {} | {}", f, a_force.0, b_force.0);
            //println!(
            //"{}, {}, {}, a_force: {}, {}, b_force: {}",
            //f, len, a_pos.translation, a_force.0, b_pos.translation, b_force.0
            //);

            //println!(
            //"trans: {}, b_force: {}, force: {}, len: {}",
            //b_pos.translation, b_force.0, f, len
            //);
        }
    }
}

// Calculates new node position based on Force component
fn update_nodes(
    dt: f32,
    nodes: &mut Query<
        (
            &mut Transform,
            &mut PreviousPosition,
            &mut Force,
            &Mass,
            Option<&Pinned>,
        ),
        With<Index>,
    >,
) {
    for (mut pos, mut prev_pos, mut force, mass, _) in nodes.iter_mut() {
        let dampen_factor = 0.99;
        //let v = (pos.translation - prev_pos.0) / dt;
        let a = force.0 / mass.0;
        //let v = v * 0.3;
        //let v = v + *dt;

        //let new_pos = pos.translation + v + (force.0 / mass.0) * dt;
        let new_pos =
            pos.translation + dampen_factor * (pos.translation - prev_pos.0) + a * dt * dt;
        prev_pos.0 = pos.translation;

        // New pos
        pos.translation = new_pos;

        force.0 = Vec3::ZERO;
    }
}
