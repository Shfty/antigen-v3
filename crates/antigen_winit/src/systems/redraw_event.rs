use std::ops::{Deref, DerefMut};

use winit::window::WindowId;

#[derive(Debug, Copy, Clone)]
pub struct WinitRedrawEvent(pub WindowId);

#[derive(Debug, Default, Clone)]
pub struct WinitRedrawEvents(Vec<WinitRedrawEvent>);

impl From<WindowId> for WinitRedrawEvent {
    fn from(window_id: WindowId) -> Self {
        WinitRedrawEvent(window_id)
    }
}

impl From<WinitRedrawEvent> for WindowId {
    fn from(event: WinitRedrawEvent) -> Self {
        event.0
    }
}

impl Deref for WinitRedrawEvents {
    type Target = Vec<WinitRedrawEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WinitRedrawEvents {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
