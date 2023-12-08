//! [`egui`] integration for `antigen`

//pub use egui;

mod egui_render_pass;
mod egui_user_interface;
mod widgets;

pub use egui_render_pass::*;
pub use egui_user_interface::*;
pub use widgets::*;

#[cfg(feature = "clipboard")]
use clipboard::ClipboardProvider;

use antigen_rendering::RedrawFlag;
use antigen_winit::{
    winit::event::{ElementState, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode},
    WinitWindow, WinitWindowEvent, WinitWindowEvents, WinitWindows,
};
use async_std::sync::Arc;
use deebs::{
    macros::{CommonKeys, Row},
    BorrowColumn, BorrowSingleton, CommonKeys, ReadCell, ReadSingleton, Row, Table, WriteCell,
};
use egui::{pos2, vec2, Rect};
use futures::StreamExt;
use std::ops::Deref;

/// Translates winit to egui modifier keys.
#[inline]
fn winit_to_egui_modifiers(modifiers: ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        #[cfg(target_os = "macos")]
        mac_cmd: modifiers.logo(),
        #[cfg(target_os = "macos")]
        command: modifiers.logo(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: modifiers.ctrl(),
    }
}

/// Translates winit to egui keycodes.
#[inline]
fn winit_to_egui_key_code(key: VirtualKeyCode) -> Option<egui::Key> {
    Some(match key {
        VirtualKeyCode::Escape => egui::Key::Escape,
        VirtualKeyCode::Insert => egui::Key::Insert,
        VirtualKeyCode::Home => egui::Key::Home,
        VirtualKeyCode::Delete => egui::Key::Delete,
        VirtualKeyCode::End => egui::Key::End,
        VirtualKeyCode::PageDown => egui::Key::PageDown,
        VirtualKeyCode::PageUp => egui::Key::PageUp,
        VirtualKeyCode::Left => egui::Key::ArrowLeft,
        VirtualKeyCode::Up => egui::Key::ArrowUp,
        VirtualKeyCode::Right => egui::Key::ArrowRight,
        VirtualKeyCode::Down => egui::Key::ArrowDown,
        VirtualKeyCode::Back => egui::Key::Backspace,
        VirtualKeyCode::Return => egui::Key::Enter,
        VirtualKeyCode::Tab => egui::Key::Tab,
        VirtualKeyCode::Space => egui::Key::Space,

        VirtualKeyCode::A => egui::Key::A,
        VirtualKeyCode::K => egui::Key::K,
        VirtualKeyCode::U => egui::Key::U,
        VirtualKeyCode::W => egui::Key::W,
        VirtualKeyCode::Z => egui::Key::Z,

        _ => {
            return None;
        }
    })
}

/// We only want printable characters and ignore all special keys.
#[inline]
fn is_printable(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

#[derive(Row, CommonKeys)]
struct HandleEventsRow<'a, T, F>
where
    T: 'static,
    F: UserInterface + Send + Sync + 'static,
{
    window: Option<ReadCell<'a, WinitWindow>>,
    redraw_flag: Option<WriteCell<'a, RedrawFlag>>,
    winit_window_events: WriteCell<'a, WinitWindowEvents>,
    egui_user_interface: WriteCell<'a, EguiUserInterface<T, F>>,
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_window_event_system<F, T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WinitWindows>
        + BorrowColumn<RedrawFlag>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitWindowEvents>
        + BorrowColumn<EguiUserInterface<T, F>>
        + Send
        + Sync
        + 'static,
    F: UserInterface + Send + Sync + 'static,
{
    let windows = ReadSingleton::<WinitWindows>::new(table.deref()).await;

    let mut stream = HandleEventsRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let HandleEventsRow {
            window,
            redraw_flag,
            mut winit_window_events,
            mut egui_user_interface,
        } = HandleEventsRow::<T, F>::new(table.deref(), &key).await;

        let has_events = !winit_window_events.is_empty();

        for event in winit_window_events.drain(..) {
            let is_ctrl = egui_user_interface.modifiers.ctrl();
            let is_logo = egui_user_interface.modifiers.logo();
            let scale_factor = egui_user_interface.scale_factor;

            match event {
                WinitWindowEvent::Opened => {
                    if let Some(window) = window.as_ref() {
                        if let WinitWindow::Ready { window_id, .. } = window.deref() {
                            let window = windows.get(window_id).expect("Invalid Window ID.");

                            egui_user_interface.scale_factor = window.scale_factor() as f32;

                            let size = window.inner_size();
                            egui_user_interface.screen_rect = Rect::from_min_size(
                                Default::default(),
                                vec2(size.width as f32, size.height as f32) / scale_factor as f32,
                            );
                        }
                    }
                }
                WinitWindowEvent::Resized(size) => {
                    egui_user_interface.screen_rect = Rect::from_min_size(
                        Default::default(),
                        vec2(size.width as f32, size.height as f32)
                            / egui_user_interface.scale_factor,
                    );
                }
                WinitWindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    egui_user_interface.scale_factor = scale_factor as f32;

                    egui_user_interface.screen_rect = Rect::from_min_size(
                        Default::default(),
                        vec2(new_inner_size.width as f32, new_inner_size.height as f32)
                            / scale_factor as f32,
                    );
                }
                WinitWindowEvent::MouseInput { button, state, .. } => {
                    let pointer_position = egui_user_interface.pointer_position;
                    let modifiers = winit_to_egui_modifiers(egui_user_interface.modifiers);
                    egui_user_interface
                        .raw_input
                        .events
                        .push(egui::Event::PointerButton {
                            pos: pointer_position,
                            button: match button {
                                MouseButton::Left => egui::PointerButton::Primary,
                                MouseButton::Right => egui::PointerButton::Secondary,
                                MouseButton::Middle => egui::PointerButton::Middle,
                                MouseButton::Other(_) => unreachable!(),
                            },
                            pressed: state == ElementState::Pressed,
                            modifiers,
                        });
                }
                WinitWindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            let line_height = 24.0; // TODO as in egui_glium
                            egui_user_interface.raw_input.scroll_delta = vec2(x, y) * line_height;
                        }
                        MouseScrollDelta::PixelDelta(delta) => {
                            // Actually point delta
                            egui_user_interface.raw_input.scroll_delta =
                                vec2(delta.x as f32, delta.y as f32);
                        }
                    }
                }
                WinitWindowEvent::CursorMoved { position, .. } => {
                    let pointer_position = pos2(
                        position.x as f32 / scale_factor as f32,
                        position.y as f32 / scale_factor as f32,
                    );

                    egui_user_interface.pointer_position = pointer_position;
                    egui_user_interface
                        .raw_input
                        .events
                        .push(egui::Event::PointerMoved(pointer_position))
                }
                WinitWindowEvent::CursorLeft { .. } => egui_user_interface
                    .raw_input
                    .events
                    .push(egui::Event::PointerGone),
                WinitWindowEvent::ModifiersChanged(state) => egui_user_interface.modifiers = state,
                WinitWindowEvent::KeyboardInput { input, .. } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        let pressed = input.state == ElementState::Pressed;

                        if pressed {
                            if is_ctrl && virtual_keycode == VirtualKeyCode::C {
                                egui_user_interface.raw_input.events.push(egui::Event::Copy)
                            } else if is_ctrl && virtual_keycode == VirtualKeyCode::X {
                                egui_user_interface.raw_input.events.push(egui::Event::Cut)
                            } else if is_ctrl && virtual_keycode == VirtualKeyCode::V {
                                #[cfg(feature = "clipboard")]
                                if let Ok(contents) = egui_user_interface.clipboard.get_contents() {
                                    egui_user_interface
                                        .raw_input
                                        .events
                                        .push(egui::Event::Text(contents))
                                }
                            } else if let Some(key) = winit_to_egui_key_code(virtual_keycode) {
                                let modifiers =
                                    winit_to_egui_modifiers(egui_user_interface.modifiers);

                                egui_user_interface.raw_input.events.push(egui::Event::Key {
                                    key,
                                    pressed: input.state == ElementState::Pressed,
                                    modifiers,
                                });
                            }
                        }
                    }
                }
                WinitWindowEvent::ReceivedCharacter(ch) => {
                    if is_printable(ch) && !is_ctrl && !is_logo {
                        egui_user_interface
                            .raw_input
                            .events
                            .push(egui::Event::Text(ch.to_string()));
                    }
                }
                _ => {}
            }

            egui_user_interface.pointer_exclusive =
                captures_event(&egui_user_interface.context, event);
        }

        if let Some(mut redraw_flag) = redraw_flag {
            if has_events | egui_user_interface.output.needs_repaint {
                **redraw_flag = true;
            }
        }
    }
}
