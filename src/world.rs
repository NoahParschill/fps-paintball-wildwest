//! `world` - Level-Aufbau und Sonnenuntergangs-Beleuchtung.
//!
//! Wir bauen prozedural einen dichten Wild-West-Marktplatz mit 30+
//! unterschiedlichen Objekttypen. Da die .fbx-Originale nicht direkt
//! von Bevy geladen werden koennen, verwenden wir Primitive-Formen
//! (Cuboid, Cylinder, Capsule, ...), die in Groesse, Farbe und
//! Anordnung den Look der FBX-Assets nachempfinden.
//!
//! Das Layout ist asymmetrisch und vermeidet bewusst gleichmaessige
//! Verteilungen, damit der Marktplatz lebendig und nicht "gekachelt"
//! wirkt.

use bevy::math::primitives::{Cylinder, Plane3d};
use bevy::prelude::*;

/// Marker-Komponente: Deckungs-Stelle im Level.
///
/// Wird vom KI-System (`ai.rs`) genutzt, um Bots in die
/// naechste Barrikade fliehen zu lassen. `half_extents` wird
/// fuer Character-vs-Cover-Kollision benoetigt.
#[derive(Component, Debug, Clone, Copy)]
pub struct Cover {
    pub position: Vec3,
    pub half_extents: Vec3,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_sunset_lighting, setup_marketplace));
    }
}

/// Warme, tiefstehende Sonnenuntergangs-Beleuchtung.
fn setup_sunset_lighting(mut commands: Commands) {
    // Key light - tiefstehende Sonne, warm orange-rot.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 0.55, 0.25),
            illuminance: 35_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.95, // ~ -54 deg
            0.6,   // leichter seitlicher Einfallswinkel
            0.0,
        )),
        ..default()
    });

    // Fill light - kalter Lila-Ton von der Gegenseite.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.35, 0.30, 0.55),
            illuminance: 6_000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, -2.4, 0.0)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.6, 0.45, 0.55),
        brightness: 0.40,
    });

    commands.insert_resource(ClearColor(Color::rgb(0.85, 0.45, 0.30)));
}

// =============================================================================
// Materialien - vorberechnet, damit wir nicht hunderte StandardMaterial-Duplikate
// im AssetServer haben. Jeder Aufruf von materials.add(...) erzeugt eine neue
// Handle; dieselbe Handle mehrfach zu nutzen ist effizienter.
// =============================================================================

struct WildWestMats {
    ground: Handle<StandardMaterial>,
    wood_dark: Handle<StandardMaterial>,
    wood_mid: Handle<StandardMaterial>,
    wood_light: Handle<StandardMaterial>,
    wood_plank: Handle<StandardMaterial>,
    metal: Handle<StandardMaterial>,
    metal_rusty: Handle<StandardMaterial>,
    barrel: Handle<StandardMaterial>,
    brick: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    thatch: Handle<StandardMaterial>,
    canvas: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
    cactus: Handle<StandardMaterial>,
    sand: Handle<StandardMaterial>,
    red_cloth: Handle<StandardMaterial>,
    blue_cloth: Handle<StandardMaterial>,
    green_cloth: Handle<StandardMaterial>,
    lamp_glow: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    leaf_green: Handle<StandardMaterial>,
    leaf_orange: Handle<StandardMaterial>,
}

fn build_materials(materials: &mut Assets<StandardMaterial>) -> WildWestMats {
    let ground = materials.add(StandardMaterial {
        base_color: Color::rgb(0.62, 0.45, 0.30),
        perceptual_roughness: 1.0,
        ..default()
    });
    let wood_dark = materials.add(StandardMaterial {
        base_color: Color::rgb(0.30, 0.18, 0.10),
        perceptual_roughness: 0.95,
        ..default()
    });
    let wood_mid = materials.add(StandardMaterial {
        base_color: Color::rgb(0.50, 0.32, 0.18),
        perceptual_roughness: 0.9,
        ..default()
    });
    let wood_light = materials.add(StandardMaterial {
        base_color: Color::rgb(0.65, 0.45, 0.28),
        perceptual_roughness: 0.9,
        ..default()
    });
    let wood_plank = materials.add(StandardMaterial {
        base_color: Color::rgb(0.58, 0.38, 0.22),
        perceptual_roughness: 0.95,
        ..default()
    });
    let metal = materials.add(StandardMaterial {
        base_color: Color::rgb(0.55, 0.55, 0.58),
        perceptual_roughness: 0.4,
        metallic: 0.8,
        ..default()
    });
    let metal_rusty = materials.add(StandardMaterial {
        base_color: Color::rgb(0.45, 0.30, 0.18),
        perceptual_roughness: 0.7,
        metallic: 0.6,
        ..default()
    });
    let barrel = materials.add(StandardMaterial {
        base_color: Color::rgb(0.45, 0.28, 0.15),
        perceptual_roughness: 0.95,
        ..default()
    });
    let brick = materials.add(StandardMaterial {
        base_color: Color::rgb(0.55, 0.30, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    let stone = materials.add(StandardMaterial {
        base_color: Color::rgb(0.55, 0.50, 0.45),
        perceptual_roughness: 0.95,
        ..default()
    });
    let thatch = materials.add(StandardMaterial {
        base_color: Color::rgb(0.70, 0.55, 0.25),
        perceptual_roughness: 1.0,
        ..default()
    });
    let canvas = materials.add(StandardMaterial {
        base_color: Color::rgb(0.85, 0.75, 0.50),
        perceptual_roughness: 0.95,
        ..default()
    });
    let water = materials.add(StandardMaterial {
        base_color: Color::rgb(0.20, 0.40, 0.55),
        perceptual_roughness: 0.1,
        metallic: 0.2,
        ..default()
    });
    let cactus = materials.add(StandardMaterial {
        base_color: Color::rgb(0.30, 0.50, 0.25),
        perceptual_roughness: 0.95,
        ..default()
    });
    let sand = materials.add(StandardMaterial {
        base_color: Color::rgb(0.80, 0.65, 0.40),
        perceptual_roughness: 1.0,
        ..default()
    });
    let red_cloth = materials.add(StandardMaterial {
        base_color: Color::rgb(0.70, 0.15, 0.15),
        perceptual_roughness: 0.95,
        ..default()
    });
    let blue_cloth = materials.add(StandardMaterial {
        base_color: Color::rgb(0.15, 0.30, 0.65),
        perceptual_roughness: 0.95,
        ..default()
    });
    let green_cloth = materials.add(StandardMaterial {
        base_color: Color::rgb(0.20, 0.55, 0.25),
        perceptual_roughness: 0.95,
        ..default()
    });
    let lamp_glow = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.75, 0.30),
        emissive: LinearRgba::rgb(1.0, 0.65, 0.20),
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        base_color: Color::rgb(0.7, 0.85, 0.95),
        perceptual_roughness: 0.05,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let leaf_green = materials.add(StandardMaterial {
        base_color: Color::rgb(0.20, 0.45, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    let leaf_orange = materials.add(StandardMaterial {
        base_color: Color::rgb(0.75, 0.40, 0.10),
        perceptual_roughness: 0.95,
        ..default()
    });

    WildWestMats {
        ground,
        wood_dark,
        wood_mid,
        wood_light,
        wood_plank,
        metal,
        metal_rusty,
        barrel,
        brick,
        stone,
        thatch,
        canvas,
        water,
        cactus,
        sand,
        red_cloth,
        blue_cloth,
        green_cloth,
        lamp_glow,
        glass,
        leaf_green,
        leaf_orange,
    }
}

// =============================================================================
// Primitives
// =============================================================================

fn plane(
    meshes: &mut ResMut<Assets<Mesh>>,
    size: f32,
) -> Handle<Mesh> {
    meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(size * 0.5))))
}

fn box_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    sx: f32,
    sy: f32,
    sz: f32,
) -> Handle<Mesh> {
    meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(sx, sy, sz)))
}

fn cylinder_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    radius: f32,
    height: f32,
) -> Handle<Mesh> {
    meshes.add(Mesh::from(Cylinder { radius, half_height: height * 0.5 }))
}

fn capsule_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    radius: f32,
    length: f32,
) -> Handle<Mesh> {
    meshes.add(Mesh::from(bevy::math::primitives::Capsule3d::new(radius, length)))
}

fn sphere_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    radius: f32,
) -> Handle<Mesh> {
    meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius }))
}

// =============================================================================
// Spawner - jeder Spawner platziert EIN konkretes Objekt mit konkreter
// Position, Rotation und Maszen. Wir verteilen sie asymmetrisch.
// =============================================================================

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    commands.spawn(PbrBundle {
        mesh: plane(meshes, 80.0),
        material: mats.ground.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
}

// ---------- HAUS 1: Saloon (Sued-Ost) ----------
fn spawn_saloon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(13.0, 0.0, 6.0);
    // Hauptgebaeude
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 10.0, 4.0, 7.0),
        material: mats.wood_light.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 2.0, 0.0)),
        ..default()
    });
    // Dach (flacher Sattel)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 10.6, 0.2, 7.6),
        material: mats.thatch.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 4.15, 0.0)),
        ..default()
    });
    // Vordach ueber Eingang (Sued-Seite, +Z)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 4.5, 0.15, 1.6),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 2.8, 3.55))
            .with_rotation(Quat::from_rotation_y(0.15)),
        ..default()
    });
    // 4 Stuetzen des Vordachs
    for dx in [-1.8, 1.8] {
        for dz in [3.0, 3.7] {
            commands.spawn(PbrBundle {
                mesh: cylinder_mesh(meshes, 0.10, 2.6),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(
                    base + Vec3::new(dx, 1.3, dz),
                ),
                ..default()
            });
        }
    }
    // Schwingtueren (zwei braune Rechtecke in der Sued-Fassade)
    for dx in [-0.7, 0.7] {
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 1.1, 2.6, 0.15),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(
                base + Vec3::new(dx, 1.3, 3.51),
            )
            .with_rotation(Quat::from_rotation_y(if dx < 0.0 { 0.35 } else { -0.35 })),
            ..default()
        });
    }
    // Fenster (zwei, Nord-Seite)
    for dx in [-3.0, 3.0] {
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 1.0, 1.2, 0.08),
            material: mats.glass.clone(),
            transform: Transform::from_translation(
                base + Vec3::new(dx, 2.6, -3.51),
            ),
            ..default()
        });
    }
    // Schild "SALOON" ueber der Tuer
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 3.6, 0.8, 0.12),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(
            base + Vec3::new(0.0, 3.55, 3.65),
        )
        .with_rotation(Quat::from_rotation_y(0.05)),
        ..default()
    });
    // Laternen (zwei, neben Eingang)
    for dx in [-2.4, 2.4] {
        let lantern_pos = base + Vec3::new(dx, 1.8, 3.5);
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.04, 1.6),
            material: mats.metal_rusty.clone(),
            transform: Transform::from_translation(lantern_pos + Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: sphere_mesh(meshes, 0.16),
            material: mats.lamp_glow.clone(),
            transform: Transform::from_translation(lantern_pos + Vec3::new(0.0, 0.85, 0.0)),
            ..default()
        });
        // Echtes Punktlicht an der Laterne
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(1.0, 0.65, 0.20),
                intensity: 4000.0,
                radius: 0.5,
                range: 8.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_translation(lantern_pos + Vec3::new(0.0, 0.85, 0.0)),
            ..default()
        });
    }
}

// ---------- HAUS 2: Sheriff Office (Nord-West) ----------
fn spawn_sheriff(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(-13.0, 0.0, -7.0);
    // Hauptgebaeude (etwas kleiner)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 8.0, 3.5, 6.0),
        material: mats.brick.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.75, 0.0)),
        ..default()
    });
    // Dach
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 8.6, 0.2, 6.6),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 3.65, 0.0)),
        ..default()
    });
    // Veranda (West-Seite, -X)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 2.0, 0.12, 5.0),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(base + Vec3::new(-4.0, 1.1, 0.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // Veranda-Stuetzen
    for dz in [-2.0, 2.0] {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.09, 1.1),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(base + Vec3::new(-4.7, 0.55, dz))
                .with_rotation(Quat::from_rotation_z(0.0),
            ),
            ..default()
        });
    }
    // Tuer (West-Seite)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 0.12, 2.2, 1.0),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(-3.95, 1.1, 0.0)),
        ..default()
    });
    // Fenster an Nord- und Sued-Seite
    for dz in [-2.2, 2.2] {
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 0.08, 1.0, 0.9),
            material: mats.glass.clone(),
            transform: Transform::from_translation(base + Vec3::new(0.0, 2.3, dz)),
            ..default()
        });
    }
    // Stern-Schild (silberner Cylinder, am Giebel)
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.45, 0.08),
        material: mats.metal.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 4.2, 3.0))
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}

// ---------- HAUS 3: Stall (Nord-Ost) ----------
fn spawn_stable(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(13.0, 0.0, -8.0);
    // Langes, niedriges Gebaeude
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 7.0, 3.0, 4.0),
        material: mats.wood_mid.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.5, 0.0)),
        ..default()
    });
    // Schraeges Dach (zwei schraege Quader)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 4.5, 0.15, 5.5),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(-1.5, 3.0, 0.0))
            .with_rotation(Quat::from_rotation_z(0.35)),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 4.5, 0.15, 5.5),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(1.5, 3.0, 0.0))
            .with_rotation(Quat::from_rotation_z(-0.35)),
        ..default()
    });
    // Grosser Stall-Tor
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 0.12, 2.0, 1.6),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(3.51, 1.0, 0.0)),
        ..default()
    });
    // Schieber (horizontale Latten, nur dekorativ)
    for y in [0.5, 1.0, 1.5] {
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 0.14, 0.08, 1.5),
            material: mats.metal_rusty.clone(),
            transform: Transform::from_translation(base + Vec3::new(3.55, y, 0.0)),
            ..default()
        });
    }
    // Heuballen innen (durch offene Tuer sichtbar)
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.55, 0.7),
        material: mats.thatch.clone(),
        transform: Transform::from_translation(base + Vec3::new(1.5, 0.35, -0.6))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}

// ---------- HAUS 4: kleines Holzhaus (Sued-West) ----------
fn spawn_small_house(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(-12.0, 0.0, 9.0);
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 5.0, 3.0, 4.0),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.5, 0.0)),
        ..default()
    });
    // Pyramidendach
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.0, 1.4),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 3.7, 0.0))
            .with_rotation(Quat::from_rotation_x(0.0),
        ),
        ..default()
    });
    // Tuer
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 0.10, 2.0, 0.9),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.0, 2.0)),
        ..default()
    });
    // Fenster
    for dx in [-1.6, 1.6] {
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 0.08, 0.9, 0.8),
            material: mats.glass.clone(),
            transform: Transform::from_translation(base + Vec3::new(dx, 1.9, 2.0)),
            ..default()
    });
    }
}

// ---------- Windmühle (Nord-Mitte) ----------
fn spawn_windmill(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(0.0, 0.0, -18.0);
    // Turm
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 1.2, 8.0),
        material: mats.wood_mid.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 4.0, 0.0)),
        ..default()
    });
    // Dach (kegel)
    commands.spawn(PbrBundle {
        mesh: cone_mesh(meshes, 1.5, 2.0),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 9.0, 0.0)),
        ..default()
    });
    // 4 Fluegel
    for i in 0..4 {
        let rot = Quat::from_rotation_z(i as f32 * std::f32::consts::FRAC_PI_2);
        commands.spawn((
            PbrBundle {
                mesh: box_mesh(meshes, 0.15, 4.0, 0.6),
                material: mats.wood_plank.clone(),
                transform: Transform::from_translation(base + Vec3::new(0.0, 6.5, 0.0))
                    .with_rotation(rot * Quat::from_rotation_y(0.2)),
                ..default()
            },
            WindmillBlade { angle: i as f32 * std::f32::consts::FRAC_PI_2 },
        ));
    }
}

fn cone_mesh(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, height: f32) -> Handle<Mesh> {
    meshes.add(Mesh::from(bevy::math::primitives::Cone { radius, height }))
}

// Marker, damit wir die Fluegel spaeter drehen koennen (wenn Zeit ist).
#[derive(Component)]
struct WindmillBlade {
    pub angle: f32,
}

// ---------- Wasserturm (Sued-Mitte) ----------
fn spawn_water_tower(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(0.0, 0.0, 16.0);
    // 4 Stelzen
    for (dx, dz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.12, 4.5),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(base + Vec3::new(dx, 2.25, dz)),
            ..default()
        });
    }
    // Wassertank
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 1.4, 1.8),
        material: mats.wood_mid.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 5.4, 0.0)),
        ..default()
    });
    // Dach
    commands.spawn(PbrBundle {
        mesh: cone_mesh(meshes, 1.6, 1.0),
        material: mats.thatch.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 6.8, 0.0)),
        ..default()
    });
    // Wasser-Ring (innen sichtbar, oben)
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 1.35, 0.05),
        material: mats.water.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 5.95, 0.0)),
        ..default()
    });
}

// ---------- Brunnen (Marktplatz-Zentrum) ----------
fn spawn_well(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let center = Vec3::new(0.0, 0.0, 0.0);
    // Brunnen-Mantel
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.9, 0.9),
        material: mats.stone.clone(),
        transform: Transform::from_translation(center + Vec3::new(0.0, 0.45, 0.0)),
        ..default()
    });
    // Wasser
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.85, 0.06),
        material: mats.water.clone(),
        transform: Transform::from_translation(center + Vec3::new(0.0, 0.85, 0.0)),
        ..default()
    });
    // 2 Stuetzen + Dach
    for dx in [-0.7, 0.7] {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.07, 1.6),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(center + Vec3::new(dx, 1.5, 0.0)),
            ..default()
        });
    }
    commands.spawn(PbrBundle {
        mesh: cone_mesh(meshes, 1.0, 0.6),
        material: mats.thatch.clone(),
        transform: Transform::from_translation(center + Vec3::new(0.0, 2.4, 0.0)),
        ..default()
    });
}

// ---------- Galgen (am Rand) ----------
fn spawn_gallows(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(-18.0, 0.0, 0.0);
    // 2 senkrechte Pfosten
    for dx in [-0.5, 0.5] {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.12, 4.5),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(base + Vec3::new(dx, 2.25, 0.0)),
            ..default()
        });
    }
    // Querbalken
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.12, 2.5),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 4.3, 0.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // Seil
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.03, 1.2),
        material: mats.metal_rusty.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.5, 3.6, 0.0)),
        ..default()
    });
    // Plattform
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 2.4, 0.2, 2.4),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 0.6, 0.0)),
        ..default()
    });
}

// ---------- Kutsche (Cart) ----------
fn spawn_stagecoach(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(8.0, 0.0, 10.0);
    let rot = Quat::from_rotation_y(0.6);
    // Karosserie
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 2.4, 1.3, 4.0),
        material: mats.red_cloth.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.45, 0.0))
            .with_rotation(rot),
        ..default()
    });
    // Verdeck
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 2.5, 0.15, 3.2),
        material: mats.canvas.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 2.55, 0.0))
            .with_rotation(rot),
        ..default()
    });
    // 4 Raeder (Cylinder, liegend)
    for (dx, dz) in [(-1.1, -1.4), (1.1, -1.4), (-1.1, 1.4), (1.1, 1.4)] {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.55, 0.15),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(base + Vec3::new(dx, 0.55, dz))
                .with_rotation(rot * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
    }
    // Deichsel
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.07, 2.4),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 0.7, 2.8))
            .with_rotation(rot * Quat::from_rotation_x(-0.3)),
        ..default()
    });
}

// ---------- Hufeisen-Pfosten (Hitching Post) ----------
fn spawn_hitching_post(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(7.5, 0.0, 7.0);
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.15, 1.6),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 0.8, 0.0)),
        ..default()
    });
    // Querarm
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.10, 1.4),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.4, 0.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // Ring
    commands.spawn(PbrBundle {
        mesh: torus_mesh(meshes, 0.18, 0.04),
        material: mats.metal_rusty.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.7, 1.4, 0.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}

fn torus_mesh(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, tube: f32) -> Handle<Mesh> {
    meshes.add(Mesh::from(bevy::math::primitives::Torus {
        minor_radius: tube,
        major_radius: radius,
    }))
}

// ---------- Bar / Saloon-Theke (sued-ost vom Brunnen) ----------
fn spawn_bar_counter(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    let base = Vec3::new(5.0, 0.0, -5.0);
    let rot = Quat::from_rotation_y(0.3);
    // Theke (lang, schmal)
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 0.9, 1.0, 4.0),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 0.5, 0.0))
            .with_rotation(rot),
        ..default()
    });
    // Thekenplatte
    commands.spawn(PbrBundle {
        mesh: box_mesh(meshes, 1.1, 0.08, 4.2),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(base + Vec3::new(0.0, 1.05, 0.0))
            .with_rotation(rot),
        ..default()
    });
    // 2 Barhocker
    for dz in [-1.2, 1.2] {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.30, 0.7),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(base + Vec3::new(0.9, 0.35, dz))
                .with_rotation(rot),
            ..default()
        });
    }
}

// ---------- Tische + Stuehle (Marktplatz) ----------
fn spawn_tables_and_chairs(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // 3 Tische mit je 2 Stuehlen
    let table_positions = [
        (Vec3::new(-5.0, 0.0, -2.0), 0.0),
        (Vec3::new(-7.0, 0.0, 2.0), 0.4),
        (Vec3::new(4.0, 0.0, 2.5), -0.2),
    ];
    for (pos, ry) in table_positions {
        let rot = Quat::from_rotation_y(ry);
        // Tischplatte
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 1.4, 0.08, 1.4),
            material: mats.wood_plank.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.0, 0.8, 0.0))
                .with_rotation(rot),
            ..default()
        });
        // 4 Beine
        for (dx, dz) in [(-0.6, -0.6), (0.6, -0.6), (-0.6, 0.6), (0.6, 0.6)] {
            commands.spawn(PbrBundle {
                mesh: cylinder_mesh(meshes, 0.05, 0.8),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new(dx, 0.4, dz))
                    .with_rotation(rot),
                ..default()
            });
        }
        // 2 Stuehle
        for dz in [-0.9, 0.9] {
            commands.spawn(PbrBundle {
                mesh: box_mesh(meshes, 0.4, 0.4, 0.4),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new(0.0, 0.2, dz))
                    .with_rotation(rot),
                ..default()
            });
            commands.spawn(PbrBundle {
                mesh: box_mesh(meshes, 0.4, 0.5, 0.08),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new(0.0, 0.6, dz + dz.signum() * 0.20))
                    .with_rotation(rot),
                ..default()
            });
        }
    }
}

// ---------- Krauter/Faesser/Bottiche (Cluster im Westen) ----------
fn spawn_clutter_west(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // 4 Krauter (3 hoch, 1 breit)
    let positions = [
        (Vec3::new(-3.0, 0.0, -8.0), 0.5, 1.2),
        (Vec3::new(-4.0, 0.0, -7.0), 0.5, 1.2),
        (Vec3::new(-3.5, 0.0, -9.0), 0.5, 1.2),
        (Vec3::new(-2.0, 0.0, -7.0), 0.5, 0.6),
    ];
    for (pos, r, h) in positions {
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, r, h),
            material: mats.barrel.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.0, h * 0.5, 0.0)),
            ..default()
        });
        // 2 Eisenringe
        for dy in [0.15, h - 0.20] {
            commands.spawn(PbrBundle {
                mesh: torus_mesh(meshes, r + 0.02, 0.025),
                material: mats.metal_rusty.clone(),
                transform: Transform::from_translation(pos + Vec3::new(0.0, dy, 0.0))
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ..default()
            });
        }
    }
    // 1 grosser Bottich
    commands.spawn(PbrBundle {
        mesh: cylinder_mesh(meshes, 0.8, 0.9),
        material: mats.wood_plank.clone(),
        transform: Transform::from_translation(Vec3::new(-5.0, 0.45, -8.5)),
        ..default()
    });
    // 3 kleine Falsche (Kugelform)
    for (i, off) in [(-0.5, -0.0), (0.5, -0.0), (0.0, 0.6)].iter().enumerate() {
        let x = -4.0 + off.0;
        let z = -10.0 + off.1;
        commands.spawn(PbrBundle {
            mesh: sphere_mesh(meshes, 0.18),
            material: if i == 0 { mats.glass.clone() } else { mats.barrel.clone() },
            transform: Transform::from_translation(Vec3::new(x, 0.18, z)),
            ..default()
        });
    }
}

// ---------- Sandhaufen + Kakteen (Sued-Ost) ----------
fn spawn_desert_corner(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // 3 Kakteen
    let cacti = [
        (Vec3::new(16.0, 0.0, 14.0), 1.4),
        (Vec3::new(17.5, 0.0, 16.0), 1.0),
        (Vec3::new(15.0, 0.0, 17.0), 0.8),
    ];
    for (pos, h) in cacti {
        // Stamm
        commands.spawn(PbrBundle {
            mesh: capsule_mesh(meshes, 0.18, h * 0.7),
            material: mats.cactus.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.0, h * 0.35, 0.0)),
            ..default()
        });
        // 2 Arme
        commands.spawn(PbrBundle {
            mesh: capsule_mesh(meshes, 0.10, 0.4),
            material: mats.cactus.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.25, h * 0.6, 0.0))
                .with_rotation(Quat::from_rotation_z(-0.7)),
            ..default()
        });
    }
    // Sandhaufen
    commands.spawn(PbrBundle {
        mesh: sphere_mesh(meshes, 1.2),
        material: mats.sand.clone(),
        transform: Transform::from_translation(Vec3::new(14.5, 0.2, 15.0))
            .with_rotation(Quat::from_rotation_x(0.0),
        ),
        ..default()
    });
    // Tumbleweed (Draht-Kugel, dunkel)
    commands.spawn(PbrBundle {
        mesh: sphere_mesh(meshes, 0.30),
        material: mats.wood_dark.clone(),
        transform: Transform::from_translation(Vec3::new(12.0, 0.30, 13.0)),
        ..default()
    });
}

// ---------- Markisen / Zelte (Marktplatz-Norden) ----------
fn spawn_market_stalls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // 3 Marktstaende mit gestreiften "Dachplanen"
    let stalls = [
        (Vec3::new(-4.0, 0.0, 6.0), 0.0, mats.red_cloth.clone(), mats.canvas.clone()),
        (Vec3::new(0.0, 0.0, 6.5), 0.1, mats.blue_cloth.clone(), mats.canvas.clone()),
        (Vec3::new(4.0, 0.0, 6.0), -0.05, mats.green_cloth.clone(), mats.canvas.clone()),
    ];
    for (pos, ry, cloth, _canvas) in stalls {
        let rot = Quat::from_rotation_y(ry);
        // Tisch
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 2.4, 0.08, 1.2),
            material: mats.wood_plank.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.0, 0.85, 0.0))
                .with_rotation(rot),
            ..default()
        });
        // 4 Beine
        for (dx, dz) in [(-1.0, -0.45), (1.0, -0.45), (-1.0, 0.45), (1.0, 0.45)] {
            commands.spawn(PbrBundle {
                mesh: cylinder_mesh(meshes, 0.05, 0.85),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new(dx, 0.42, dz))
                    .with_rotation(rot),
                ..default()
            });
        }
        // 4 Pfosten + Dachplane
        for (dx, dz) in [(-1.1, -0.55), (1.1, -0.55), (-1.1, 0.55), (1.1, 0.55)] {
            commands.spawn(PbrBundle {
                mesh: cylinder_mesh(meshes, 0.05, 2.0),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new(dx, 1.0, dz))
                    .with_rotation(rot),
                ..default()
            });
        }
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 2.6, 0.05, 1.3),
            material: cloth,
            transform: Transform::from_translation(pos + Vec3::new(0.0, 2.05, 0.0))
                .with_rotation(rot),
            ..default()
        });
        // Waren auf dem Tisch (z.B. Kaese-Block, Apfel-Kugel)
        commands.spawn(PbrBundle {
            mesh: box_mesh(meshes, 0.30, 0.18, 0.30),
            material: mats.canvas.clone(),
            transform: Transform::from_translation(pos + Vec3::new(-0.6, 1.0, 0.0))
                .with_rotation(rot),
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: sphere_mesh(meshes, 0.10),
            material: mats.red_cloth.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.3, 1.0, 0.2))
                .with_rotation(rot),
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: sphere_mesh(meshes, 0.10),
            material: mats.green_cloth.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.55, 1.0, -0.1))
                .with_rotation(rot),
            ..default()
        });
    }
}

// ---------- Zaeune (Perimeter, unterbrochen) ----------
fn spawn_fences(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // 3 Zaun-Stuecke an verschiedenen Stellen
    let fences = [
        (Vec3::new(-6.0, 0.0, 11.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(6.0, 0.0, 11.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(0.0, 0.0, -13.0), 0.0),
    ];
    for (pos, ry) in fences {
        // 3 Latten + 2 Querstreben
        for i in 0..3 {
            commands.spawn(PbrBundle {
                mesh: box_mesh(meshes, 0.08, 1.4, 0.08),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new((i as f32 - 1.0) * 1.0, 0.7, 0.0))
                    .with_rotation(Quat::from_rotation_y(ry)),
                ..default()
            });
        }
        for dz in [-0.4, 0.4] {
            commands.spawn(PbrBundle {
                mesh: box_mesh(meshes, 2.8, 0.08, 0.08),
                material: mats.wood_dark.clone(),
                transform: Transform::from_translation(pos + Vec3::new(0.0, 0.8 + dz * 0.4, dz))
                    .with_rotation(Quat::from_rotation_y(ry)),
                ..default()
            });
        }
    }
}

// ---------- Baeume und Buesche (Halloween-Flair, prozedural) ----------
//
// Die .fbx-Dateien aus Halloween_Kit_AssetQuest/ werden NICHT direkt geladen,
// weil Bevy keinen robusten FBX-Importer hat und ein toter Build schlimmer ist
// als stilistische Buesche aus Primaerformen. Wir bauen Baeume und Buesche aus
// Zylindern (Stamm) + Kugeln (Krone) zusammen, mit Herbstfaerbung.
fn spawn_trees_and_shrubs(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // 4 Baeume: 1 gruen, 1 orange, 2 Herbst-Mix. Asymmetrisch im Perimeter.
    let trees = [
        (Vec3::new(11.0, 0.0, 14.0), 3.0, mats.leaf_green.clone()),
        (Vec3::new(-12.0, 0.0, 12.0), 2.6, mats.leaf_orange.clone()),
        (Vec3::new(-18.0, 0.0, -8.0), 3.4, mats.leaf_green.clone()),
        (Vec3::new(15.0, 0.0, -12.0), 2.8, mats.leaf_orange.clone()),
    ];
    for (pos, height, leaf_mat) in trees {
        // Stamm
        commands.spawn(PbrBundle {
            mesh: cylinder_mesh(meshes, 0.18, height * 0.5),
            material: mats.wood_dark.clone(),
            transform: Transform::from_translation(pos + Vec3::new(0.0, height * 0.25, 0.0)),
            ..default()
        });
        // Krone: 3 Kugeln gestapelt
        let crown_color = leaf_mat;
        let crown_r = height * 0.45;
        for dy in [0.55_f32, 0.85, 1.15] {
            commands.spawn(PbrBundle {
                mesh: sphere_mesh(meshes, crown_r * (1.6_f32 - dy * 0.4).max(0.6)),
                material: crown_color.clone(),
                transform: Transform::from_translation(pos + Vec3::new(0.0, height * dy, 0.0)),
                ..default()
            });
        }
    }

    // 6 Straeucher: kleinere Kugeln, kein Stamm, bunt gemischt.
    let shrubs = [
        (Vec3::new(7.5, 0.0, 10.0), 0.7, mats.leaf_green.clone()),
        (Vec3::new(-7.5, 0.0, 9.0), 0.6, mats.leaf_orange.clone()),
        (Vec3::new(8.0, 0.0, -10.0), 0.8, mats.leaf_green.clone()),
        (Vec3::new(-9.0, 0.0, -10.0), 0.65, mats.leaf_orange.clone()),
        (Vec3::new(0.0, 0.0, 14.0), 0.75, mats.leaf_green.clone()),
        (Vec3::new(0.0, 0.0, -14.0), 0.7, mats.leaf_orange.clone()),
    ];
    for (pos, r, leaf_mat) in shrubs {
        commands.spawn(PbrBundle {
            mesh: sphere_mesh(meshes, r),
            material: leaf_mat,
            transform: Transform::from_translation(pos + Vec3::new(0.0, r * 0.6, 0.0)),
            ..default()
        });
    }
}

// ---------- Cover-Barrikaden (4 zentrale, KI-relevant) ----------
fn spawn_cover_barricades(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &WildWestMats,
) {
    // Asymmetrische Positionierung um den Brunnen herum.
    // Brunnen-Mantel hat Radius ~0.9m; Barrikaden 1.6m breit, 0.4m tief.
    // Min-Abstand zum Brunnen: 2.5m (Barrikaden-Aussenkante -> Brunnen-Mitte).
    // Spawn-Punkt des Spielers: (3, 1.6, 3). Mindestabstand 2m.
    let positions = [
        (Vec3::new(4.5, 0.0, 0.0), Quat::IDENTITY),
        (Vec3::new(-4.5, 0.0, 0.0), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, 0.0, 4.5), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        (Vec3::new(0.0, 0.0, -4.5), Quat::IDENTITY),
    ];
    for (pos, rot) in positions {
        let entity = commands
            .spawn(PbrBundle {
                mesh: box_mesh(meshes, 1.6, 1.0, 0.4),
                material: mats.wood_plank.clone(),
                transform: Transform::from_translation(pos + Vec3::new(0.0, 0.5, 0.0))
                    .with_rotation(rot),
                ..default()
            })
            .id();
        commands.entity(entity).insert(Cover {
            position: pos,
            half_extents: Vec3::new(0.8, 0.5, 0.2),
        });
    }
}

// ---------- Haupt-Setup ----------
fn setup_marketplace(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mats = build_materials(&mut materials);
    spawn_ground(&mut commands, &mut meshes, &mats);
    spawn_saloon(&mut commands, &mut meshes, &mats);
    spawn_sheriff(&mut commands, &mut meshes, &mats);
    spawn_stable(&mut commands, &mut meshes, &mats);
    spawn_small_house(&mut commands, &mut meshes, &mats);
    spawn_windmill(&mut commands, &mut meshes, &mats);
    spawn_water_tower(&mut commands, &mut meshes, &mats);
    spawn_well(&mut commands, &mut meshes, &mats);
    spawn_gallows(&mut commands, &mut meshes, &mats);
    spawn_stagecoach(&mut commands, &mut meshes, &mats);
    spawn_hitching_post(&mut commands, &mut meshes, &mats);
    spawn_bar_counter(&mut commands, &mut meshes, &mats);
    spawn_tables_and_chairs(&mut commands, &mut meshes, &mats);
    spawn_clutter_west(&mut commands, &mut meshes, &mats);
    spawn_desert_corner(&mut commands, &mut meshes, &mats);
    spawn_market_stalls(&mut commands, &mut meshes, &mats);
    spawn_fences(&mut commands, &mut meshes, &mats);
    spawn_cover_barricades(&mut commands, &mut meshes, &mats);
    spawn_trees_and_shrubs(&mut commands, &mut meshes, &mats);
}
