use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_gravity.run_if(in_state(crate::AppState::InGame)));
    }
}

#[derive(Component)]
pub struct GravitySource {
    pub mass: f32,
    pub radius: f32,
}

fn apply_gravity(
    time: Res<Time>,
    config: Res<crate::config::GameConfig>,
    gravity_sources: Query<(&Transform, &GravitySource)>,
    mut bodies: Query<(&Transform, &mut ExternalImpulse), (With<RigidBody>, Without<GravitySource>)>,
) {
    let dt = time.delta_secs();

    for (body_transform, mut impulse) in bodies.iter_mut() {
        let mut total_grav_impulse = Vec2::ZERO;
        let body_pos = body_transform.translation.truncate();

        for (grav_transform, grav_source) in gravity_sources.iter() {
            let grav_pos = grav_transform.translation.truncate();
            let dist_vec = grav_pos - body_pos;
            let dist_sq = dist_vec.length_squared();

            let min_dist = 50.0 * 50.0; // Cap max gravity
            if dist_sq > min_dist && dist_sq < grav_source.radius * grav_source.radius {
                let force_mag = config.gravity_constant * grav_source.mass / dist_sq;
                total_grav_impulse += dist_vec.normalize() * force_mag * dt;
            }
        }

        impulse.impulse += total_grav_impulse;
    }
}
