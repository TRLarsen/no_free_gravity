use bevy::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_environment_ui);
        app.add_systems(Update, (
            scanning_system,
            mining_system,
            scanner_visual_system,
            shop_system,
            update_resource_ui_system,
        ).run_if(in_state(crate::AppState::InGame)));
    }
}

// Effect struct for the radar ping
#[derive(Component)]
pub struct ScannerBubble {
    pub timer: Timer,
    pub max_radius: f32,
}

#[derive(Component)]
pub struct Shop {
    pub parent_asteroid: Entity,
    pub angle: f32, // Location on asteroid
}

#[derive(Component)]
pub struct ShopPromptText;

#[derive(Component)]
pub struct ShopMenuText;

#[derive(Component)]
pub struct ResourceTrackerUI;

fn setup_environment_ui(mut commands: Commands) {
    // Prompt Text "Press F to open Shop"
    commands.spawn((
        Text::new("Press F to Shop"),
        TextFont { font_size: 30.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(45.0),
            ..default()
        },
        Visibility::Hidden,
        ShopPromptText,
    ));

    // Menu Text
    commands.spawn((
        Text::new("SHOP MENU"),
        TextFont { font_size: 40.0, ..default() },
        TextColor(Color::srgb(0.8, 0.2, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.0),
            left: Val::Percent(30.0),
            ..default()
        },
        Visibility::Hidden,
        ShopMenuText,
    ));

    // Resource Tracker Text
    commands.spawn((
        Text::new("Resources: 0"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::srgb(0.2, 0.8, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        ResourceTrackerUI,
    ));
}

fn scanning_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&GlobalTransform, &crate::player::Scanner), With<crate::player::Player>>,
    mut node_query: Query<(&mut MaterialNode, &GlobalTransform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        if let Some((player_transform, scanner)) = player_query.iter().next() {
            let p_pos = player_transform.translation().truncate();

            // Spawn visual radar ping as a circle
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(1.0))),
                MeshMaterial2d(materials.add(Color::srgba(0.0, 1.0, 0.3, 0.5))), // Green radar
                Transform::from_translation(p_pos.extend(-0.5)),
                ScannerBubble {
                    timer: Timer::from_seconds(0.4, TimerMode::Once), // Fast pulse
                    max_radius: scanner.radius,
                },
            ));

            // Reveal valid nodes
            for (mut node, node_transform, mut visibility) in node_query.iter_mut() {
                if !node.is_revealed {
                    let n_pos = node_transform.translation().truncate();
                    if p_pos.distance(n_pos) <= scanner.radius {
                        node.is_revealed = true;
                        *visibility = Visibility::Inherited; // Make it and its visual children visible
                    }
                }
            }
        }
    }
}

fn scanner_visual_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScannerBubble, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta();
    for (entity, mut bubble, mut transform, material_handle) in query.iter_mut() {
        bubble.timer.tick(dt);
        let fraction = bubble.timer.fraction();
        
        if fraction >= 1.0 {
            commands.entity(entity).despawn();
        } else {
            // Scale the circle mesh up to the max radius
            let current_size = bubble.max_radius * fraction;
            transform.scale = Vec3::splat(current_size);
            
            // Fade out
            if let Some(material) = materials.get_mut(material_handle) {
                let alpha = 0.5 * (1.0 - fraction.powi(2)); // start at 0.5, quadratic falloff
                material.color.set_alpha(alpha);
            }
        }
    }
}

fn mining_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut crate::player::Player, &crate::player::Drill)>,
    node_query: Query<(Entity, &MaterialNode)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        if let Some((mut player, drill)) = player_query.iter_mut().next() {
            if let crate::player::PlayerState::Landed { planet, angle, .. } = player.state {
                // Find a revealed node on this planet at the same approximate angle
                for (node_entity, node) in node_query.iter() {
                    if node.is_revealed && node.parent_asteroid == planet {
                        // Check angle difference
                        let angle_diff = (node.angle - angle).abs();
                        // Handle wraparound for angle (mod 2pi doesn't work for f32 directly in same way, so manual check)
                        let mut dist = angle_diff % (std::f32::consts::PI * 2.0);
                        if dist > std::f32::consts::PI {
                            dist = std::f32::consts::PI * 2.0 - dist;
                        }
                        
                        if dist < 0.2 { // Within roughly ~12 degrees
                            if drill.max_depth >= node.depth {
                                player.inventory += node.resource_value;
                                println!("Mined {} resources! Total: {}", node.resource_value, player.inventory);
                                commands.entity(node_entity).despawn();
                            } else {
                                println!("Drill level too low! Needs level {}, but you have level {}", node.depth, drill.max_depth);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn shop_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut crate::player::Player, &mut crate::player::Drill, &mut crate::player::Scanner)>,
    shop_query: Query<&Shop>,
    mut prompt_query: Query<&mut Visibility, (With<ShopPromptText>, Without<ShopMenuText>)>,
    mut menu_query: Query<(&mut Visibility, &mut Text), (With<ShopMenuText>, Without<ShopPromptText>)>,
    mut is_shopping: Local<bool>,
    mut config: ResMut<crate::config::GameConfig>,
) {
    let mut near_shop = false;
    
    if let Some((mut player, mut drill, mut scanner)) = player_query.iter_mut().next() {
        if let crate::player::PlayerState::Landed { planet, angle, .. } = player.state {
            for shop in shop_query.iter() {
                if shop.parent_asteroid == planet {
                    let angle_diff = (shop.angle - angle).abs();
                    let mut dist = angle_diff % (std::f32::consts::PI * 2.0);
                    if dist > std::f32::consts::PI {
                        dist = std::f32::consts::PI * 2.0 - dist;
                    }
                    if dist < 0.2 { // ~12 degrees
                        near_shop = true;
                    }
                }
            }
        }
        
        if !near_shop {
            *is_shopping = false;
        } else if keyboard_input.just_pressed(KeyCode::KeyF) {
            *is_shopping = !*is_shopping;
        }

        // Handle shopping
        if *is_shopping {
            if keyboard_input.just_pressed(KeyCode::Digit1) && player.inventory >= 50 {
                player.inventory -= 50;
                drill.max_depth += 1;
            }
            if keyboard_input.just_pressed(KeyCode::Digit2) && player.inventory >= 50 {
                player.inventory -= 50;
                scanner.radius += 200.0;
            }
            if keyboard_input.just_pressed(KeyCode::Digit3) && player.inventory >= 50 {
                player.inventory -= 50;
                config.player_launch_multiplier += 1.0;
            }
            if keyboard_input.just_pressed(KeyCode::Escape) {
                *is_shopping = false;
            }
        }
        
        // Update Visibilities
        if let Some(mut vis) = prompt_query.iter_mut().next() {
            *vis = if near_shop && !*is_shopping { Visibility::Inherited } else { Visibility::Hidden };
        }
        
        if let Some((mut vis, mut text)) = menu_query.iter_mut().next() {
            *vis = if *is_shopping { Visibility::Inherited } else { Visibility::Hidden };
            // Update Text string
            text.0 = format!(
                "SHOP MENU (Resources: {})\n[1] Upgrade Drill Depth (50) - Current: {}\n[2] Upgrade Scanner (50) - Current Rad: {}\n[3] Upgrade Boost (50) - Current Mul: {}\n[ESC] Close",
                player.inventory, drill.max_depth, scanner.radius, config.player_launch_multiplier
            );
        }
    }
}

fn update_resource_ui_system(
    player_query: Query<&crate::player::Player, Changed<crate::player::Player>>,
    mut ui_query: Query<&mut Text, With<ResourceTrackerUI>>,
) {
    if let Some(player) = player_query.iter().next() {
        if let Some(mut text) = ui_query.iter_mut().next() {
            text.0 = format!("Resources: {}", player.inventory);
        }
    }
}

// Data model for resource nodes spawned on asteroids
#[derive(Component)]
pub struct MaterialNode {
    pub parent_asteroid: Entity,
    pub angle: f32, // Where it is on the asteroid's surface circumference
    pub depth: usize, // Required drill level to extract (e.g. 1, 2, or 3)
    pub is_revealed: bool,
    pub resource_value: u32,
}

// To visually represent when a node is revealed
#[derive(Component)]
pub struct NodeVisual;
