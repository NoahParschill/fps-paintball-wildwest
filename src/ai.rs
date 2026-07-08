//! `ai` - KI-Bot-System (1-3 Bots).
//!
//! Bots patrouillieren am Marktplatz-Rand, suchen Deckung, orten den
//! Spieler per Raycast und feuern in Intervallen.

use crate::player::Player;
use crate::world::Cover;
use bevy::math::primitives::Capsule3d;
use bevy::prelude::*;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BotConfig>()
            .init_resource::<Score>()
            .add_systems(Startup, spawn_bots)
            .add_systems(Update, (bot_state_machine, bot_movement, bot_shoot));
    }
}

/// Zustandsmaschine eines Bots.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotState {
    /// Patrouille: langsam zwischen Wegpunkten laufen.
    Patrol,
    /// Spieler geortet - seitlich ausweichen.
    Flank,
    /// Hinter naechster Barrikade in Deckung gehen.
    Cover,
    /// Aus Deckung feuern.
    Attack,
}

#[derive(Component, Debug)]
pub struct Bot {
    pub state: BotState,
    pub target_cover: Option<Vec3>,
    pub patrol_target: Vec3,
    pub fire_cooldown: f32,
    /// Eindeutige ID pro Bot, damit Friendly-Fire moeglich ist
    /// (Bot schiesst andere Bots nicht ab).
    pub id: u32,
}

/// Health-Komponente fuer Spieler + Bots. `current <= 0` = Tod.
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
    pub fn ratio(&self) -> f32 {
        if self.max <= 0.0 { 0.0 } else { self.current / self.max }
    }
}

/// Globale Score-Anzeige. Wird vom HUD ausgelesen.
#[derive(Resource, Debug, Default)]
pub struct Score(pub u32);

/// Konfiguration: Anzahl Bots. Wird vom Spieler ueber Zifferntasten geaendert.
#[derive(Resource, Debug, Clone, Copy)]
pub struct BotConfig {
    pub count: usize,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self { count: 1 }
    }
}

fn spawn_bots(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<BotConfig>,
) {
    let n = config.count.max(1);
    for i in 0..n {
        let angle = (i as f32) * std::f32::consts::TAU / (n as f32);
        let pos = Vec3::new(angle.cos() * 10.0, 1.0, angle.sin() * 10.0);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Capsule3d::new(0.35, 1.2))),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.65, 0.20, 0.20),
                    perceptual_roughness: 0.7,
                    ..default()
                }),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Bot {
                state: BotState::Patrol,
                target_cover: None,
                patrol_target: pos + Vec3::new(5.0, 0.0, 0.0),
                fire_cooldown: 2.0,
                id: i as u32 + 1,
            },
            Health::new(100.0),
        ));
    }
}

fn bot_state_machine(mut bots: Query<&mut Bot>, player: Query<&Transform, With<Player>>) {
    let Ok(player_tf) = player.get_single() else {
        return;
    };
    let player_pos = player_tf.translation;

    for mut bot in &mut bots {
        let to_player = player_pos - bot.patrol_target;
        let dist = to_player.length();

        // Sehr einfach gehaltene State-Machine.
        bot.state = if dist < 6.0 {
            // Spieler nah - Deckung suchen und feuern.
            if bot.target_cover.is_some() {
                BotState::Attack
            } else {
                BotState::Cover
            }
        } else if dist < 20.0 {
            BotState::Flank
        } else {
            BotState::Patrol
        };
    }
}

fn bot_movement(
    mut bots: Query<(&mut Bot, &mut Transform)>,
    time: Res<Time>,
    cover_q: Query<(&Transform, &crate::world::Cover)>,
) {
    for (mut bot, mut tf) in &mut bots {
        let target = match bot.state {
            BotState::Patrol => bot.patrol_target,
            BotState::Flank => bot.patrol_target + Vec3::new(0.0, 0.0, 2.0),
            BotState::Cover | BotState::Attack => {
                if let Some(c) = bot.target_cover {
                    c
                } else {
                    bot.patrol_target
                }
            }
        };

        let dir = (target - tf.translation).with_y(0.0);
        let dist = dir.length();
        if dist > 0.5 {
            let step = dir.normalize() * 2.0 * time.delta_seconds();
            let next = tf.translation + step;
            tf.translation = crate::player::resolve_cover_collision(
                tf.translation, next, &cover_q, 0.35,
            );
            // Richtung grob ausrichten.
            let look = (target - tf.translation).with_y(0.0).normalize_or_zero();
            if look != Vec3::ZERO {
                tf.look_at(target, Vec3::Y);
            }
        }
    }
}

fn bot_shoot(
    mut commands: Commands,
    mut bots: Query<(Entity, &Transform, &mut Bot, &mut Health)>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, bot_tf, mut bot, mut bot_hp) in &mut bots {
        if bot_hp.is_dead() {
            // Tote Bots schiessen nicht mehr.
            continue;
        }
        if bot.state != BotState::Attack {
            continue;
        }
        bot.fire_cooldown -= time.delta_seconds();
        if bot.fire_cooldown > 0.0 {
            continue;
        }
        bot.fire_cooldown = 1.5;

        // Schuss in Richtung Spieler.
        let Ok(player_tf) = player.get_single() else {
            continue;
        };
        let dir = (player_tf.translation - bot_tf.translation).normalize_or(Vec3::X);

        crate::weapon::spawn_paintball(
            &mut commands,
            &mut meshes,
            &mut materials,
            bot_tf.translation + Vec3::Y * 1.2,
            dir * 30.0,
            // Bots schiessen rote Paintballs.
            Color::rgb(0.95, 0.20, 0.10),
            crate::weapon::Owner::Bot(bot.id),
        );
        // Vermeide "unused variable"-Warnung.
        let _ = entity;
    }
}
