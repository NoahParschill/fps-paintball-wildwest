//! `screenshot` - Headless-Rendering-Helper.

use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::Window;

#[derive(Resource, Debug, Default, Clone)]
pub struct ScreenshotConfig {
    pub path: Option<String>,
    pub frames: u32,
}

impl ScreenshotConfig {
    pub fn from_args() -> Self {
        let mut cfg = Self {
            path: None,
            frames: 90,
        };
        let mut args = std::env::args().skip(1);
        while let Some(a) = args.next() {
            if a == "--screenshot" {
                cfg.path = args.next();
            } else if a == "--frames" {
                if let Some(n) = args.next() {
                    cfg.frames = n.parse().unwrap_or(90);
                }
            }
        }
        cfg
    }
}

#[derive(Resource, Debug, Default)]
struct FrameCounter {
    frames_seen: u32,
    shot_triggered: bool,
    shot_done_frames: u32,
}

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        let cfg = ScreenshotConfig::from_args();
        if cfg.path.is_some() {
            app.insert_resource(cfg)
                .init_resource::<FrameCounter>()
                .add_systems(Update, screenshot_system);
        }
    }
}

fn screenshot_system(
    cfg: Res<ScreenshotConfig>,
    mut counter: ResMut<FrameCounter>,
    windows: Query<Entity, With<Window>>,
    mut manager: ResMut<ScreenshotManager>,
    mut app_exit: EventWriter<AppExit>,
) {
    if counter.shot_triggered {
        counter.shot_done_frames += 1;
        if counter.shot_done_frames > 30 {
            app_exit.send(AppExit::Success);
        }
        return;
    }
    counter.frames_seen += 1;
    if counter.frames_seen < cfg.frames {
        return;
    }
    let Some(path) = cfg.path.clone() else {
        return;
    };
    let Ok(window_entity) = windows.get_single() else {
        return;
    };
    info!("Aufnahme Screenshot -> {path}");

    // Callback: schreibt asynchron auf Platte. Fehler werden geloggt,
    // nicht paniced (anders als die eingebaute save_screenshot_to_disk).
    let result = manager.take_screenshot(window_entity, move |img| {
        let dyn_img = match img.try_into_dynamic() {
            Ok(d) => d,
            Err(e) => {
                error!("Cannot convert screenshot to dynamic image: {e}");
                return;
            }
        };
        let rgb = dyn_img.to_rgb8();
        match rgb.save(&path) {
            Ok(_) => info!("Screenshot saved to {path}"),
            Err(e) => error!("Cannot save screenshot to {path}: {e}"),
        }
    });
    match result {
        Ok(_) => counter.shot_triggered = true,
        Err(e) => {
            error!("take_screenshot fehlgeschlagen: {e:?}");
            counter.shot_triggered = true;
        }
    }
}
