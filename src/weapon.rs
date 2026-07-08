//! `weapon` - 3D-Waffen, Ballistik, Schuss- und Tracer-Effekte.
//!
//! Scope dieser Datei nach User-Wunsch:
//!   * Welt, Player und Level bleiben aus unserem Projekt.
//!   * Waffen-Viewmodels, Schuss-State-Machine, Spread, Projektile und
//!     Tracer orientieren sich am Referenzprojekt
//!     `wild-west-paintball-fps-master`.
//!   * Target-Bevy-Version bleibt 0.14.2; Referenz ist Bevy 0.19, daher
//!     ist dies ein Port, kein Copy/Paste.

use crate::player::Player;
use crate::world::Cover;
use bevy::gltf::GltfAssetLabel;
use bevy::math::primitives::{Cuboid, Plane3d, Sphere};
use bevy::prelude::*;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponState>()
            .add_systems(Startup, setup_weapon_model.after(crate::player::spawn_player))
            .add_systems(
                Update,
                (
                    weapon_switch_input,
                    weapon_state_tick,
                    shoot_input,
                    update_weapon_viewmodel_animation,
                    update_paintballs,
                    update_trail_segments,
                    update_hud_weapon,
                )
                    .chain()
                    .in_set(WeaponMutationSet),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct WeaponMutationSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Revolver,
    Winchester,
    Shotgun,
}

impl WeaponType {
    pub fn max_ammo(self) -> u32 {
        match self {
            WeaponType::Revolver => 6,
            WeaponType::Winchester => 12,
            WeaponType::Shotgun => 2,
        }
    }

    pub fn reserve(self) -> u32 {
        match self {
            WeaponType::Revolver => 36,
            WeaponType::Winchester => 60,
            WeaponType::Shotgun => 24,
        }
    }

    pub fn display(self) -> &'static str {
        match self {
            WeaponType::Revolver => "Revolver",
            WeaponType::Winchester => "Winchester",
            WeaponType::Shotgun => "Shotgun",
        }
    }

    pub fn long_name(self) -> &'static str {
        match self {
            WeaponType::Revolver => "Single-Action Revolver",
            WeaponType::Winchester => "Lever-Action Rifle",
            WeaponType::Shotgun => "Double-Barrel Shotgun",
        }
    }

    pub fn cycle_time(self) -> f32 {
        match self {
            WeaponType::Revolver => 0.0,
            WeaponType::Winchester => 0.42,
            WeaponType::Shotgun => 0.82,
        }
    }

    pub fn reload_time(self) -> f32 {
        match self {
            WeaponType::Revolver => 1.9,
            WeaponType::Winchester => 1.4,
            WeaponType::Shotgun => 1.65,
        }
    }

    pub fn cock_time(self) -> f32 {
        match self {
            WeaponType::Revolver => 0.22,
            _ => 0.0,
        }
    }

    pub fn lever_time(self) -> f32 {
        match self {
            WeaponType::Winchester => 0.42,
            _ => 0.0,
        }
    }

    pub fn muzzle_speed(self) -> f32 {
        match self {
            WeaponType::Revolver => 58.0,
            WeaponType::Winchester => 66.7,
            WeaponType::Shotgun => 47.6,
        }
    }

    pub fn base_spread(self) -> f32 {
        match self {
            WeaponType::Revolver => 0.006,
            WeaponType::Winchester => 0.002,
            WeaponType::Shotgun => 0.07,
        }
    }

    pub fn pellet_count(self) -> usize {
        match self {
            WeaponType::Revolver => 1,
            WeaponType::Winchester => 1,
            WeaponType::Shotgun => 12,
        }
    }

    fn default_phase(self) -> WeaponPhase {
        match self {
            WeaponType::Revolver => WeaponPhase::Uncocked,
            _ => WeaponPhase::Ready,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeaponPhase {
    Ready,
    Uncocked,
    Cocking,
    LeverOut,
    LeverIn,
    BreakingOpen,
    Reloading,
    DryFire,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub current_ammo: u32,
    pub max_ammo: u32,
    pub reserve: u32,
}

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        Self {
            weapon_type,
            current_ammo: weapon_type.max_ammo(),
            max_ammo: weapon_type.max_ammo(),
            reserve: weapon_type.reserve(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct WeaponState {
    pub active: WeaponType,
    phase: WeaponPhase,
    timer: f32,
    pub muzzle_flash_timer: f32,
    pub shot_sequence: u64,
    pub spread_mult: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            active: WeaponType::Revolver,
            phase: WeaponType::Revolver.default_phase(),
            timer: 0.0,
            muzzle_flash_timer: 0.0,
            shot_sequence: 0,
            spread_mult: 1.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct Paintball {
    pub velocity: Vec3,
    pub gravity_drop: f32,
    pub color: Color,
    pub lifetime: f32,
}

#[derive(Component, Debug)]
pub struct PaintballTrail {
    pub color: Color,
    pub age: f32,
    pub lifetime: f32,
}

#[derive(Component, Debug)]
pub struct SplatterDecal {
    pub lifetime: f32,
}

#[derive(Component, Debug)]
pub struct TracerBeam {
    pub age: f32,
    pub max_age: f32,
}

#[derive(Component, Debug)]
pub struct WeaponModel {
    pub weapon_type: WeaponType,
}

const PROJECTILE_GRAVITY: f32 = -9.81;
const PROJECTILE_DRAG: f32 = 0.18;
const PAINTBALL_LIFETIME: f32 = 3.0;
const PAINTBALL_RADIUS: f32 = 0.05;
const TRAIL_SEGMENT_LIFETIME: f32 = 0.55;
const TRAIL_SEGMENT_THICKNESS: f32 = 0.045;
const TRACER_LENGTH: f32 = 8.0;
const TRACER_THICKNESS: f32 = 0.03;
const TRACER_MAX_AGE: f32 = 0.5;

struct WeaponVisuals {
    pos: Vec3,
    rot: Quat,
    scale: Vec3,
    scene_path: &'static str,
}

fn visuals_for(wt: WeaponType) -> WeaponVisuals {
    // Empirisch aus Screenshots 10-14.
    //
    // Winchester/Shotgun: Roll -90° um Z, Skala 0.32, Position rechts.
    // User bestaetigt in Screenshot 13-14: passt.
    //
    // Revolver: Das GLB hat eine interne 180°-X-Drehung in node[1].
    // Effektive Modellachsen sind also: +X = Lauf-Spitze, +Y = Kimme
    // (nach internem 180°-Flip). Wir wollen: +X (Lauf) -> -Z (weg vom
    // Spieler), +Y (Kimme) -> +Y (oben). Yaw -90° um Y macht genau das.
    // Kein Pitch, kein Roll noetig.
    match wt {
        WeaponType::Revolver => WeaponVisuals {
            pos: Vec3::new(0.22, -0.15, -0.45),
            rot: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(0.7),
            scene_path: "models/weapons/revolver_paint.glb",
        },
        WeaponType::Winchester => WeaponVisuals {
            pos: Vec3::new(0.24, -0.15, -0.55),
            rot: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(0.32),
            scene_path: "models/weapons/lever_rifle_hopper.glb",
        },
        WeaponType::Shotgun => WeaponVisuals {
            pos: Vec3::new(0.24, -0.15, -0.55),
            rot: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(0.32),
            scene_path: "models/weapons/double_barrel_paint.glb",
        },
    }
}

fn setup_weapon_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<Camera3d>>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        std::fs::write("/tmp/wildwest-weapon.log", "WARNUNG: keine Kamera-Entity\n").ok();
        return;
    };

    let mut log = format!("Kamera-Entity = {:?}\n", camera_entity);
    commands.entity(camera_entity).with_children(|cam| {
        for wt in [WeaponType::Revolver, WeaponType::Winchester, WeaponType::Shotgun] {
            let vis = visuals_for(wt);
            let initial_visibility = if wt == WeaponType::Revolver {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            log.push_str(&format!("lade {} -> {}\n", wt.display(), vis.scene_path));
            cam.spawn((
                SceneBundle {
                    scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset(vis.scene_path)),
                    transform: Transform {
                        translation: vis.pos,
                        rotation: vis.rot,
                        scale: vis.scale,
                    },
                    visibility: initial_visibility,
                    ..default()
                },
                WeaponModel { weapon_type: wt },
                Weapon::new(wt),
                Name::new(wt.long_name()),
            ));
        }
    });
    std::fs::write("/tmp/wildwest-weapon.log", log).ok();
}

fn weapon_switch_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<WeaponState>,
    mut models: Query<(&mut Visibility, &WeaponModel)>,
) {
    let new_active = if keys.just_pressed(KeyCode::Digit1) {
        Some(WeaponType::Revolver)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(WeaponType::Winchester)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(WeaponType::Shotgun)
    } else {
        None
    };

    if let Some(wt) = new_active {
        state.active = wt;
        state.phase = wt.default_phase();
        state.timer = 0.0;
        state.muzzle_flash_timer = 0.0;
        for (mut vis, model) in &mut models {
            *vis = if model.weapon_type == wt {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn weapon_state_tick(
    time: Res<Time>,
    mut state: ResMut<WeaponState>,
    mut weapons: Query<&mut Weapon>,
) {
    let dt = time.delta_seconds();
    let was_timed = state.timer > 0.0;
    if state.timer > 0.0 {
        state.timer = (state.timer - dt).max(0.0);
    }
    if state.muzzle_flash_timer > 0.0 {
        state.muzzle_flash_timer = (state.muzzle_flash_timer - dt).max(0.0);
    }
    if !was_timed || state.timer > 0.0 {
        return;
    }

    match state.phase {
        WeaponPhase::Cocking => state.phase = WeaponPhase::Ready,
        WeaponPhase::LeverOut => {
            state.phase = WeaponPhase::LeverIn;
            state.timer = state.active.lever_time() * 0.5;
        }
        WeaponPhase::LeverIn => state.phase = WeaponPhase::Ready,
        WeaponPhase::Reloading => {
            for mut w in &mut weapons {
                if w.weapon_type == state.active {
                    let needed = w.max_ammo.saturating_sub(w.current_ammo);
                    let loaded = needed.min(w.reserve);
                    w.current_ammo += loaded;
                    w.reserve -= loaded;
                    break;
                }
            }
            state.phase = state.active.default_phase();
        }
        WeaponPhase::DryFire => {
            state.phase = if state.active == WeaponType::Revolver {
                WeaponPhase::Uncocked
            } else {
                WeaponPhase::Ready
            };
        }
        WeaponPhase::BreakingOpen | WeaponPhase::Ready | WeaponPhase::Uncocked => {}
    }
}

fn compute_spread_multiplier(
    _player: &crate::player::PlayerLook,
    velocity: &crate::player::Velocity,
    transform: &Transform,
    stance: &crate::player::PlayerStance,
) -> f32 {
    let speed_sq = velocity.linear.x * velocity.linear.x + velocity.linear.z * velocity.linear.z;
    let sprinting = speed_sq > (4.5_f32 * 4.5_f32);
    let airborne = (transform.translation.y - stance.eye_height).abs() > 0.05;
    let mut m = 1.0_f32;
    if airborne {
        m *= 1.8;
    }
    if stance.sliding {
        m *= 1.35;
    }
    if sprinting {
        m *= 1.5;
    }
    if stance.crouched {
        m *= 0.75;
    }
    m
}

fn shoot_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<WeaponState>,
    mut weapons: Query<&mut Weapon>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<
        (
            &Transform,
            &crate::player::Velocity,
            &crate::player::PlayerLook,
            &crate::player::PlayerStance,
        ),
        With<Player>,
    >,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        let mut can_reload = false;
        for w in &weapons {
            if w.weapon_type == state.active {
                can_reload = w.current_ammo < w.max_ammo && w.reserve > 0;
                break;
            }
        }
        if can_reload
            && (state.active != WeaponType::Shotgun
                || matches!(state.phase, WeaponPhase::Ready | WeaponPhase::BreakingOpen))
        {
            state.phase = WeaponPhase::Reloading;
            state.timer = state.active.reload_time();
        } else {
            match (state.active, state.phase) {
                (WeaponType::Revolver, WeaponPhase::Uncocked) => {
                    state.phase = WeaponPhase::Cocking;
                    state.timer = state.active.cock_time();
                }
                (WeaponType::Winchester, WeaponPhase::LeverOut) => {
                    state.phase = WeaponPhase::LeverIn;
                    state.timer = state.active.lever_time() * 0.5;
                }
                _ => {}
            }
        }
    }

    if !buttons.pressed(MouseButton::Left) || state.timer > 0.0 {
        return;
    }

    let wt = state.active;
    let fanning = wt == WeaponType::Revolver && buttons.pressed(MouseButton::Right);

    let mut fired = false;
    let mut dry_fire = false;
    for mut weapon in &mut weapons {
        if weapon.weapon_type != wt {
            continue;
        }
        if weapon.current_ammo == 0 {
            state.phase = WeaponPhase::DryFire;
            state.timer = 0.08;
            dry_fire = true;
            break;
        }

        match wt {
            WeaponType::Revolver => {
                if state.phase == WeaponPhase::Uncocked {
                    // Referenz: Auto-cock on any fire attempt.
                    state.phase = WeaponPhase::Ready;
                }
                if state.phase == WeaponPhase::Ready || fanning {
                    weapon.current_ammo = weapon.current_ammo.saturating_sub(1);
                    state.shot_sequence = state.shot_sequence.wrapping_add(1);
                    state.phase = if fanning {
                        WeaponPhase::Ready
                    } else {
                        WeaponPhase::Uncocked
                    };
                    state.timer = if fanning { 0.14 } else { 0.0 };
                    fired = true;
                }
            }
            WeaponType::Winchester if state.phase == WeaponPhase::Ready => {
                weapon.current_ammo = weapon.current_ammo.saturating_sub(1);
                state.shot_sequence = state.shot_sequence.wrapping_add(1);
                state.phase = WeaponPhase::LeverOut;
                state.timer = wt.lever_time() * 0.5;
                fired = true;
            }
            WeaponType::Shotgun if state.phase == WeaponPhase::Ready => {
                weapon.current_ammo = weapon.current_ammo.saturating_sub(1);
                state.shot_sequence = state.shot_sequence.wrapping_add(1);
                if weapon.current_ammo == 0 {
                    state.phase = WeaponPhase::BreakingOpen;
                    state.timer = 0.25;
                } else {
                    state.phase = WeaponPhase::Ready;
                    state.timer = wt.cycle_time();
                }
                fired = true;
            }
            _ => {}
        }
        break;
    }

    if dry_fire || !fired {
        return;
    }

    let Ok((player_tf, velocity, look, stance)) = player_query.get_single() else {
        return;
    };

    let yaw = Quat::from_rotation_y(look.yaw);
    let pitch = Quat::from_rotation_x(look.pitch);
    let forward = yaw.mul_vec3(pitch.mul_vec3(-Vec3::Z)).normalize_or_zero();
    if forward == Vec3::ZERO {
        return;
    }
    let right = yaw.mul_vec3(Vec3::X).normalize_or_zero();
    let up = Vec3::Y;
    let muzzle = player_tf.translation + forward * 0.70 + right * 0.18 + up * -0.12;

    state.muzzle_flash_timer = 0.08;
    state.spread_mult = compute_spread_multiplier(look, velocity, player_tf, stance);
    let effective_spread = wt.base_spread() * state.spread_mult;
    let dirs = sample_spread_discs(forward, wt.pellet_count(), effective_spread, state.shot_sequence);
    let paint_color = if wt == WeaponType::Shotgun {
        Color::rgb(1.0, 0.12, 0.05)
    } else {
        Color::rgb(0.05, 0.85, 1.0)
    };
    for dir in dirs {
        spawn_paintball(
            &mut commands,
            &mut meshes,
            &mut materials,
            muzzle,
            dir * wt.muzzle_speed(),
            paint_color,
        );
        // Kein separater Tracer-Beam mehr: das per-Frame-Trail-Segment
        // in `update_paintballs` zeichnet die tatsaechliche Schussbahn
        // (inkl. Schwerkraft/Bulletdrop). Der fruehere 8m-Cuboid-Strahl
        // hat eine gerade Linie ohne Bulletdrop gezeigt.
    }
}

fn update_weapon_viewmodel_animation(
    time: Res<Time>,
    state: Res<WeaponState>,
    mut q: Query<(&WeaponModel, &mut Transform)>,
) {
    let bob = (time.elapsed_seconds() * 5.0).sin() * 0.006;
    for (model, mut tf) in &mut q {
        let vis = visuals_for(model.weapon_type);
        let active = model.weapon_type == state.active;
        let mut pos = vis.pos;
        let mut rot = vis.rot;
        let mut scale = vis.scale;

        if active {
            pos.y += bob;
            let flash_t = (state.muzzle_flash_timer / 0.08).clamp(0.0, 1.0);
            pos.z += flash_t * 0.08; // recoil Richtung Kamera
            pos.y -= flash_t * 0.02;

            let timer_norm = if state.timer > 0.0 {
                (state.timer / model.weapon_type.reload_time().max(0.001)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            match state.phase {
                WeaponPhase::Cocking => {
                    rot = vis.rot * Quat::from_rotation_x(-0.25 * (1.0 - timer_norm));
                }
                WeaponPhase::LeverOut | WeaponPhase::LeverIn => {
                    let lever_t = (state.timer / (model.weapon_type.lever_time() * 0.5).max(0.001))
                        .clamp(0.0, 1.0);
                    let phase = if state.phase == WeaponPhase::LeverOut {
                        1.0 - lever_t
                    } else {
                        lever_t
                    };
                    rot = vis.rot * Quat::from_rotation_x(-0.35 * phase);
                    pos.y -= 0.04 * phase;
                }
                WeaponPhase::BreakingOpen => {
                    rot = vis.rot * Quat::from_rotation_x(0.45);
                    pos.y -= 0.06;
                }
                WeaponPhase::Reloading => {
                    let phase = (1.0 - timer_norm).sin().abs();
                    pos.y -= 0.14;
                    pos.z += 0.08;
                    rot = vis.rot
                        * Quat::from_rotation_x(0.20 + 0.10 * phase)
                        * Quat::from_rotation_z(-0.18);
                }
                WeaponPhase::DryFire => {
                    scale *= 0.995;
                }
                WeaponPhase::Ready | WeaponPhase::Uncocked => {}
            }
        }

        tf.translation = pos;
        tf.rotation = rot;
        tf.scale = scale;
    }
}

fn sample_spread_discs(forward: Vec3, n: usize, max_angle: f32, seed: u64) -> Vec<Vec3> {
    let f = forward.normalize_or_zero();
    if f == Vec3::ZERO || n == 0 {
        return Vec::new();
    }
    let right = if f.z.abs() > 0.9 {
        Vec3::X
    } else {
        Vec3::new(f.z, 0.0, -f.x).normalize_or_zero()
    };
    let up = right.cross(f).normalize_or_zero();
    let mut s = mix_seed(seed ^ ((n as u64) << 32));
    (0..n)
        .map(|_| {
            let r = next_unit(&mut s).sqrt() * max_angle;
            let theta = next_unit(&mut s) * std::f32::consts::TAU;
            (f + right * (theta.cos() * r) + up * (theta.sin() * r)).normalize_or_zero()
        })
        .collect()
}

fn mix_seed(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn next_unit(seed: &mut u64) -> f32 {
    *seed = mix_seed(*seed);
    ((*seed >> 40) as f32) / ((1u64 << 24) as f32)
}

pub fn spawn_paintball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    vel: Vec3,
    color: Color,
) {
    let mesh = meshes.add(Mesh::from(Sphere { radius: PAINTBALL_RADIUS }));
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.into(),
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(pos),
            ..default()
        },
        Paintball {
            velocity: vel,
            gravity_drop: PROJECTILE_GRAVITY,
            color,
            lifetime: PAINTBALL_LIFETIME,
        },
    ));
}

fn spawn_tracer_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    dir: Vec3,
    color: Color,
) {
    let dir3 = dir.normalize_or_zero();
    if dir3 == Vec3::ZERO {
        return;
    }
    let mid = origin + dir3 * (TRACER_LENGTH * 0.5);
    let rot = Quat::from_rotation_arc(Vec3::Z, dir3);
    let linear = color.to_linear();
    let mat = materials.add(StandardMaterial {
        base_color: Color::rgba(linear.red, linear.green, linear.blue, 0.7),
        emissive: color.into(),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(
                TRACER_THICKNESS,
                TRACER_THICKNESS,
                TRACER_LENGTH,
            ))),
            material: mat,
            transform: Transform::from_translation(mid).with_rotation(rot),
            ..default()
        },
        TracerBeam {
            age: 0.0,
            max_age: TRACER_MAX_AGE,
        },
    ));
}

fn update_paintballs(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut param_set: ParamSet<(
        Query<(Entity, &mut Transform, &mut Paintball)>,
        Query<&Cover>,
        Query<(Entity, &PaintballTrail)>,
    )>,
) {
    let dt = time.delta_seconds();
    let cover_positions: Vec<Vec3> = param_set.p1().iter().map(|c| c.position).collect();
    let mut new_trails: Vec<(Vec3, Vec3, Color)> = Vec::new();

    for (entity, mut tf, mut pb) in param_set.p0().iter_mut() {
        let prev = tf.translation;
        pb.velocity.y += pb.gravity_drop * dt;
        pb.velocity *= 1.0 - (PROJECTILE_DRAG * dt).min(0.8);
        let next = tf.translation + pb.velocity * dt;
        tf.translation = next;
        pb.lifetime -= dt;
        new_trails.push((prev, next, pb.color));

        let mut hit: Option<(Vec3, Vec3)> = None;
        for cp in &cover_positions {
            let half = Vec3::new(1.0, 0.5, 0.2);
            let min = *cp - half;
            let max = *cp + half;
            if next.x >= min.x
                && next.x <= max.x
                && next.y >= min.y
                && next.y <= max.y
                && next.z >= min.z
                && next.z <= max.z
            {
                let dx_min = (next.x - min.x).abs();
                let dx_max = (max.x - next.x).abs();
                let dy_min = (next.y - min.y).abs();
                let dy_max = (max.y - next.y).abs();
                let dz_min = (next.z - min.z).abs();
                let dz_max = (max.z - next.z).abs();
                let n = if dx_min.min(dx_max) < dy_min.min(dy_max).min(dz_min.min(dz_max)) {
                    if dx_min < dx_max { Vec3::new(-1.0, 0.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) }
                } else if dy_min.min(dy_max) < dz_min.min(dz_max) {
                    if dy_min < dy_max { Vec3::new(0.0, -1.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) }
                } else if dz_min < dz_max {
                    Vec3::new(0.0, 0.0, -1.0)
                } else {
                    Vec3::new(0.0, 0.0, 1.0)
                };
                hit = Some((next, n));
                break;
            }
        }

        if hit.is_none() && next.y < 0.1 {
            hit = Some((next, Vec3::new(0.0, 1.0, 0.0)));
        }
        if pb.lifetime <= 0.0 && hit.is_none() {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some((pos, normal)) = hit {
            spawn_splatter(&mut commands, &mut meshes, &mut materials, pos, normal, pb.color);
            commands.entity(entity).despawn();
        }
    }

    for (a, b, color) in new_trails {
        spawn_trail_segment(&mut commands, &mut meshes, &mut materials, a, b, color);
    }

    // Trail-Segment-Cap: bei 12 Shotgun-Pellets erzeugen wir pro Schuss
    // bis zu 200+ Segmente. Cap auf 60, um Render-Spam zu vermeiden
    // (jedes Segment hat eigenes Mesh-Handle, eigenes Material, eigene
    // Render-Entity). Optik bleibt: aelteste verschwinden eh bald.
    const MAX_TRAILS: usize = 60;
    let mut trail_count = 0;
    let mut to_despawn: Vec<bevy::ecs::entity::Entity> = Vec::new();
    for (e, _t) in param_set.p2().iter() {
        trail_count += 1;
        if trail_count > MAX_TRAILS {
            to_despawn.push(e);
        }
    }
    for e in to_despawn {
        commands.entity(e).despawn();
    }
}

fn spawn_trail_segment(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    a: Vec3,
    b: Vec3,
    color: Color,
) {
    let mid = (a + b) * 0.5;
    let dir = b - a;
    let length = dir.length();
    if length < 0.0001 {
        return;
    }
    let dir_n = dir / length;
    let rot = Quat::from_rotation_arc(Vec3::Z, dir_n);
    let mesh = meshes.add(Mesh::from(Cuboid::new(
        TRAIL_SEGMENT_THICKNESS,
        TRAIL_SEGMENT_THICKNESS,
        length,
    )));
    let mat = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh,
            material: mat,
            transform: Transform::from_translation(mid).with_rotation(rot),
            ..default()
        },
        PaintballTrail { color, age: 0.0, lifetime: TRAIL_SEGMENT_LIFETIME },
    ));
}

fn update_trail_segments(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut PaintballTrail, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_seconds();
    for (entity, mut trail, mat_handle) in &mut q {
        trail.age += dt;
        if trail.age >= trail.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = (1.0 - trail.age / trail.lifetime) * 0.85;
        if let Some(mat) = materials.get_mut(mat_handle) {
            let rgba = trail.color.to_linear();
            mat.base_color = Color::rgba(rgba.red, rgba.green, rgba.blue, alpha);
        }
    }
}

fn update_tracer_beams(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut TracerBeam, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_seconds();
    for (entity, mut tracer, mat_handle) in &mut q {
        tracer.age += dt;
        if tracer.age >= tracer.max_age {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.base_color.set_alpha(1.0 - (tracer.age / tracer.max_age));
        }
    }
}

fn spawn_splatter(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    normal: Vec3,
    color: Color,
) {
    let mesh = meshes.add(Mesh::from(Plane3d::new(Vec3::Z, Vec2::splat(0.18))));
    let mat = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let rot = Quat::from_rotation_arc(Vec3::Z, normal);
    commands.spawn((
        PbrBundle {
            mesh,
            material: mat,
            transform: Transform::from_translation(pos + normal * 0.01).with_rotation(rot),
            ..default()
        },
        SplatterDecal { lifetime: 30.0 },
    ));
}

fn update_hud_weapon(
    state: Res<WeaponState>,
    weapons: Query<&Weapon>,
    mut hud: ResMut<crate::ui::HudState>,
) {
    hud.active_weapon_name = state.active.display().to_string();
    for w in &weapons {
        if w.weapon_type == state.active {
            hud.current_ammo = w.current_ammo;
            hud.max_ammo = w.max_ammo;
            hud.reserve = w.reserve;
            return;
        }
    }
}
