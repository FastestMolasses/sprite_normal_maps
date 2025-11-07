use bevy::prelude::*;

/// Marker component for UI instructions panel
#[derive(Component)]
pub struct InstructionsPanel;

/// Marker component for status display
#[derive(Component)]
pub struct StatusPanel;

/// Component to track which scene the UI belongs to
#[derive(Component)]
pub struct SceneUi {
    pub scene_type: SceneType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneType {
    TextureMapped,
    Procedural,
}

/// Spawn the UI for the texture mapped scene
pub fn spawn_texture_mapped_ui(commands: &mut Commands) {
    let text_font = TextFont {
        font_size: 14.0,
        ..default()
    };

    // Instructions panel
    commands
        .spawn((
            Text::new("=== TEXTURE MAPPED SCENE ===\n\n"),
            text_font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            InstructionsPanel,
            SceneUi {
                scene_type: SceneType::TextureMapped,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextSpan::new("Controls:\n"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));
            parent.spawn((
                TextSpan::new("  WASD - Move light\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  U/J - Light height\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  I/K - Light intensity\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  O/L - Ambient intensity\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  [/] - Light radius\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  +/- - Light falloff\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  P/; - Position scale\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  V - Cycle debug modes\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  Space - Display info\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  F1 - Switch to Procedural Scene\n"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 1.0, 0.5)),
            ));
        });
    
    // Status panel (top right)
    commands
        .spawn((
            Text::new("Debug Mode: Normal Lighting"),
            text_font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StatusPanel,
            SceneUi {
                scene_type: SceneType::TextureMapped,
            },
        ));
}

/// Spawn the UI for the procedural scene
pub fn spawn_procedural_ui(commands: &mut Commands, render_mode: &str) {
    let text_font = TextFont {
        font_size: 14.0,
        ..default()
    };

    // Instructions panel
    commands
        .spawn((
            Text::new("=== PROCEDURAL SCENE ===\n\n"),
            text_font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            InstructionsPanel,
            SceneUi {
                scene_type: SceneType::Procedural,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextSpan::new(format!("Mode: {}\n\n", render_mode)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 1.0, 1.0)),
            ));
            parent.spawn((
                TextSpan::new("Controls:\n"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));
            parent.spawn((
                TextSpan::new("  WASD - Move light\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  U/J - Light height\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  I/K - Light intensity\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  O/L - Ambient intensity\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  [/] - Light radius\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  +/- - Light falloff\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  V - Cycle debug modes\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  Space - Display info\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("\nRotation:\n"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.5)),
            ));
            parent.spawn((
                TextSpan::new("  Q/E - Rotate Y-axis\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  R/F - Rotate X-axis\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  T/Y - Rotate Z-axis\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("  X - Reset rotation\n"),
                text_font.clone(),
            ));
            parent.spawn((
                TextSpan::new("\n  M - Toggle CPU/GPU rendering\n"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.5, 1.0)),
            ));
            parent.spawn((
                TextSpan::new("  F1 - Switch to Texture Mapped Scene\n"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 1.0, 0.5)),
            ));
        });
    
    // Status panel (top right)
    commands
        .spawn((
            Text::new("Debug Mode: Normal Lighting"),
            text_font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StatusPanel,
            SceneUi {
                scene_type: SceneType::Procedural,
            },
        ));
}

/// Update UI when render mode changes
pub fn update_procedural_ui_mode(
    mode_text: &str,
    ui_query: Query<Entity, (With<InstructionsPanel>, With<SceneUi>)>,
    mut commands: Commands,
) {
    // Despawn old UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Spawn new UI with updated mode
    spawn_procedural_ui(&mut commands, mode_text);
}

/// Despawn all UI elements for a specific scene
pub fn despawn_scene_ui(
    mut commands: Commands,
    ui_query: Query<(Entity, &SceneUi)>,
    scene_type: SceneType,
) {
    for (entity, scene_ui) in ui_query.iter() {
        if scene_ui.scene_type == scene_type {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawn all UI elements
pub fn despawn_all_ui(
    mut commands: Commands,
    ui_query: Query<Entity, Or<(With<InstructionsPanel>, With<StatusPanel>)>>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn();
    }
}
