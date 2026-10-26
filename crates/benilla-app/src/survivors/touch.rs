//! On-screen stick and buttons for a phone. The stick is analog; Jump / F / C / pause
//! are taps that the existing key systems also honour.

use bevy::input::touch::TouchPhase;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;

use super::hero::Hero;
use super::Phase;
use crate::touch_pointer::CursorSkipTouch;

/// Screen-space stick: `x` right, `y` up, each in `-1..=1`.
#[derive(Resource, Default)]
pub(super) struct TouchPad {
    pub dir: Vec2,
    pub jump_just: bool,
    pub mobility_just: bool,
    pub pause_just: bool,
    pub cam_just: bool,
}

#[derive(Component)]
struct PadRoot;

#[derive(Component)]
struct StickLook;

#[derive(Component)]
struct StickKnob;

#[derive(Component, Clone, Copy)]
enum PadBtn {
    Jump,
    Mobility,
    Camera,
    Pause,
}

// Window points, then multiplied by UiScale (short-side / 768). Landscape on the S22
// wanted this stick twice the 32-radius pass; buttons stay the smaller size.
const STICK_R: f32 = 64.0;
const KNOB: f32 = 32.0;
const BTN_W: f32 = 28.0;
const BTN_H: f32 = 24.0;
const BTN_FONT: f32 = 9.0;
const PAD_GAP: f32 = 4.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<TouchPad>()
        .add_systems(Startup, spawn_pad)
        .add_systems(PreUpdate, (clear_just, drive_pad).chain())
        .add_systems(Update, (pad_visible, seat_knob));
}

fn spawn_pad(mut commands: Commands) {
    commands
        .spawn((
            PadRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(500),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                StickLook,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(18.0),
                    width: Val::Px(STICK_R * 2.0),
                    height: Val::Px(STICK_R * 2.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(Color::srgba(1.0, 0.82, 0.0, 0.45)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.28)),
                Pickable::IGNORE,
            ))
            .with_children(|stick| {
                stick.spawn((
                    StickKnob,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(STICK_R - KNOB * 0.5),
                        top: Val::Px(STICK_R - KNOB * 0.5),
                        width: Val::Px(KNOB),
                        height: Val::Px(KNOB),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 0.82, 0.0, 0.7)),
                    Pickable::IGNORE,
                ));
            });

            let col = Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(14.0),
                width: Val::Px(BTN_W),
                height: Val::Px(BTN_H * 4.0 + PAD_GAP * 3.0),
                flex_direction: FlexDirection::ColumnReverse,
                row_gap: Val::Px(PAD_GAP),
                ..default()
            };
            root.spawn((col, Pickable::IGNORE))
                .with_children(|col| {
                    pad_button(col, PadBtn::Jump, "JUMP");
                    pad_button(col, PadBtn::Mobility, "F");
                    pad_button(col, PadBtn::Camera, "C");
                    pad_button(col, PadBtn::Pause, "II");
                });
        });
}

fn pad_button(parent: &mut ChildSpawnerCommands, btn: PadBtn, label: &'static str) {
    parent
        .spawn((
            btn,
            Button,
            Node {
                width: Val::Px(BTN_W),
                height: Val::Px(BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(1.0, 0.82, 0.0, 0.5)),
            BackgroundColor(Color::srgba(0.12, 0.05, 0.02, 0.82)),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont {
                    font_size: BTN_FONT,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.55)),
                Pickable::IGNORE,
            ));
        });
}

fn clear_just(mut pad: ResMut<TouchPad>) {
    pad.jump_just = false;
    pad.mobility_just = false;
    pad.pause_just = false;
    pad.cam_just = false;
}

#[derive(Default)]
struct StickFinger {
    id: u64,
    origin: Vec2,
}

fn drive_pad(
    mut touches: MessageReader<TouchInput>,
    phase: Res<State<Phase>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Query<(&PadBtn, &Interaction), Changed<Interaction>>,
    mut pad: ResMut<TouchPad>,
    mut skip: ResMut<CursorSkipTouch>,
    mut stick_finger: Local<Option<StickFinger>>,
) {
    let playing = *phase.get() == Phase::Playing;
    if !playing {
        *stick_finger = None;
        skip.0 = None;
        pad.dir = Vec2::ZERO;
    }

    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn {
            PadBtn::Jump => pad.jump_just = true,
            PadBtn::Mobility => pad.mobility_just = true,
            PadBtn::Camera => pad.cam_just = true,
            PadBtn::Pause => pad.pause_just = true,
        }
    }

    if !playing {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let w = window.width();
    let h = window.height();

    for ev in touches.read() {
        if let TouchPhase::Started = ev.phase {
            // Left-bottom quadrant is the thumb stick; the right-side buttons keep the cursor.
            if stick_finger.is_none() && ev.position.x < w * 0.42 && ev.position.y > h * 0.52 {
                *stick_finger = Some(StickFinger {
                    id: ev.id,
                    origin: ev.position,
                });
                skip.0 = Some(ev.id);
            }
        }
        let Some(finger) = stick_finger.as_ref() else {
            continue;
        };
        if finger.id != ev.id {
            continue;
        }
        let delta = ev.position - finger.origin;
        let mag = delta.length();
        let n = if mag < 1.0 {
            Vec2::ZERO
        } else {
            delta * (mag.min(STICK_R) / mag) / STICK_R
        };
        // Bevy UI y grows down; stick up is negative y.
        pad.dir = Vec2::new(n.x, -n.y);
        if matches!(ev.phase, TouchPhase::Ended | TouchPhase::Canceled) {
            *stick_finger = None;
            skip.0 = None;
            pad.dir = Vec2::ZERO;
        }
    }
}

fn pad_visible(
    phase: Res<State<Phase>>,
    hero: Query<(), With<Hero>>,
    mut root: Query<(&mut Visibility, &mut Node), With<PadRoot>>,
) {
    let show = matches!(*phase.get(), Phase::Playing) && !hero.is_empty();
    for (mut vis, mut node) in &mut root {
        if show {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
}

fn seat_knob(
    pad: Res<TouchPad>,
    mut knob: Query<&mut Node, With<StickKnob>>,
) {
    let Ok(mut node) = knob.single_mut() else {
        return;
    };
    let half = KNOB * 0.5;
    let x = STICK_R - half + pad.dir.x * (STICK_R - 6.0);
    let y = STICK_R - half - pad.dir.y * (STICK_R - 6.0);
    node.left = Val::Px(x);
    node.top = Val::Px(y);
}
