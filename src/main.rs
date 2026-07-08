//! Wild West Paintball FPS - Haupteinstiegspunkt.
//!
//! Diese Datei enthaelt ausschliesslich das App-Skelett und die Plugin-Registrierung.
//! Die eigentliche Logik lebt in den modularen Plugins:
//!   * `world`     - Level-Aufbau, Sonnenuntergangs-Beleuchtung
//!   * `player`    - First-Person-Controller (Maus, WASD, Springen)
//!   * `weapon`    - Waffen-Inventar, Ballistik, Splatter-Decals
//!   * `ai`        - Bot-State-Machine, Patrouille, Deckung, Schiessen
//!   * `ui`        - HUD (FPS, Munition, Waffenname) + Tastatur-Liste
//!   * `screenshot` - Headless-Screenshot-Helper (nur wenn --screenshot CLI-Arg)

use bevy::prelude::*;

mod player;
mod weapon;
mod ai;
mod ui;
mod world;
mod screenshot;

fn main() {
    // Panics in Bevy-Systems fuehren normalerweise zu einem std::process::exit,
    // was das Spiel "einfach schliesst" ohne sichtbaren Fehler. Wir installieren
    // einen Custom-Panic-Hook, der den Panic-Inhalt nach stderr UND in eine
    // Log-Datei schreibt, damit der Spieler den Backtrace sehen kann.
    let log_path = std::env::temp_dir().join("wildwest_panic.log");
    std::panic::set_hook(Box::new(move |info| {
        // Versuche, die Panic-Payload als String zu lesen (klassisch
        // panic!("...") oder panic!("{var}"). Fuer Anything-anderes
        // (Box<dyn Any>) wird die Debug-Darstellung genommen.
        let payload_msg = if let Some(s) = info.payload().downcast_ref::<&'static str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        let backtrace = std::backtrace::Backtrace::force_capture();
        let msg = format!(
            "WILDWEST PANIC @ {}\nlocation: {:?}\npayload:  {}\n\nbacktrace:\n{}\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            info.location(),
            payload_msg,
            backtrace
        );
        eprint!("{msg}");
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, msg.as_bytes()));
    }));

    App::new()
        .add_plugins(DefaultPlugins)
        // HINWEIS: GLB-Loader funktioniert in dieser Build-Konfiguration
        // nicht zuverlaessig ("Could not find an asset loader matching"
        // fuer .glb-Dateien). Statt GLB zu laden, bauen wir die Waffen-
        // Viewmodels in `weapon.rs::spawn_weapon_entity` aus Primitives
        // (Cylinder fuer den Lauf, Cuboid fuer Body/Grip) - das passt
        // zum Wild-West-Stil und umgeht das Loader-Problem komplett.
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            weapon::WeaponPlugin,
            ai::AiPlugin,
            ui::UiPlugin,
        ))
        // ScreenshotPlugin ist nur aktiv, wenn --screenshot als CLI-Arg
        // uebergeben wurde. Ohne dieses Flag tut es nichts.
        .add_plugins(screenshot::ScreenshotPlugin)
        .run();
}
