//! Shared entry for the desktop binary and the Android `cdylib`.

use benilla_app::BuildId;

/// The `bevy_main` proc-macro emits `android_main` and keeps this `main` for desktop.
#[bevy::prelude::bevy_main]
pub fn main() -> benilla_app::AppExit {
    #[cfg(target_os = "android")]
    bind_android_paths();
    benilla_app::run_survivors(BuildId {
        version: env!("CARGO_PKG_VERSION"),
        describe: env!("BENILLA_GIT_DESCRIBE"),
        sha: env!("BENILLA_GIT_SHA"),
        short: env!("BENILLA_GIT_SHORT"),
        date: env!("BENILLA_GIT_DATE"),
        profile: env!("BENILLA_PROFILE"),
        project_dir: env!("BENILLA_PROJECT_DIR"),
        ..Default::default()
    })
}

#[cfg(target_os = "android")]
fn bind_android_paths() {
    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let ext = app.external_data_path();
    let shared = std::path::Path::new("/storage/emulated/0/azeroth-survivors/Data");
    if std::env::var_os("WOW_DATA").is_none() {
        if shared.is_dir() {
            std::env::set_var("WOW_DATA", shared);
        } else if let Some(ext) = ext.as_ref() {
            std::env::set_var("WOW_DATA", ext.join("Data"));
        }
    }
    if std::env::var_os("BENILLA_HOME").is_none() {
        if let Some(ext) = ext {
            std::env::set_var("BENILLA_HOME", ext.join("benilla-config"));
        }
    }
    // Phone panel is 1440×3088 @ 120. MSAA 4× at that size is fill-rate waste on Adreno 730;
    // native WoW never ran it. gxMultisample can still be raised from config after a first run.
    if std::env::var_os("WOW_MSAA").is_none() {
        std::env::set_var("WOW_MSAA", "off");
    }
    // Color grade is a fullscreen extra pass; off on the phone unless the player asks.
    if std::env::var_os("WOW_GRADE").is_none() {
        std::env::set_var("WOW_GRADE", "0");
    }
    // Survivors has no `/console`, so the player-build FPS journal never started on the
    // phone. Pin the path for the whole run (CVar cannot turn it off once the env is set).
    if std::env::var_os("WOW_FPS_JOURNAL").is_none() {
        if let Ok(home) = std::env::var("BENILLA_HOME") {
            std::env::set_var(
                "WOW_FPS_JOURNAL",
                std::path::Path::new(&home)
                    .join("Diagnostics")
                    .join("fps-journal.csv"),
            );
        }
    }
}
