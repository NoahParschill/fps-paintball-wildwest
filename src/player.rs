//! `player` - First-Person-Spieler-Controller mit Sprint / Crouch / Slide.
//!
//!  * First-Person-Kamera (an Spieler-Entity gebunden)
//!  * Look ueber Pfeiltasten
//!  * WASD-Bewegung relativ zur Blickrichtung
//!  * Sprung ueber Leertaste
//!  * Shift = Sprint
//!  * Ctrl = Crouch
//!  * Shift + Ctrl (im Sprint) = Slide (Boost voraus, dann auslaufen)
//!  * Hoehe der Spieler-Kapsel aendert sich mit Crouch / Slide
//!  * Esc = Spiel beenden (eindeutige Aktion statt "Fenster schliesst
//!    still bei Random-Tastendruck" Symptom aus frueherer Iteration).
//!
//! Die Komponenten `Player` (Marker) und `PlayerLook` (Yaw/Pitch)
//! sind in `weapon.rs` referenziert (Waffen-Spawn am Spieler).

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                // Reihenfolge mit .chain(): arrow_look VOR player_movement,
                // damit die frisch berechnete Rotation direkt fuer die
                // Bewegungsrichtung genutzt wird.
                (arrow_look, player_movement, player_jump, update_player_pose)
                    .chain()
                    .in_set(PlayerMutationSet),
            )
            // Esc sendet explizit ein AppExit-Event. So ist klar, dass
            // Esc das Spiel beendet - kein Mystery-Close mehr.
            .add_systems(Update, esc_to_exit);
    }
}

/// System-Set fuer Spieler-Transform-Mutationen. Garantiert, dass mouse_look
/// und player_movement serialisiert laufen, weil beide `&mut Transform` auf
/// der Spieler-Entity brauchen.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct PlayerMutationSet;

/// Hoehe des Spielers (Augenhoehe) in den drei Hauptposen.
///
/// - `STAND` = 1.6 m (default)
/// - `CROUCH` = 1.0 m (ca. 0.6m kleiner)
/// - `SLIDE` = 0.7 m (noch tiefer, weil Slide ein "Duck-Sprint" ist)
const EYE_STAND: f32 = 1.6;
const EYE_CROUCH: f32 = 1.0;
const EYE_SLIDE: f32 = 0.7;

const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 8.5;
const CROUCH_SPEED: f32 = 2.5;
const SLIDE_BOOST_SPEED: f32 = 11.0; // Eingangsgeschwindigkeit
const SLIDE_DECAY: f32 = 4.5; // m/s^2 -> bremst auf Sprint-Speed ab
const SLIDE_MIN_DURATION: f32 = 0.35;
const SLIDE_MAX_DURATION: f32 = 0.85;

const JUMP_SPEED: f32 = 5.5;
const GRAVITY: f32 = -15.0;
// Tastatur-Look: konstante Rotationsrate (rad/s) pro gedrueckter Pfeiltaste.
const LOOK_RATE: f32 = 1.8; // ~103 deg/s
const PITCH_LIMIT: f32 = 85.0_f32.to_radians();

/// Marker-Komponente fuer den Spieler.
#[derive(Component)]
pub struct Player;

/// Yaw- (um Y) und Pitch- (um X) Winkel der First-Person-Kamera.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerLook {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for PlayerLook {
    fn default() -> Self {
        // Initiale Yaw-Rotation = -2.4 rad (siehe spawn_player), damit beim
        // ersten Frame kein Sprung passiert. Sonst wuerde die Maus-Logik mit
        // yaw=0.0 starten, dann player_tf.rotation auf Quat::from_axis_angle(Y, 0)
        // ueberschreiben - visuell ein 137-Grad-Sprung beim ersten Maus-Event.
        Self {
            yaw: -2.4,
            pitch: 0.0,
        }
    }
}

/// Physikalisches Geschwindigkeits-Vektor-Component (fuer Sprung & Bewegung).
#[derive(Component, Debug, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

/// Body-Status des Spielers: crouch, sprint, slide.
///
/// `crouched` und `sprinting` sind reine Input-Tasten-Zustaende.
/// `sliding` ist eine transiente Animation, die automatisch startet
/// wenn sprinting + crouched gleichzeitig gedrueckt werden (am Boden)
/// und nach `SLIDE_DURATION` Sekunden endet. Waehrend des Slides
/// werden neue Slide-Trigger ignoriert, bis `crouched` losgelassen
/// wurde (sonst Slide-Bug bei gedrueckter Ctrl).
#[derive(Component, Debug, Default)]
pub struct PlayerStance {
    pub crouched: bool,
    pub sprinting: bool,
    pub sliding: bool,
    /// Verbleibende Slide-Zeit in Sekunden. Wird auf SLIDE_MAX_DURATION
    /// gesetzt, sobald ein Slide getriggert wird, und laeuft linear ab.
    pub slide_timer: f32,
    /// Aktuelle horizontale Geschwindigkeit, die der Slide beibehalten
    /// soll (snapshot zum Start). Wir verwenden den Snapshot NICHT
    /// direkt, sondern die horizontale Geschwindigkeit in `Velocity`.
    pub slide_speed: f32,
    /// Aktuelle Eye-Height, sanft interpoliert zwischen Stand/Crouch/Slide.
    /// Wird in `update_player_pose` aktualisiert und vom Waffen-Viewmodel
    /// gelesen (indirekt ueber die Spieler-Transform).
    pub eye_height: f32,
    /// Sanfter Crouch-Interp-Faktor: 0.0 = steht, 1.0 = crouch.
    pub crouch_blend: f32,
}

pub fn spawn_player(mut commands: Commands) {
    // Spieler-Entity (unsichtbar) traegt Kamera als Kind.
    // Spawn auf (3, 1.6, 3), damit er nicht im Brunnen bei (0,0,0) steckt
    // und nicht in einer Cover-Barrikade.
    let player = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(3.0, 1.6, 3.0))
                    .with_rotation(Quat::from_rotation_y(-2.4)), // schaut Richtung Brunnen
                ..default()
            },
            Player,
            PlayerLook::default(),
            Velocity::default(),
            // PlayerStance komplett explizit (kein ..default() damit wir
            // sicher eye_height = 1.6 setzen). Default wuerde 0.0 setzen
            // und der erste update_player_pose-Frame wuerde den Spieler
            // auf y=0.0 snapen.
            PlayerStance {
                crouched: false,
                sprinting: false,
                sliding: false,
                slide_timer: 0.0,
                slide_speed: 0.0,
                eye_height: EYE_STAND,
                crouch_blend: 0.0,
            },
            crate::ai::Health::new(100.0),
        ))
        .id();

    // Kamera als Kind des Spielers.
    commands.entity(player).with_children(|parent| {
        parent.spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        });
    });
}

/// Esc sendet AppExit-Event, damit das Schliessen deterministisch und
/// beabsichtigt ist. Kein No-Op mehr.
fn esc_to_exit(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

/// Tastatur-Look: Pfeiltasten rotieren die Kamera mit konstanter Rate.
///   * Links / Rechts  -> Yaw (um Y-Achse, links/rechts drehen)
///   * Hoch / Runter   -> Pitch (um X-Achse, hoch/runter schauen)
///
/// Wir packen arrow_look und player_movement in `PlayerMutationSet` mit
/// `.chain()`, weil BEIDE `&mut Transform` auf der Spieler-Entity brauchen.
fn arrow_look(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&mut PlayerLook, &mut Transform, &Children), With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok((mut look, mut player_tf, children)) = player_q.get_single_mut() else {
        return;
    };

    let mut yaw_input: f32 = 0.0;
    let mut pitch_input: f32 = 0.0;
    if keys.pressed(KeyCode::ArrowLeft) {
        yaw_input += 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        yaw_input -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        pitch_input += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        pitch_input -= 1.0;
    }

    if yaw_input == 0.0 && pitch_input == 0.0 {
        return;
    }

    let dt = time.delta_seconds();
    // Yaw: Links positiv (im Uhrzeigersinn von oben gesehen), Rechts negativ.
    look.yaw += yaw_input * LOOK_RATE * dt;
    // Pitch: Hoch positiv (nach oben schauen), Runter negativ.
    look.pitch = (look.pitch + pitch_input * LOOK_RATE * dt)
        .clamp(-PITCH_LIMIT, PITCH_LIMIT);

    // Yaw auf den Spieler (Rotation um Y).
    player_tf.rotation = Quat::from_axis_angle(Vec3::Y, look.yaw);

    // Pitch auf die Kamera (als Kind des Spielers).
    for &child in children.iter() {
        if let Ok(mut cam_tf) = camera_q.get_mut(child) {
            cam_tf.rotation = Quat::from_axis_angle(Vec3::X, look.pitch);
        }
    }
}

/// Bewegt den Spieler relativ zur Blickrichtung (WASD) und integriert Geschwindigkeit.
///
/// Logik:
///   1. WASD-Input in Wunsch-Richtung umrechnen.
///   2. Aus `PlayerStance` die aktuelle Maximalgeschwindigkeit bestimmen:
///      - Slide   : SLIDE_BOOST_SPEED, faellt mit SLIDE_DECAY ab
///      - Sprint  : SPRINT_SPEED (nur wenn Ctrl nicht gedrueckt)
///      - Crouch  : CROUCH_SPEED
///      - Default : WALK_SPEED
///   3. Wunschgeschwindigkeit anwenden.
///   4. Schwerkraft integrieren.
///   5. Slide-Logik: startet automatisch bei Shift+Ctrl, endet fruehestens
///      nach `SLIDE_MIN_DURATION` (sonst "Dauer-Slide-Bug") und automatisch
///      nach `SLIDE_MAX_DURATION` ODER wenn Ctrl losgelassen wird.
fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &PlayerLook, &mut PlayerStance), With<Player>>,
) {
    let Ok((mut transform, mut vel, _look, mut stance)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_seconds();

    // Input-Auslese.
    let mut input = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        input.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        input.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        input.x += 1.0;
    }

    // Shift = Sprint, Ctrl = Crouch.
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // Vorwaerts / rechts aus Yaw ableiten.
    let forward = transform.forward();
    let right = transform.right();
    let mut wish_dir = (forward * -input.z + right * input.x).with_y(0.0);
    if wish_dir.length_squared() > 0.0 {
        wish_dir = wish_dir.normalize();
    }
    let has_input = wish_dir.length_squared() > 0.0;

    // Boden-Check: Spieler y ist Bodenhöhe + Spieler-Y-Basis. Aktuelle
    // Eye-Height ist `stance.eye_height` (interpoliert), also:
    //   on_ground = translation.y ≈ eye_height (für nicht-airborne)
    // Wir verwenden 0.05 m Toleranz, damit Springen-Schwerkraft sauber
    // durchgreifen kann.
    let on_ground = (transform.translation.y - stance.eye_height).abs() < 0.05;

    // ---- Slide-Trigger ----
    // Slide startet NUR wenn:
    //   * Shift (sprinting) und Ctrl (crouching) BEIDE gedrueckt
    //   * am Boden
    //   * aktive horizontale Bewegung gewuenscht (wir wollen Aktion, kein Idle-Slide)
    //   * wir nicht schon im Slide sind (sonst Dauer-Slide)
    if !stance.sliding && shift && ctrl && on_ground && has_input {
        stance.sliding = true;
        stance.slide_timer = SLIDE_MAX_DURATION;
        // Initialer Boost in Blickrichtung (Vorwärts = -Z).
        vel.linear.x = wish_dir.x * SLIDE_BOOST_SPEED;
        vel.linear.z = wish_dir.z * SLIDE_BOOST_SPEED;
    }

    // ---- Slide-Tick ----
    if stance.sliding {
        stance.slide_timer -= dt;
        // Slide endet, wenn:
        //   1. Timer abgelaufen (maximale Dauer)
        //   2. Ctrl losgelassen UND Mindestdauer vorbei
        //   3. vom Boden abgekommen (z. B. von Kante gerutscht)
        let ctrl_released = !ctrl;
        let min_done = stance.slide_timer <= (SLIDE_MAX_DURATION - SLIDE_MIN_DURATION);
        if stance.slide_timer <= 0.0
            || (ctrl_released && min_done)
            || !on_ground
        {
            stance.sliding = false;
        } else {
            // Decay: Geschwindigkeit faellt gleichmaessig Richtung
            // SPRINT_SPEED ab, damit der Spieler noch kontrolliert
            // weiter rutscht.
            let horiz_speed =
                (vel.linear.x * vel.linear.x + vel.linear.z * vel.linear.z).sqrt();
            if horiz_speed > SPRINT_SPEED {
                let new_speed = (horiz_speed - SLIDE_DECAY * dt).max(SPRINT_SPEED);
                let k = if horiz_speed > 0.0001 {
                    new_speed / horiz_speed
                } else {
                    0.0
                };
                vel.linear.x *= k;
                vel.linear.z *= k;
            }
        }
    }

    // ---- Geschwindigkeits-Auswahl (Nicht-Slide) ----
    if !stance.sliding {
        let max_speed = if ctrl {
            CROUCH_SPEED
        } else if shift {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        if has_input {
            vel.linear.x = wish_dir.x * max_speed;
            vel.linear.z = wish_dir.z * max_speed;
        } else {
            // Schnell auf 0 abbremsen fuer responsive FPS-Feel.
            // Horizontal-Velocity linear abklingen.
            let decel = max_speed * 8.0 * dt; // voller Stopp in ~0.125s
            let hx = vel.linear.x.abs().min(decel).copysign(vel.linear.x);
            let hz = vel.linear.z.abs().min(decel).copysign(vel.linear.z);
            vel.linear.x -= hx;
            vel.linear.z -= hz;
        }
    }

    // ---- Stance-Flags ----
    stance.crouched = ctrl;
    stance.sprinting = shift && !stance.sliding;

    // ---- Schwerkraft ----
    if !on_ground {
        vel.linear.y += GRAVITY * time.delta_seconds();
    } else if vel.linear.y < 0.0 {
        vel.linear.y = 0.0;
    }

    // Integriere Position.
    transform.translation += vel.linear * time.delta_seconds();

    // Snap auf Bodenhoehe: y = eye_height (aktuell interpoliert).
    let target_y = stance.eye_height;
    if transform.translation.y < target_y {
        transform.translation.y = target_y;
        vel.linear.y = 0.0;
    }
}

/// Sprung ueber Leertaste, nur wenn am Boden.
/// Springen waehrend Slide ist NICHT erlaubt (Realismus-Feel:
/// erst aus dem Slide raus, dann springen). Springen aus dem Crouch
/// heraus ist erlaubt.
fn player_jump(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity, &PlayerStance), With<Player>>,
) {
    let Ok((transform, mut vel, stance)) = query.get_single_mut() else {
        return;
    };
    let on_ground = (transform.translation.y - stance.eye_height).abs() < 0.05;
    if keys.just_pressed(KeyCode::Space) && on_ground && !stance.sliding {
        vel.linear.y = JUMP_SPEED;
    }
}

/// Interpoliert die Spieler-Eye-Height sanft zwischen Stand, Crouch und Slide.
///
/// Wird in einem separaten System nach `player_movement` aufgerufen,
/// damit der Slide-Tick in diesem Frame bereits `stance.sliding`
/// aktualisiert hat. Die Interpolationsrate ist 12 m/s, das ergibt
/// ein knackiges Ducken ohne "Bouncy"-Artefakte.
fn update_player_pose(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerStance), With<Player>>,
) {
    let Ok((mut transform, mut stance)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_seconds();
    let target = if stance.sliding {
        EYE_SLIDE
    } else if stance.crouched {
        EYE_CROUCH
    } else {
        EYE_STAND
    };
    // Sanfte Annäherung: eye_height → target. 12 m/s ergibt ein
    // knackiges Ducken ohne "Bouncy"-Artefakte.
    let rate = 12.0;
    let step = rate * dt;
    stance.eye_height += (target - stance.eye_height).clamp(-step, step);

    // Crouch-Blend-Faktor fuer externe Systeme (Viewmodel, Spread).
    // 0.0 = steht, 1.0 = voll gecroucht.
    let crouch_target = if stance.crouched { 1.0 } else { 0.0 };
    stance.crouch_blend += (crouch_target - stance.crouch_blend).clamp(-step, step);

    // Y-Position synchronisieren. Wenn `update_player_pose` zum ersten
    // Mal laeuft, ist `eye_height` 1.6 (aus spawn) und transform.y
    // ebenfalls 1.6 -> keine Spruenge. Bei Crouch geht transform.y
    // runter auf eye_height, das ist der "Duck"-Effekt auf der
    // Spieler-Hitbox.
    if (stance.eye_height - transform.translation.y).abs() > 0.0001 {
        transform.translation.y = stance.eye_height;
    }
}
