use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashSet;
use noise::{NoiseFn, OpenSimplex};

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadedChunks {
            active_chunks: HashSet::new(),
        })
        .add_systems(Update, manage_chunks.run_if(in_state(crate::AppState::InGame)));
    }
}



#[derive(Resource)]
pub struct LoadedChunks {
    active_chunks: HashSet<(i32, i32)>,
}

#[derive(Component)]
pub struct ChunkDesc {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Asteroid {
    pub radius: f32,
}

fn manage_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut existing_chunks: Query<(Entity, &ChunkDesc)>,
    config: Res<crate::config::GameConfig>,
) {
    if let Some(player_transform) = player_query.iter().next() {
        let px = player_transform.translation.x;
        let py = player_transform.translation.y;

        let player_chunk_x = (px / config.chunk_size).floor() as i32;
        let player_chunk_y = (py / config.chunk_size).floor() as i32;

        let mut required_chunks = HashSet::new();

        for x in -config.chunk_load_distance..=config.chunk_load_distance {
            for y in -config.chunk_load_distance..=config.chunk_load_distance {
                required_chunks.insert((player_chunk_x + x, player_chunk_y + y));
            }
        }

        // Spawn new chunks
        for chunk_pos in required_chunks.iter() {
            if !loaded_chunks.active_chunks.contains(chunk_pos) {
                spawn_chunk(&mut commands, *chunk_pos, &config);
                loaded_chunks.active_chunks.insert(*chunk_pos);
            }
        }

        // Despawn old chunks
        for (entity, chunk) in existing_chunks.iter_mut() {
            let pos = (chunk.x, chunk.y);
            if !required_chunks.contains(&pos) {
                commands.entity(entity).despawn();
                loaded_chunks.active_chunks.remove(&pos);
            }
        }
    }
}

fn spawn_chunk(commands: &mut Commands, (chunk_x, chunk_y): (i32, i32), config: &crate::config::GameConfig) {
    let noise = OpenSimplex::new(42); // Fixed seed for now
    
    // Spawn a parent entity for the chunk so we can despawn it easily
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        ChunkDesc { x: chunk_x, y: chunk_y }
    )).with_children(|parent| {
        let chunk_base_x = chunk_x as f32 * config.chunk_size;
        let chunk_base_y = chunk_y as f32 * config.chunk_size;

        // Try placing asteroids at grid points within the chunk
        let divisions = config.asteroid_grid_divisions;
        let step = config.chunk_size / divisions as f32;

        for dx in 0..divisions {
            for dy in 0..divisions {
                let cell_base_x = chunk_base_x + (dx as f32 * step);
                let cell_base_y = chunk_base_y + (dy as f32 * step);

                // Use noise to determine existence and jitter
                let noise_val = noise.get([cell_base_x as f64 * 0.001, cell_base_y as f64 * 0.001]);
                let jitter_x = noise.get([cell_base_x as f64 * 0.005, cell_base_y as f64 * 0.005]) as f32 * step * 0.8;
                let jitter_y = noise.get([cell_base_y as f64 * 0.005, cell_base_x as f64 * 0.005]) as f32 * step * 0.8;

                let cell_x = cell_base_x + jitter_x;
                let cell_y = cell_base_y + jitter_y;

                // Lower threshold to make them less patchy
                if noise_val > config.asteroid_density_threshold {
                    // Use a different noise scale for size so it's not strictly tied to density
                    let size_noise = noise.get([cell_x as f64 * 0.003, cell_y as f64 * 0.003]);
                    let radius = 30.0 + ((size_noise + 1.0) as f32 * 60.0); // 30 to 150
                    let mass = radius * 1.5; // Slightly denser planets

                    let color_val = 0.4 + (size_noise as f32 * 0.2); // Procedural color variation

                    let mut ent_cmd = parent.spawn((
                        Sprite {
                            color: Color::srgb(color_val, color_val * 0.9, color_val * 0.8), // Rocky tint
                            custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                            ..default()
                        },
                        Transform::from_xyz(cell_x, cell_y, -1.0),
                        Asteroid { radius },
                        crate::physics::GravitySource {
                            mass,
                            radius: radius * 6.0, // Area of effect proportional to size
                        },
                        RigidBody::Fixed,
                        Collider::ball(radius),
                    ));

                    let asteroid_id = ent_cmd.id();

                    ent_cmd.with_children(|aster| {
                        // Spawn 0 to 4 resource nodes on this asteroid
                        let node_count = (noise.get([cell_x as f64 * -0.007, cell_y as f64 * 0.005]) * 2.0 + 2.0) as i32;
                        for i in 0..node_count.max(0) {
                            let angle = noise.get([cell_y as f64 * i as f64, cell_x as f64]) as f32 * std::f32::consts::PI;
                            let raw_depth = noise.get([cell_x as f64, cell_y as f64 * i as f64]);
                            
                            // Depth 1, 2, or 3
                            let depth = if raw_depth > 0.3 { 3 } else if raw_depth > -0.3 { 2 } else { 1 };
                            
                            // Visual properties based on depth
                            let node_r = 6.0 - (depth as f32 * 1.0); // Smaller as they get deeper
                            let depth_factor = depth as f32;
                            let alpha = 1.0 - (depth_factor * 0.25); // Dimmer as they get deeper
                            
                            // Position on the surface (push deeper ones towards center)
                            let depth_offset = depth_factor * 8.0; // Push inwards by 8 units per depth level
                            let actual_radius = radius - depth_offset;
                            let local_x = angle.cos() * actual_radius;
                            let local_y = angle.sin() * actual_radius;
                            
                            aster.spawn((
                                Transform::from_xyz(local_x, local_y, 0.5), // Slightly in front of asteroid
                                Visibility::Hidden, // Hidden until scanned
                                crate::environment::MaterialNode {
                                    parent_asteroid: asteroid_id,
                                    angle,
                                    depth,
                                    is_revealed: false,
                                    resource_value: depth as u32 * 10,
                                },
                            )).with_children(|node_visual| {
                                // Add a visual child so we can just toggle the parent's visibility to hide/show
                                node_visual.spawn((
                                    Sprite {
                                        color: Color::srgba(0.2, 0.8, 1.0, alpha), // Cool glowing blue for resources, dimming with depth
                                        custom_size: Some(Vec2::new(node_r * 2.0, node_r * 2.0)),
                                        ..default()
                                    },
                                    Transform::default(),
                                    crate::environment::NodeVisual,
                                ));
                            });
                        }
                        
                        // Spawn a shop if the asteroid is large enough
                        if radius > 115.0 {
                            let shop_angle = noise.get([cell_x as f64 * -0.01, cell_y as f64 * 0.01]) as f32 * std::f32::consts::PI;
                            let local_x = shop_angle.cos() * radius;
                            let local_y = shop_angle.sin() * radius;
                            
                            aster.spawn((
                                Sprite {
                                    color: Color::srgb(0.8, 0.2, 0.8), // Purple shop vendor
                                    custom_size: Some(Vec2::new(24.0, 24.0)),
                                    ..default()
                                },
                                Transform::from_xyz(local_x, local_y, 0.8).with_rotation(Quat::from_rotation_z(shop_angle - std::f32::consts::FRAC_PI_2)),
                                crate::environment::Shop {
                                    parent_asteroid: asteroid_id,
                                    angle: shop_angle,
                                },
                            ));
                        }
                    });
                }
            }
        }
    });
}
