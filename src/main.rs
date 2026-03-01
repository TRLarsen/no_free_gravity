use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod config;
mod environment;
mod physics;
mod player;
mod world;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Paused,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Space Astronaut Roguelike".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // State
        .init_state::<AppState>()
        // Config
        .add_plugins(config::ConfigPlugin)
        // Physics
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0).with_default_system_setup(true))
        .add_plugins(RapierDebugRenderPlugin::default()) // For development only
        // Our Plugins
        .add_plugins(player::PlayerPlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(world::WorldGenPlugin)
        .add_plugins(environment::EnvironmentPlugin)
        // Setup
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(AppState::Loading), transition_to_in_game)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn transition_to_in_game(mut next_state: ResMut<NextState<AppState>>) {
    // Immediately skip to InGame
    next_state.set(AppState::InGame);
}
