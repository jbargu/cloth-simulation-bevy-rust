use super::util;
use super::Params;
use bevy::prelude::*;
use bevy::sprite::Rect;

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

#[derive(Component)]
pub struct WindWave {
    pub rect: Rect,
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

        update_nodes(step_dt, &params, &mut nodes);

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
    for edge in edges.iter() {
        let [(mut a_pos, _, _, a_mass, a_pinned), (mut b_pos, _, _, b_mass, b_pinned)] =
            nodes.many_mut([edge.a, edge.b]);

        let difference = a_pos.translation - b_pos.translation;
        let distance = difference.length();
        let tension = params.r[0] - distance;

        let f = -(params.k[0] * tension);

        if let None = a_pinned {
            a_pos.translation += 0.5 * -((difference / distance) * f / a_mass.0) * dt * dt;
        }

        if let None = b_pinned {
            b_pos.translation += 0.5 * ((difference / distance) * f / b_mass.0) * dt * dt;
        }
    }
}

// Calculates new node position based on Force component
fn update_nodes(
    dt: f32,
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
    for (mut pos, mut prev_pos, mut force, mass, _) in nodes.iter_mut() {
        let a = force.0 / mass.0;

        let new_pos =
            pos.translation + params.dampen_factor * (pos.translation - prev_pos.0) + a * dt * dt;
        prev_pos.0 = pos.translation;

        // New pos
        pos.translation = new_pos;

        force.0 = Vec3::ZERO;
    }
}

pub fn apply_wind(
    windows: Res<Windows>,
    params: Res<Params>,
    mut wind_waves: Query<(&mut WindWave, &mut Force), Without<Index>>,
    mut nodes: Query<(&Transform, &mut Force), (With<Index>, Without<Pinned>)>,
) {
    let dt = params.dt;

    let window = util::get_primary_window_size(&windows);
    for (mut wave, wave_force) in wind_waves.iter_mut() {
        wave.rect.min.x += wave_force.0.x * dt;
        wave.rect.max.x += wave_force.0.x * dt;

        if wave.rect.min.x >= window.x {
            wave.rect.min.x -= window.x;
            wave.rect.max.x -= window.x;
        }

        //wave.rect.min.y += wave_force.0.y * dt;
        //wave.rect.max.y += wave_force.0.y * dt;

        //if wave.rect.min.y >= window.y {
        //wave.rect.min.y += window.y;
        //wave.rect.max.y += window.y;
        //}

        for (pos, mut node_force) in nodes.iter_mut() {
            if pos.translation.x >= wave.rect.min.x
                && pos.translation.x <= wave.rect.max.x
                && pos.translation.y >= wave.rect.min.y
                && pos.translation.y <= wave.rect.max.y
            {
                node_force.0 += wave_force.0;
            }
        }
    }
}
