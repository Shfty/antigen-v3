use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use winit::{event::Event, window::WindowId};

use crate::WinitWindowEvent;

#[derive(Debug, Copy, Clone)]
pub enum WinitMainEvent {
    WindowEvent {
        window_id: WindowId,
        event: WinitWindowEvent,
    },
    Unimplemented,
}

impl<'a> From<&Event<'a, ()>> for WinitMainEvent {
    fn from(event: &Event<'a, ()>) -> Self {
        match event {
            //Event::NewEvents(_) => {}
            Event::WindowEvent { window_id, event } => WinitMainEvent::WindowEvent {
                window_id: *window_id,
                event: event.into(),
            },
            //Event::DeviceEvent { device_id, event } => {}
            //Event::UserEvent(_) => {}
            //Event::MainEventsCleared => {}
            _ => WinitMainEvent::Unimplemented,
            /*
            Event::Suspended => {}
            Event::Resumed => {}
            Event::LoopDestroyed => {}
            Event::RedrawRequested(window_id) => {},
            Event::RedrawEventsCleared => {}
            */
        }
    }
}

impl<'a> From<Event<'a, ()>> for WinitMainEvent {
    fn from(event: Event<'a, ()>) -> Self {
        (&event).into()
    }
}

#[derive(Debug, Default)]
pub struct WinitMainEvents(Vec<WinitMainEvent>);

impl Deref for WinitMainEvents {
    type Target = Vec<WinitMainEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WinitMainEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for WinitMainEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Winit Events")
    }
}
