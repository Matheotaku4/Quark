//! Entrées manette Linux et détection du mode Steam Deck.
//!
//! Le thread ne rend que les changements utiles : l'interface ne redessine pas
//! un écran à chaque rapport du contrôleur.

use gilrs::{Axis, Button, EventType, Gilrs};
use serde::Serialize;
use std::time::{Duration, Instant};

const DEADZONE: f32 = 0.45;
const REPEAT_DELAY: Duration = Duration::from_millis(350);
const REPEAT_INTERVAL: Duration = Duration::from_millis(110);
const POLL_INTERVAL: Duration = Duration::from_millis(8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PadAction {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Play,
    Favorite,
    Menu,
    TabPrev,
    TabNext,
}

#[derive(Debug, Clone, Serialize)]
pub struct PadEvent {
    pub action: PadAction,
    pub pressed: bool,
}

#[derive(Debug, Serialize)]
pub struct DeckStatus {
    pub steam_deck: bool,
    pub gamescope: bool,
    pub gamepad: bool,
}

pub fn map_button(button: Button) -> Option<PadAction> {
    match button {
        Button::South => Some(PadAction::Confirm),
        Button::East => Some(PadAction::Back),
        Button::West => Some(PadAction::Play),
        Button::North => Some(PadAction::Favorite),
        Button::DPadUp => Some(PadAction::Up),
        Button::DPadDown => Some(PadAction::Down),
        Button::DPadLeft => Some(PadAction::Left),
        Button::DPadRight => Some(PadAction::Right),
        Button::LeftTrigger | Button::LeftTrigger2 => Some(PadAction::TabPrev),
        Button::RightTrigger | Button::RightTrigger2 => Some(PadAction::TabNext),
        Button::Start | Button::Select => Some(PadAction::Menu),
        _ => None,
    }
}

pub fn axis_action(x: f32, y: f32) -> Option<PadAction> {
    if x.abs() < DEADZONE && y.abs() < DEADZONE {
        return None;
    }
    if x.abs() >= y.abs() {
        Some(if x > 0.0 {
            PadAction::Right
        } else {
            PadAction::Left
        })
    } else {
        Some(if y > 0.0 {
            PadAction::Down
        } else {
            PadAction::Up
        })
    }
}

fn dmi_product() -> Option<String> {
    std::fs::read_to_string("/sys/class/dmi/id/product_name")
        .ok()
        .map(|value| value.trim().to_string())
}

pub fn detect_steam_deck() -> bool {
    std::env::var("SteamDeck").ok().as_deref() == Some("1")
        || matches!(dmi_product().as_deref(), Some("Jupiter") | Some("Galileo"))
}

pub fn detect_gamescope() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|value| value.to_lowercase().contains("gamescope"))
        .unwrap_or(false)
        || std::env::var_os("GAMESCOPE_WAYLAND_DISPLAY").is_some()
}

fn gamepad_present() -> bool {
    Gilrs::new()
        .map(|gilrs| !gilrs.gamepads().collect::<Vec<_>>().is_empty())
        .unwrap_or(false)
}

#[tauri::command]
pub fn deck_status() -> DeckStatus {
    DeckStatus {
        steam_deck: detect_steam_deck(),
        gamescope: detect_gamescope(),
        gamepad: gamepad_present(),
    }
}

fn emit(app: &tauri::AppHandle, action: PadAction, pressed: bool) {
    use tauri::Emitter;
    let _ = app.emit("quark://gamepad", PadEvent { action, pressed });
}

fn is_direction(action: PadAction) -> bool {
    matches!(
        action,
        PadAction::Up | PadAction::Down | PadAction::Left | PadAction::Right
    )
}

pub fn spawn(app: tauri::AppHandle) {
    std::thread::Builder::new()
        .name("quark-gamepad".to_string())
        .spawn(move || run(app))
        .ok();
}

fn run(app: tauri::AppHandle) {
    let mut gilrs = match Gilrs::new() {
        Ok(gilrs) => gilrs,
        Err(_) => return,
    };
    let mut axis = (0.0_f32, 0.0_f32);
    let mut held: Option<(PadAction, Instant, Instant)> = None;

    loop {
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(button, _) => {
                    if let Some(action) = map_button(button) {
                        if is_direction(action) {
                            let now = Instant::now();
                            held = Some((action, now, now));
                        }
                        emit(&app, action, true);
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(action) = map_button(button) {
                        if held.map(|(current, _, _)| current) == Some(action) {
                            held = None;
                        }
                        emit(&app, action, false);
                    }
                }
                EventType::AxisChanged(axis_kind, value, _) => match axis_kind {
                    Axis::LeftStickX => axis.0 = value,
                    Axis::LeftStickY => axis.1 = value,
                    _ => {}
                },
                _ => {}
            }
        }

        let now = Instant::now();
        let next = axis_action(axis.0, axis.1);
        if next != held.map(|(action, _, _)| action) {
            if let Some(action) = next {
                held = Some((action, now, now));
                emit(&app, action, true);
            } else if held.is_some() {
                held = None;
            }
        } else if let Some((action, started, last)) = held {
            if now.duration_since(started) >= REPEAT_DELAY
                && now.duration_since(last) >= REPEAT_INTERVAL
            {
                held = Some((action, started, now));
                emit(&app, action, true);
            }
        }

        std::thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadzone_ignores_small_stick_noise() {
        assert_eq!(axis_action(0.2, -0.1), None);
    }

    #[test]
    fn dominant_axis_wins() {
        assert_eq!(axis_action(0.9, 0.2), Some(PadAction::Right));
        assert_eq!(axis_action(0.1, -0.8), Some(PadAction::Up));
        assert_eq!(axis_action(-0.9, 0.2), Some(PadAction::Left));
    }

    #[test]
    fn face_buttons_map_to_actions() {
        assert_eq!(map_button(Button::South), Some(PadAction::Confirm));
        assert_eq!(map_button(Button::East), Some(PadAction::Back));
        assert_eq!(map_button(Button::West), Some(PadAction::Play));
        assert_eq!(map_button(Button::North), Some(PadAction::Favorite));
    }
}
