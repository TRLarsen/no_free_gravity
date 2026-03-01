use bevy::prelude::*;

#[derive(Resource)]
pub struct GameConfig {
    // Player Movement
    pub player_thrust_power: f32,
    pub player_rotation_speed: f32,
    pub player_launch_multiplier: f32,

    // Physics
    pub gravity_constant: f32,

    // World Gen
    pub chunk_size: f32,
    pub chunk_load_distance: i32,
    pub asteroid_density_threshold: f64,
    pub asteroid_grid_divisions: i32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_thrust_power: 30000.0,
            player_rotation_speed: 3.5,
            player_launch_multiplier: 3.0,
            gravity_constant: 4000000.0,
            chunk_size: 1000.0,
            chunk_load_distance: 2,
            asteroid_density_threshold: 0.15,
            asteroid_grid_divisions: 4,
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>();
    }
}
