use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::AppState::InGame), setup_player)
           .add_systems(Update, player_movement.run_if(in_state(crate::AppState::InGame)))
           .add_systems(Update, camera_follow_player.run_if(in_state(crate::AppState::InGame)))
           .add_systems(Update, handle_landing.run_if(in_state(crate::AppState::InGame)));
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum PlayerState {
    #[default]
    Flying,
    Landed { planet: Entity, angle: f32, distance: f32 },
}

#[derive(Component)]
pub struct Scanner {
    pub radius: f32,
}

#[derive(Component)]
pub struct Drill {
    pub max_depth: usize,
}

#[derive(Component)]
pub struct Player {
    pub state: PlayerState,
    pub charge: f32,
    pub inventory: u32,
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Player {
            state: PlayerState::Flying,
            charge: 0.0,
            inventory: 0,
        },
        Scanner {
            radius: 400.0, // Default scan radius
        },
        Drill {
            max_depth: 1, // Can only mine surface nodes initially
        },
        Sprite {
            color: Color::srgb(1.0, 0.5, 0.0), // Default orange suit placeholder
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Dynamic,
        Collider::ball(12.0),
        GravityScale(0.0),
        Velocity::zero(),
        ExternalImpulse::default(),
        LockedAxes::ROTATION_LOCKED, // Rotation feels better if fully controlled manually
        Damping {
            linear_damping: 0.2, // Small damping to prevent literal infinite drifting speeds over time
            angular_damping: 5.0,
        },
        ActiveEvents::COLLISION_EVENTS, // Needed to detect landing
    ));
}

pub fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut ExternalImpulse, &mut Player, &mut Damping, &mut Velocity)>,
    planet_query: Query<(Entity, &Transform, &crate::world::Asteroid), Without<Player>>,
    config: Res<crate::config::GameConfig>,
) {
    let dt = time.delta_secs();
    
    for (mut transform, mut impulse, mut player, mut damping, mut velocity) in query.iter_mut() {
        let state_clone = player.state.clone();
        match state_clone {
            PlayerState::Flying => {
                damping.linear_damping = 0.2;
                damping.angular_damping = 5.0;

                let mut rotation_dir = 0.0;
                
                if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                    rotation_dir += 1.0;
                }
                if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
                    rotation_dir -= 1.0;
                }

                transform.rotate_z(rotation_dir * config.player_rotation_speed * dt);

                if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
                    let forward = transform.up().truncate();
                    impulse.impulse += forward * config.player_thrust_power * dt;
                }
            }
            PlayerState::Landed { mut planet, mut angle, mut distance } => {
                damping.linear_damping = 2.0; 
                damping.angular_damping = 5.0;

                // Stop any accumulated physics forces so we stay hard-locked
                velocity.linvel = Vec2::ZERO;
                impulse.impulse = Vec2::ZERO;

                let mut took_off = false;
                let mut current_normal = Vec2::Y;

                if let Ok((_, planet_transform, planet_asteroid)) = planet_query.get(planet) {
                    let mut move_dir = 0.0;
                    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                        move_dir += 1.0;
                    }
                    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
                        move_dir -= 1.0;
                    }

                    angle += move_dir * config.player_rotation_speed * 0.75 * dt;

                    let mut a_pos = planet_transform.translation.truncate();
                    current_normal = Vec2::new(angle.cos(), angle.sin());
                    let mut new_pos = a_pos + current_normal * distance;
                    let current_threshold = planet_asteroid.radius + 12.0;

                    // Dynamic Transition Check: Are we dipping inside another planet's bounding circle?
                    for (other_entity, other_transform, other_asteroid) in planet_query.iter() {
                        if other_entity == planet { continue; }
                        
                        let other_pos = other_transform.translation.truncate();
                        let dist_to_other = (new_pos - other_pos).length();
                        let threshold = other_asteroid.radius + 12.0; // Player radius is 12.0
                        
                        // To prevent oscillating between two intersecting planets, 
                        // we only swap if we are strictly *deeper* inside the new planet's radius.
                        if dist_to_other < threshold {
                            let current_penetration = current_threshold - distance;
                            let new_penetration = threshold - dist_to_other;
                            
                            if new_penetration > current_penetration + 0.1 {
                                // Swap anchor to the new intersecting planet!
                                planet = other_entity;
                                distance = threshold;
                                let diff = new_pos - other_pos;
                                angle = diff.y.atan2(diff.x);
                                
                                // Re-evaluate normal and position exactly on the new planet boundary
                                a_pos = other_pos;
                                current_normal = Vec2::new(angle.cos(), angle.sin());
                                new_pos = a_pos + current_normal * distance;
                                break;
                            }
                        }
                    }

                    transform.translation = new_pos.extend(transform.translation.z);

                    let target_angle = angle - std::f32::consts::FRAC_PI_2;
                    transform.rotation = Quat::from_rotation_z(target_angle);
                }

                if keyboard_input.pressed(KeyCode::Space) {
                    player.charge += dt * 2.0;
                    player.charge = player.charge.min(1.0);
                } else if keyboard_input.just_released(KeyCode::Space) {
                    let launch_power = config.player_thrust_power * config.player_launch_multiplier * player.charge;
                    impulse.impulse += current_normal * launch_power;
                    player.charge = 0.0;
                    player.state = PlayerState::Flying;
                    took_off = true;
                }

                if !took_off {
                    player.state = PlayerState::Landed { planet, angle, distance };
                }
            }
        }
    }
}

fn handle_landing(
    mut collision_events: MessageReader<CollisionEvent>,
    mut player_query: Query<(Entity, &mut Player, &Transform, &mut Velocity)>,
    asteroid_query: Query<(&Transform, &crate::world::Asteroid)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let ent1 = *e1;
            let ent2 = *e2;
            
            let (player_ent, asteroid_ent) = if player_query.contains(ent1) && asteroid_query.contains(ent2) {
                (ent1, ent2)
            } else if player_query.contains(ent2) && asteroid_query.contains(ent1) {
                (ent2, ent1)
            } else {
                continue;
            };

            if let Ok((_, mut player, p_transform, mut velocity)) = player_query.get_mut(player_ent) {
                if let Ok((a_transform, asteroid)) = asteroid_query.get(asteroid_ent) {
                    if player.state == PlayerState::Flying {
                        let p_pos = p_transform.translation.truncate();
                        let a_pos = a_transform.translation.truncate();
                        let diff = p_pos - a_pos;
                        
                        // Force explicit distance to ensure smooth dynamic transitions
                        let distance = asteroid.radius + 12.0; 
                        let angle = diff.y.atan2(diff.x);

                        player.state = PlayerState::Landed { planet: asteroid_ent, angle, distance };
                        velocity.linvel = Vec2::ZERO;
                    }
                }
            }
        }
    }
}

fn camera_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    for player_transform in player_query.iter() {
        for mut camera_transform in camera_query.iter_mut() {
            let target = player_transform.translation;
            let current = camera_transform.translation;
            camera_transform.translation = current.lerp(target, 0.1);
        }
    }
}
