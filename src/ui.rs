//! `ui` - HUD-System.
//!
//!  * Unten links: FPS, Munition, aktiver Waffenname
//!  * Oben rechts: Tastatur-Belegungen
//!  * Kein 2D-Waffen-Ersatz mehr: echte 3D-GLB-Waffen hängen an der Kamera
//!    in `weapon.rs`.

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudState>()
            .init_resource::<FpsTracker>()
            .add_systems(Startup, setup_hud)
            .add_systems(Update, (update_fps, update_ammo_text, update_weapon_name_text));
    }
}

#[derive(Resource, Debug, Default)]
pub struct HudState {
    pub active_weapon_name: String,
    pub current_ammo: u32,
    pub max_ammo: u32,
    pub reserve: u32,
}

#[derive(Resource, Debug)]
pub struct FpsTracker {
    pub current_fps: f32,
    pub sample_accum: f32,
    pub sample_count: u32,
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self {
            current_fps: 0.0,
            sample_accum: 0.0,
            sample_count: 0,
        }
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct AmmoText;

#[derive(Component)]
struct WeaponNameText;

#[derive(Component)]
struct WeaponSilhouette;

#[derive(Component)]
struct ControlsPanel;

const CONTROL_ROWS: &[(&str, &str)] = &[
    ("WASD", "Bewegen"),
    ("Pfeiltasten", "Umsehen"),
    ("Space", "Springen"),
    ("Shift", "Sprinten"),
    ("Ctrl", "Ducken"),
    ("Shift+Ctrl", "Slide"),
    ("1 / 2 / 3", "Waffe wechseln"),
    ("Linksklick", "Schiessen / halten"),
    ("Rechtsklick", "Revolver-Fanning"),
    ("R", "Nachladen / Cock-Cycle"),
];

fn setup_hud(mut commands: Commands) {
    let font: Handle<Font> = Default::default();

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            background_color: Color::rgba(0.05, 0.04, 0.03, 0.78).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "FPS: --",
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::rgb(0.95, 0.85, 0.55),
                    },
                ),
                FpsText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "Ammo: 0 / 0",
                    TextStyle {
                        font: font.clone(),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                ),
                AmmoText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "Weapon: ---",
                    TextStyle {
                        font: font.clone(),
                        font_size: 20.0,
                        color: Color::rgb(0.85, 0.75, 0.55),
                    },
                ),
                WeaponNameText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "[ R ]",
                    TextStyle {
                        font: font.clone(),
                        font_size: 22.0,
                        color: Color::rgb(0.95, 0.90, 0.75),
                    },
                ),
                WeaponSilhouette,
            ));
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                background_color: Color::rgba(0.05, 0.04, 0.03, 0.78).into(),
                ..default()
            },
            ControlsPanel,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Steuerung / Controls",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::rgb(0.95, 0.85, 0.55),
                },
            ));
            for (key, desc) in CONTROL_ROWS {
                parent.spawn(TextBundle::from_section(
                    format!("{:14}  {}", key, desc),
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                ));
            }
        });
}

fn update_fps(
    time: Res<Time>,
    mut tracker: ResMut<FpsTracker>,
    mut q: Query<&mut Text, With<FpsText>>,
) {
    let dt = time.delta_seconds();
    if dt > 0.0 {
        tracker.sample_accum += dt;
        tracker.sample_count += 1;
    }
    if tracker.sample_accum > 0.5 {
        tracker.current_fps = tracker.sample_count as f32 / tracker.sample_accum;
        tracker.sample_accum = 0.0;
        tracker.sample_count = 0;
    }
    for mut t in &mut q {
        t.sections[0].value = format!("FPS: {:.0}", tracker.current_fps);
    }
}

fn update_ammo_text(
    mut q: Query<&mut Text, With<AmmoText>>,
    weapons: Query<&crate::weapon::Weapon>,
    state: Res<crate::weapon::WeaponState>,
) {
    for w in &weapons {
        if w.weapon_type == state.active {
            for mut t in &mut q {
                t.sections[0].value = format!(
                    "Ammo: {} / {}   Reserve: {}",
                    w.current_ammo, w.max_ammo, w.reserve
                );
            }
            return;
        }
    }
}

fn update_weapon_name_text(
    state: Res<crate::weapon::WeaponState>,
    mut q: Query<&mut Text, With<WeaponNameText>>,
    mut q_silh: Query<&mut Text, (With<WeaponSilhouette>, Without<WeaponNameText>)>,
) {
    for mut t in &mut q {
        t.sections[0].value = format!("Weapon: {}", state.active.display());
    }
    for mut t in &mut q_silh {
        t.sections[0].value = ascii_silhouette(state.active.display());
    }
}

fn ascii_silhouette(name: &str) -> String {
    match name {
        "Revolver" => "[ R ]  (1)  single-action revolver".to_string(),
        "Winchester" => "[ W ]  (2)  lever-action rifle".to_string(),
        "Shotgun" => "[ S ]  (3)  double-barrel shotgun".to_string(),
        _ => String::new(),
    }
}
