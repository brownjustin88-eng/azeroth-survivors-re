//! Finger → cursor, so glue clicks and Bevy UI see a pointer on a phone.
//!
//! `ui_focus_system` places `Hovered` from `Window::physical_cursor_position`, then
//! `Touches::first_pressed_position`. On lift the touch is already gone, so a press ends
//! in `None` and [`crate::glue::glue_clicks`] never fires. Holding the last finger on the
//! window cursor makes the release `Pressed → Hovered`, which is a click.

use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::TouchPhase;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Finger id whose events must not move the cursor (the virtual stick).
#[derive(Resource, Default)]
pub(crate) struct CursorSkipTouch(pub Option<u64>);

pub(crate) struct TouchPointerPlugin;

impl Plugin for TouchPointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorSkipTouch>().add_systems(
            PreUpdate,
            hold_touch_cursor.before(bevy::input::InputSystems),
        );
    }
}

#[derive(Default)]
struct Pinch(Option<f32>);

fn hold_touch_cursor(
    mut touches: MessageReader<TouchInput>,
    skip: Res<CursorSkipTouch>,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut mouse: MessageWriter<MouseButtonInput>,
    mut wheel: MessageWriter<MouseWheel>,
    mut down: Local<Option<u64>>,
    mut pinch: Local<Pinch>,
    mut live: Local<Vec<(u64, Vec2)>>,
) {
    let skip_id = skip.0;
    let Ok((window_e, mut window)) = windows.single_mut() else {
        return;
    };

    for ev in touches.read() {
        if skip_id == Some(ev.id) {
            if matches!(ev.phase, TouchPhase::Ended | TouchPhase::Canceled) {
                if *down == Some(ev.id) {
                    *down = None;
                }
                live.retain(|(id, _)| *id != ev.id);
            }
            continue;
        }
        match ev.phase {
            TouchPhase::Started => {
                live.retain(|(id, _)| *id != ev.id);
                live.push((ev.id, ev.position));
                window.set_cursor_position(Some(ev.position));
                if down.is_none() {
                    *down = Some(ev.id);
                    mouse.write(MouseButtonInput {
                        button: MouseButton::Left,
                        state: ButtonState::Pressed,
                        window: window_e,
                    });
                }
            }
            TouchPhase::Moved => {
                if let Some(slot) = live.iter_mut().find(|(id, _)| *id == ev.id) {
                    slot.1 = ev.position;
                } else {
                    live.push((ev.id, ev.position));
                }
                if *down == Some(ev.id) {
                    window.set_cursor_position(Some(ev.position));
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                live.retain(|(id, _)| *id != ev.id);
                window.set_cursor_position(Some(ev.position));
                if *down == Some(ev.id) {
                    mouse.write(MouseButtonInput {
                        button: MouseButton::Left,
                        state: ButtonState::Released,
                        window: window_e,
                    });
                    *down = None;
                }
            }
        }
    }

    if live.len() >= 2 {
        let d = live[0].1.distance(live[1].1);
        if let Some(prev) = pinch.0 {
            let delta = d - prev;
            if delta.abs() > 1.0 {
                // Fingers apart → zoom out (negative wheel, see `hero::place_camera`).
                wheel.write(MouseWheel {
                    unit: MouseScrollUnit::Pixel,
                    x: 0.0,
                    y: -delta,
                    window: window_e,
                });
            }
        }
        pinch.0 = Some(d);
    } else {
        pinch.0 = None;
    }
}
