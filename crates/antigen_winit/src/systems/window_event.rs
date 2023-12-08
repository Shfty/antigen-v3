use async_std::sync::Arc;
use deebs::{
    macros::CommonKeys, macros::Row, BorrowColumn, BorrowSingleton, CommonKeys, ReadCell,
    ReadSingleton, Row, Table, WriteCell,
};
use futures::StreamExt;
use std::ops::{Deref, DerefMut};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        DeviceId, ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta,
        TouchPhase, WindowEvent,
    },
};

use crate::{WinitMainEvent, WinitMainEvents, WinitWindow};

#[derive(Debug, Copy, Clone)]
pub enum WinitWindowEvent {
    Opened, // antigen_winit specific
    CloseRequested,
    Resized(PhysicalSize<u32>),
    ScaleFactorChanged {
        scale_factor: f64,
        new_inner_size: PhysicalSize<u32>,
    },
    MouseInput {
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
    },
    MouseWheel {
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
    CursorMoved {
        device_id: DeviceId,
        position: PhysicalPosition<f64>,
    },
    CursorLeft {
        device_id: DeviceId,
    },
    ModifiersChanged(ModifiersState),
    KeyboardInput {
        device_id: DeviceId,
        input: KeyboardInput,
        is_synthetic: bool,
    },
    ReceivedCharacter(char),
    Unimplemented,
}

impl<'a> From<&WindowEvent<'a>> for WinitWindowEvent {
    fn from(event: &WindowEvent<'a>) -> Self {
        match event {
            WindowEvent::CloseRequested => WinitWindowEvent::CloseRequested,
            WindowEvent::Resized(size) => WinitWindowEvent::Resized(*size),
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => WinitWindowEvent::ScaleFactorChanged {
                scale_factor: *scale_factor,
                new_inner_size: **new_inner_size,
            },
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => WinitWindowEvent::MouseInput {
                device_id: *device_id,
                state: *state,
                button: *button,
            },
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => WinitWindowEvent::MouseWheel {
                device_id: *device_id,
                delta: *delta,
                phase: *phase,
            },
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => WinitWindowEvent::CursorMoved {
                device_id: *device_id,
                position: *position,
            },
            WindowEvent::CursorLeft { device_id } => WinitWindowEvent::CursorLeft {
                device_id: *device_id,
            },
            WindowEvent::ModifiersChanged(state) => WinitWindowEvent::ModifiersChanged(*state),
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => WinitWindowEvent::KeyboardInput {
                device_id: *device_id,
                input: *input,
                is_synthetic: *is_synthetic,
            },
            WindowEvent::ReceivedCharacter(ch) => WinitWindowEvent::ReceivedCharacter(*ch),
            _ => WinitWindowEvent::Unimplemented,
            /*
            WindowEvent::Moved(_) => {}
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::ReceivedCharacter(_) => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::CursorEntered { device_id } => {}
            WindowEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            } => {}
            WindowEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => {}
            WindowEvent::Touch(_) => {}
            WindowEvent::ThemeChanged(_) => {}
            */
        }
    }
}

impl<'a> From<WindowEvent<'a>> for WinitWindowEvent {
    fn from(event: WindowEvent<'a>) -> Self {
        (&event).into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct WinitWindowEvents(Vec<WinitWindowEvent>);

impl Deref for WinitWindowEvents {
    type Target = Vec<WinitWindowEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WinitWindowEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(table)))]
pub async fn run_window_event_system<T>(table: Arc<T>)
where
    T: Table
        + BorrowSingleton<WinitMainEvents>
        + BorrowColumn<WinitWindow>
        + BorrowColumn<WinitWindowEvents>
        + Send
        + Sync,
{
    let main_events = ReadSingleton::<WinitMainEvents>::new(table.deref()).await;

    // Feed WinitWindowEvents sinks
    #[derive(Row, CommonKeys)]
    struct WindowEventSinkRow<'a> {
        window: ReadCell<'a, WinitWindow>,
        events: WriteCell<'a, WinitWindowEvents>,
    }

    let mut stream = WindowEventSinkRow::common_keys(table.deref()).await;
    while let Some(key) = stream.next().await {
        let WindowEventSinkRow { window, mut events } =
            WindowEventSinkRow::new(table.deref(), &key).await;

        if let WinitWindow::Ready { window_id, .. } = window.deref() {
            for event in main_events.iter() {
                if let WinitMainEvent::WindowEvent {
                    window_id: event_window_id,
                    event,
                } = event
                {
                    if window_id == event_window_id {
                        events.push(*event);
                    }
                }
            }
        }
    }
}
