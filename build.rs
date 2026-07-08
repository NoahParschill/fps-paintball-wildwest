//! Build-Script: kopiert den `assets/`-Ordner des Crates in
//! `target/<profile>/assets/`, damit das Spiel die GLB-Modelle auch
//! dann findet, wenn es per Doppelklick auf die Binary oder aus
//! `target/release/` heraus gestartet wird.
//!
//! Vorher: `cargo run --release` startet mit Working-Dir = Repo-Root
//! und findet die Assets, ein direkter Start der Binary
//! (`target/release/fps_paintball_wildwest`) startet mit Working-Dir
//! = Binary-Ordner und sucht vergeblich in `target/release/assets/`.
//!
//! Nachher: die Assets liegen automatisch neben der Binary.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_src = manifest_dir.join("assets");

    // OUT_DIR ist z. B. `target/release/build/<crate>-<hash>/out/`. Der
    // Profil-Ordner (target/release oder target/debug) ist der Parent
    // des Grossvaters. Wir nehmen den direkten Weg: PROFILE + TARGET_DIR
    // (Cargo setzt beide Umgebungsvariablen).
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let assets_dst = target_dir.join(&profile).join("assets");

    println!("cargo:rerun-if-changed={}", assets_src.display());
    if !assets_src.exists() {
        // Kein assets/-Ordner? Kein Kopieren noetig, aber wir wollen
        // spaetere Builds trotzdem neu bewerten.
        return;
    }

    if let Err(e) = copy_tree(&assets_src, &assets_dst) {
        // Wenn das Kopieren fehlschlaegt, ist das ein Build-Problem,
        // das der User sehen sollte. Wir geben einen klaren Fehler
        // aus statt panisch abzubrechen.
        eprintln!(
            "warning: build.rs konnte {} nicht nach {} kopieren: {}",
            assets_src.display(),
            assets_dst.display(),
            e
        );
    }
}

/// Rekursiv `src` -> `dst` kopieren. Existierende Dateien werden
/// ueberschrieben. Symlinks werden nicht unterstuetzt (brauchen wir
/// nicht).
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} ist kein Verzeichnis", src.display()),
        ));
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&from, &to)?;
        } else if file_type.is_file() {
            // Nur kopieren, wenn Quelle neuer ist als Ziel.
            let should_copy = match fs::metadata(&to) {
                Ok(dst_meta) => {
                    let src_meta = fs::metadata(&from)?;
                    src_meta.modified().ok() > dst_meta.modified().ok()
                }
                Err(_) => true,
            };
            if should_copy {
                fs::copy(&from, &to)?;
            }
        }
    }
    Ok(())
}
