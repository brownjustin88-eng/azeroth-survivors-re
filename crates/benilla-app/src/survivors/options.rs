//! Survivors Options overlay.
//!
//! Aethus's Linux build (`wenilla-rogue-app::screens::options`) opens from pause as
//! Resume / Options / Quit, with pages Interface, Action Bars, Combat, Key Bindings,
//! Nameplates, Audio, Graphics. This clone never vendored that crate; FrameXML's
//! `BenillaOptionsFrame` also stays dark because survivors sets `CaptureMode`. This
//! file is the Bevy panel that matches those listings and writes the CVars the
//! engine already owns.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use super::Phase;
use super::grade::GradeOn;
use super::hud::{BaseColor, DIM_GREY, GOLD_TONE, PauseRoot, centered_bundle, text_bundle};
use crate::cvars::{Cvars, SetOutcome};

const GOLD: Color = GOLD_TONE;
const DIM: Color = DIM_GREY;
const EDGE: Color = Color::srgb(0.55, 0.45, 0.25);
const BUTTON: Color = Color::srgba(0.18, 0.06, 0.04, 0.95);
const BUTTON_HOT: Color = Color::srgba(0.36, 0.12, 0.06, 0.98);
const PANEL: Color = Color::srgba(0.06, 0.03, 0.02, 0.94);
const TRACK: Color = Color::srgba(0.12, 0.08, 0.05, 0.95);
const FILL: Color = Color::srgb(0.85, 0.65, 0.12);
const TAB_OFF: Color = Color::srgba(0.12, 0.06, 0.04, 0.92);

#[derive(Component)]
pub(super) struct OptionsRoot;

#[derive(Resource, Default)]
pub(super) struct OptionsOpen(pub bool);

#[derive(Resource, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum Page {
    Interface,
    ActionBars,
    Combat,
    KeyBindings,
    Nameplates,
    Audio,
    #[default]
    Graphics,
}

impl Page {
    const ALL: [Page; 7] = [
        Page::Interface,
        Page::ActionBars,
        Page::Combat,
        Page::KeyBindings,
        Page::Nameplates,
        Page::Audio,
        Page::Graphics,
    ];

    fn label(self) -> &'static str {
        match self {
            Page::Interface => "Interface",
            Page::ActionBars => "Action Bars",
            Page::Combat => "Combat",
            Page::KeyBindings => "Key Bindings",
            Page::Nameplates => "Nameplates",
            Page::Audio => "Audio",
            Page::Graphics => "Graphics",
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(super) struct PagePanel(Page);

#[derive(Component, Clone, Copy)]
pub(super) struct TabBtn(Page);

#[derive(Component, Clone, Copy)]
pub(super) enum OptAction {
    Close,
    Tab(Page),
    Toggle(&'static str),
    InvertToggle(&'static str),
    Nudge {
        cvar: &'static str,
        delta: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Cycle {
        cvar: &'static str,
        values: &'static [&'static str],
    },
    Grade,
}

#[derive(Component, Clone, Copy)]
pub(super) struct Readout {
    cvar: &'static str,
    kind: ReadKind,
}

#[derive(Clone, Copy)]
pub(super) enum ReadKind {
    OnOff,
    InvertOnOff,
    Percent,
    ScalePercent,
    Int,
    Stops(&'static [&'static str]),
    DisplayMode,
    Gamma,
    Grade,
    Fps,
}

#[derive(Component)]
pub(super) struct FillBar {
    cvar: &'static str,
    min: f32,
    max: f32,
}

#[derive(Component)]
pub(super) struct CheckMark(&'static str);

#[derive(Component)]
pub(super) struct InvertCheck(&'static str);

#[derive(Component)]
pub(super) struct GradeMark;

pub(super) fn spawn(commands: &mut Commands, font: &Handle<Font>) {
    commands
        .spawn((
            OptionsRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
            GlobalZIndex(435),
            Visibility::Hidden,
            FocusPolicy::Block,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(720.0),
                    max_width: Val::Percent(94.0),
                    height: Val::Percent(86.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(EDGE),
            ))
            .with_children(|panel| {
                panel.spawn(centered_bundle(font, "Options", 32.0, GOLD));
                tabs(panel, font);
                panel
                    .spawn((
                        Interaction::default(),
                        FocusPolicy::Block,
                        ScrollPosition::default(),
                        Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            overflow: Overflow::scroll_y(),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        page(body, font, Page::Interface, interface_rows);
                        page(body, font, Page::ActionBars, action_bars);
                        page(body, font, Page::Combat, combat_rows);
                        page(body, font, Page::KeyBindings, key_bindings);
                        page(body, font, Page::Nameplates, nameplate_rows);
                        page(body, font, Page::Audio, audio_rows);
                        page(body, font, Page::Graphics, graphics_rows);
                    });
                panel.spawn(centered_bundle(
                    font,
                    "Changes are saved at once and apply to your next run.",
                    13.0,
                    DIM,
                ));
                panel
                    .spawn((
                        Button,
                        OptAction::Close,
                        BaseColor(BUTTON),
                        BackgroundColor(BUTTON),
                        BorderColor::all(GOLD),
                        Node {
                            width: Val::Px(220.0),
                            align_self: AlignSelf::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                    ))
                    .with_children(|b| {
                        b.spawn(text_bundle(font, "Close", 18.0, Color::WHITE));
                    });
            });
        });
}

fn tabs(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(6.0),
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|row| {
            for page in Page::ALL {
                row.spawn((
                    Button,
                    TabBtn(page),
                    OptAction::Tab(page),
                    BaseColor(TAB_OFF),
                    BackgroundColor(TAB_OFF),
                    BorderColor::all(EDGE),
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                ))
                .with_children(|b| {
                    b.spawn(text_bundle(font, page.label(), 13.0, Color::WHITE));
                });
            }
        });
}

fn page(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    which: Page,
    fill: fn(&mut ChildSpawnerCommands, &Handle<Font>),
) {
    parent
        .spawn((
            PagePanel(which),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(7.0),
                padding: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|col| fill(col, font));
}

fn graphics_rows(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    slider(
        col,
        font,
        "UI Scale",
        "uiScale",
        0.64,
        1.0,
        0.05,
        ReadKind::Percent,
    );
    slider(
        col,
        font,
        "Terrain Distance",
        "farclip",
        177.0,
        777.0,
        50.0,
        ReadKind::Int,
    );
    slider(
        col,
        font,
        "Environment Detail",
        "WorldDetail",
        0.0,
        2.0,
        1.0,
        ReadKind::Stops(&["Low", "Medium", "High"]),
    );
    slider(
        col,
        font,
        "Spell Detail",
        "spellEffectLevel",
        0.0,
        2.0,
        1.0,
        ReadKind::Stops(&["Low", "Medium", "High"]),
    );
    slider(
        col,
        font,
        "Brightness",
        "gamma",
        0.5,
        1.5,
        0.05,
        ReadKind::Gamma,
    );
    slider(
        col,
        font,
        "Weather Intensity",
        "weatherDensity",
        0.0,
        3.0,
        1.0,
        ReadKind::Stops(&["Minimal", "Low", "Medium", "Full"]),
    );
    if cfg!(not(target_os = "android")) {
        cycle(
            col,
            font,
            "Display Mode",
            "gxWindow",
            &["0", "1"],
            ReadKind::DisplayMode,
        );
    } else {
        note(
            col,
            font,
            "Display Mode follows the phone (Auto-rotate / fullUser).",
        );
    }
    checkbox(col, font, "Vertical Sync", "gxVSync");
    #[cfg(target_os = "android")]
    {
        cycle(
            col,
            font,
            "Frame Rate",
            "maxFps",
            &["30", "60", "120"],
            ReadKind::Fps,
        );
        note(
            col,
            font,
            "30 is the default. The S22 Ultra has a real 1440×3088 @ 30 Hz panel mode. 60 and 120 pick the matching modes. Fifo vsync stays on.",
        );
    }
    slider(
        col,
        font,
        "Render Scale",
        "renderScale",
        0.5,
        2.0,
        0.05,
        ReadKind::ScalePercent,
    );
    checkbox(col, font, "Full-Screen Glow", "ffxGlow");
    grade_row(col, font);
    #[cfg(not(target_os = "android"))]
    {
        checkbox(col, font, "FPS journal", "fpsJournal");
        note(
            col,
            font,
            "One row per second of wall / CPU / GPU pass time (opaque, static, glow, post, UI) in benilla-config/Diagnostics/fps-journal.csv. Same as /console fpsJournal 1.",
        );
    }
    #[cfg(target_os = "android")]
    note(
        col,
        font,
        "FPS journal records every phone run to benilla-config/Diagnostics/fps-journal.csv. GPU columns split a hitch into opaque / static / glow / post / UI. Pull the file with adb after a run.",
    );
}

fn audio_rows(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    checkbox(col, font, "Enable All Sound", "MasterSoundEffects");
    slider(
        col,
        font,
        "Master Volume",
        "MasterVolume",
        0.0,
        1.0,
        0.05,
        ReadKind::Percent,
    );
    slider(
        col,
        font,
        "Sound Volume",
        "SoundVolume",
        0.0,
        1.0,
        0.05,
        ReadKind::Percent,
    );
    slider(
        col,
        font,
        "Music Volume",
        "MusicVolume",
        0.0,
        1.0,
        0.05,
        ReadKind::Percent,
    );
    slider(
        col,
        font,
        "Ambience Volume",
        "AmbienceVolume",
        0.0,
        1.0,
        0.05,
        ReadKind::Percent,
    );
    checkbox(col, font, "Enable Music", "EnableMusic");
    checkbox(col, font, "Loop Music", "SoundZoneMusicNoDelay");
    checkbox(col, font, "Enable Ambience", "EnableAmbience");
    checkbox(col, font, "Enable Error Speech", "EnableErrorSpeech");
    checkbox(
        col,
        font,
        "Enable Sound in Background",
        "Sound_EnableSoundWhenGameIsInBG",
    );
}

fn interface_rows(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    checkbox(col, font, "Enhanced Tooltips", "UberTooltips");
    checkbox(col, font, "Status Bar Text", "statusBarText");
    checkbox(col, font, "Profanity Filter", "profanityFilter");
    checkbox(col, font, "Show Loading Screen Tips", "showGameTips");
    invert_checkbox(col, font, "Load out of date AddOns", "checkAddonVersion");
    note(
        col,
        font,
        "Target-of-target, cloak/helm, quest text and tutorials live on FrameXML globals; survivors leaves those dark.",
    );
}

fn combat_rows(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    checkbox(col, font, "Show Target Damage", "CombatDamage");
    checkbox(col, font, "Periodic Damage", "CombatLogPeriodicSpells");
    checkbox(col, font, "Pet Damage", "PetMeleeDamage");
    note(
        col,
        font,
        "Floating combat-text boxes (SHOW_COMBAT_TEXT and friends) are FrameXML uvars. Survivors hits still apply.",
    );
}

fn nameplate_rows(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    checkbox(col, font, "Player Names", "UnitNamePlayer");
    checkbox(col, font, "Player Guild Names", "UnitNamePlayerGuild");
    checkbox(col, font, "NPC Names", "UnitNameNPC");
    checkbox(col, font, "Show Own Name", "UnitNameOwn");
    note(
        col,
        font,
        "Enemy / friendly name plates are FrameXML API rows. Survivors draws its own HP plate.",
    );
}

fn action_bars(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    note(
        col,
        font,
        "1.12 multi-bars stay off in this mode (FrameXML is not loaded). The overlay is the bar:",
    );
    bind_line(col, font, "JUMP", "Space — hop");
    bind_line(
        col,
        font,
        "F",
        "class mobility (Charge / Blink / Sprint), learned at 5",
    );
    bind_line(col, font, "C", "camera Fixed / Orbit");
    bind_line(col, font, "II", "pause / resume (Esc)");
}

fn key_bindings(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    note(
        col,
        font,
        "Press a key, mouse button or wheel to bind to it is the 1.12 Key Bindings page. Survivors uses this map:",
    );
    bind_line(col, font, "Move", "WASD / left stick");
    bind_line(col, font, "Jump", "Space / JUMP");
    bind_line(col, font, "Mobility", "F");
    bind_line(col, font, "Camera", "C");
    bind_line(col, font, "Pause", "Esc / II");
    bind_line(col, font, "Color Grade", "G");
    bind_line(col, font, "Level-up", "1 / 2 / 3 or tap a card");
}

fn note(col: &mut ChildSpawnerCommands, font: &Handle<Font>, text: &str) {
    col.spawn((
        text_bundle(font, text, 13.0, DIM),
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            ..default()
        },
    ));
}

fn bind_line(col: &mut ChildSpawnerCommands, font: &Handle<Font>, key: &str, what: &str) {
    col.spawn(Node {
        width: Val::Percent(100.0),
        justify_content: JustifyContent::SpaceBetween,
        column_gap: Val::Px(12.0),
        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(3.0)),
        ..default()
    })
    .insert(BackgroundColor(TRACK))
    .insert(BorderColor::all(Color::srgba(0.35, 0.28, 0.16, 0.8)))
    .with_children(|row| {
        row.spawn(text_bundle(font, key, 15.0, GOLD));
        row.spawn((
            text_bundle(font, what, 14.0, Color::WHITE),
            Node {
                flex_grow: 1.0,
                ..default()
            },
        ));
    });
}

fn row_shell(col: &mut ChildSpawnerCommands) -> Entity {
    col.spawn((
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(32.0),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(TRACK),
        BorderColor::all(Color::srgba(0.35, 0.28, 0.16, 0.8)),
    ))
    .id()
}

fn checkbox(col: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &str, cvar: &'static str) {
    let id = row_shell(col);
    col.commands()
        .entity(id)
        .insert((Button, OptAction::Toggle(cvar), BaseColor(TRACK)))
        .with_children(|row| {
            row.spawn((
                CheckMark(cvar),
                text_bundle(font, "[ ]", 16.0, GOLD),
                Node {
                    width: Val::Px(28.0),
                    ..default()
                },
            ));
            row.spawn((
                text_bundle(font, label, 16.0, Color::WHITE),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((
                Readout {
                    cvar,
                    kind: ReadKind::OnOff,
                },
                text_bundle(font, "", 14.0, DIM),
            ));
        });
}

fn invert_checkbox(
    col: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    cvar: &'static str,
) {
    let id = row_shell(col);
    col.commands()
        .entity(id)
        .insert((Button, OptAction::InvertToggle(cvar), BaseColor(TRACK)))
        .with_children(|row| {
            row.spawn((
                InvertCheck(cvar),
                text_bundle(font, "[ ]", 16.0, GOLD),
                Node {
                    width: Val::Px(28.0),
                    ..default()
                },
            ));
            row.spawn((
                text_bundle(font, label, 16.0, Color::WHITE),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((
                Readout {
                    cvar,
                    kind: ReadKind::InvertOnOff,
                },
                text_bundle(font, "", 14.0, DIM),
            ));
        });
}

fn grade_row(col: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    let id = row_shell(col);
    col.commands()
        .entity(id)
        .insert((Button, OptAction::Grade, BaseColor(TRACK)))
        .with_children(|row| {
            row.spawn((
                GradeMark,
                text_bundle(font, "[ ]", 16.0, GOLD),
                Node {
                    width: Val::Px(28.0),
                    ..default()
                },
            ));
            row.spawn((
                text_bundle(font, "Color Grade (G)", 16.0, Color::WHITE),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((
                Readout {
                    cvar: "",
                    kind: ReadKind::Grade,
                },
                text_bundle(font, "", 14.0, DIM),
            ));
        });
}

fn slider(
    col: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    cvar: &'static str,
    min: f32,
    max: f32,
    step: f32,
    kind: ReadKind,
) {
    col.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(TRACK),
        BorderColor::all(Color::srgba(0.35, 0.28, 0.16, 0.8)),
    ))
    .with_children(|block| {
        block
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|head| {
                head.spawn((
                    text_bundle(font, label, 16.0, Color::WHITE),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
                head.spawn((Readout { cvar, kind }, text_bundle(font, "", 15.0, GOLD)));
            });
        block
            .spawn(Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|bar| {
                nudge(bar, font, "−", cvar, -step, min, max, step);
                bar.spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(10.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.03, 0.02, 0.95)),
                ))
                .with_children(|track| {
                    track.spawn((
                        FillBar { cvar, min, max },
                        Node {
                            width: Val::Percent(50.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(FILL),
                    ));
                });
                nudge(bar, font, "+", cvar, step, min, max, step);
            });
    });
}

fn nudge(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    cvar: &'static str,
    delta: f32,
    min: f32,
    max: f32,
    step: f32,
) {
    parent
        .spawn((
            Button,
            OptAction::Nudge {
                cvar,
                delta,
                min,
                max,
                step,
            },
            BaseColor(BUTTON),
            BackgroundColor(BUTTON),
            BorderColor::all(EDGE),
            Node {
                width: Val::Px(36.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn(centered_bundle(font, label, 18.0, Color::WHITE));
        });
}

fn cycle(
    col: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    cvar: &'static str,
    values: &'static [&'static str],
    kind: ReadKind,
) {
    let id = row_shell(col);
    col.commands()
        .entity(id)
        .insert((Button, OptAction::Cycle { cvar, values }, BaseColor(TRACK)))
        .with_children(|row| {
            row.spawn((
                text_bundle(font, label, 16.0, Color::WHITE),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((Readout { cvar, kind }, text_bundle(font, "", 15.0, GOLD)));
        });
}

pub(super) fn clicks(
    mut presses: Query<
        (&Interaction, &OptAction, &BaseColor, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut open: ResMut<OptionsOpen>,
    mut page: ResMut<Page>,
    mut cvars: ResMut<Cvars>,
    mut grade: ResMut<GradeOn>,
) {
    for (interaction, action, base, mut bg) in &mut presses {
        match interaction {
            Interaction::Hovered => bg.0 = BUTTON_HOT,
            Interaction::None => bg.0 = base.0,
            Interaction::Pressed => {
                bg.0 = BUTTON_HOT;
                match *action {
                    OptAction::Close => open.0 = false,
                    OptAction::Tab(p) => *page = p,
                    OptAction::Toggle(name) => {
                        let on = cvars.flag(name).unwrap_or(false);
                        write_cvar(&mut cvars, name, if on { "0" } else { "1" });
                    }
                    OptAction::InvertToggle(name) => {
                        let on = cvars.flag(name).unwrap_or(true);
                        write_cvar(&mut cvars, name, if on { "0" } else { "1" });
                    }
                    OptAction::Nudge {
                        cvar,
                        delta,
                        min,
                        max,
                        step,
                    } => {
                        let cur = cvars.num(cvar).unwrap_or(min);
                        let snapped = ((cur + delta) / step).round() * step;
                        let next = snapped.clamp(min, max);
                        write_cvar(&mut cvars, cvar, &fmt_num(next, step));
                    }
                    OptAction::Cycle { cvar, values } => {
                        let cur = cvars.get(cvar).unwrap_or(values[0]);
                        let i = values
                            .iter()
                            .position(|v| *v == cur)
                            .map(|i| (i + 1) % values.len())
                            .unwrap_or(0);
                        write_cvar(&mut cvars, cvar, values[i]);
                    }
                    OptAction::Grade => grade.0 = !grade.0,
                }
            }
        }
    }
}

fn write_cvar(cvars: &mut Cvars, name: &str, value: &str) {
    if cvars.set(name, value) == SetOutcome::Staged {
        let _ = cvars.commit_latched();
    }
}

fn fmt_num(v: f32, step: f32) -> String {
    if step >= 1.0 {
        format!("{}", v.round() as i32)
    } else {
        format!("{v:.2}")
    }
}

pub(super) fn refresh(
    cvars: Res<Cvars>,
    grade: Res<GradeOn>,
    mut texts: Query<(&Readout, &mut Text)>,
    mut marks: Query<(&CheckMark, &mut Text), Without<Readout>>,
    mut invert: Query<(&InvertCheck, &mut Text), (Without<Readout>, Without<CheckMark>)>,
    mut grade_mark: Query<
        &mut Text,
        (
            With<GradeMark>,
            Without<Readout>,
            Without<CheckMark>,
            Without<InvertCheck>,
        ),
    >,
    mut fills: Query<(&FillBar, &mut Node)>,
) {
    for (read, mut text) in &mut texts {
        text.0 = readout(&cvars, grade.0, *read);
    }
    for (mark, mut text) in &mut marks {
        let on = cvars.flag(mark.0).unwrap_or(false);
        text.0 = if on { "[X]".into() } else { "[ ]".into() };
    }
    for (mark, mut text) in &mut invert {
        let checking = cvars.flag(mark.0).unwrap_or(true);
        text.0 = if !checking {
            "[X]".into()
        } else {
            "[ ]".into()
        };
    }
    for mut text in &mut grade_mark {
        text.0 = if grade.0 { "[X]".into() } else { "[ ]".into() };
    }
    for (bar, mut node) in &mut fills {
        let v = cvars.num(bar.cvar).unwrap_or(bar.min);
        let t = ((v - bar.min) / (bar.max - bar.min)).clamp(0.0, 1.0);
        node.width = Val::Percent(t * 100.0);
    }
}

fn readout(cvars: &Cvars, grade_on: bool, read: Readout) -> String {
    match read.kind {
        ReadKind::OnOff => {
            if cvars.flag(read.cvar).unwrap_or(false) {
                "On".into()
            } else {
                "Off".into()
            }
        }
        ReadKind::InvertOnOff => {
            if cvars.flag(read.cvar).unwrap_or(true) {
                "Off".into()
            } else {
                "On".into()
            }
        }
        ReadKind::Percent => format!("{:.0}%", cvars.num(read.cvar).unwrap_or(0.0) * 100.0),
        ReadKind::ScalePercent => format!("{:.0}%", cvars.num(read.cvar).unwrap_or(1.0) * 100.0),
        ReadKind::Int => format!("{:.0}", cvars.num(read.cvar).unwrap_or(0.0)),
        ReadKind::Stops(names) => {
            let i = cvars.num(read.cvar).unwrap_or(0.0).round() as usize;
            names.get(i).copied().unwrap_or("?").to_string()
        }
        ReadKind::Fps => match cvars.get(read.cvar).unwrap_or("30") {
            "120" => "120 FPS".into(),
            "60" => "60 FPS".into(),
            _ => "30 FPS".into(),
        },
        ReadKind::DisplayMode => match cvars.get(read.cvar).unwrap_or("0") {
            "0" => "Windowed (Fullscreen)".into(),
            _ => "Windowed".into(),
        },
        ReadKind::Gamma => format!("{:.2}", cvars.num(read.cvar).unwrap_or(1.0)),
        ReadKind::Grade => {
            if grade_on {
                "On".into()
            } else {
                "Off".into()
            }
        }
    }
}

pub(super) fn sync_page(
    page: Res<Page>,
    mut panels: Query<(&PagePanel, &mut Visibility, &mut Node)>,
    mut tabs: Query<(&TabBtn, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (panel, mut vis, mut node) in &mut panels {
        let on = panel.0 == *page;
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
        let display = if on { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    for (tab, mut bg, mut border) in &mut tabs {
        let on = tab.0 == *page;
        bg.0 = if on { BUTTON } else { TAB_OFF };
        *border = BorderColor::all(if on { GOLD } else { EDGE });
    }
}

pub(super) fn show(
    phase: Res<State<Phase>>,
    open: Res<OptionsOpen>,
    mut roots: Query<(&mut Visibility, &mut Node), With<OptionsRoot>>,
) {
    let on = *phase.get() == Phase::Paused && open.0;
    for (mut vis, mut node) in &mut roots {
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
        let display = if on { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

/// Hide the pause rungs while Options is up, and drop the flag when the run leaves pause.
pub(super) fn gate_pause(
    phase: Res<State<Phase>>,
    mut open: ResMut<OptionsOpen>,
    mut pause: Query<(&mut Visibility, &mut Node), With<PauseRoot>>,
) {
    let paused = *phase.get() == Phase::Paused;
    if !paused && open.0 {
        open.0 = false;
    }
    let on = paused && !open.0;
    for (mut vis, mut node) in &mut pause {
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
        let display = if on { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}
