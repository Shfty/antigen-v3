use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use winit::{
    dpi::Size,
    window::{Window, WindowBuilder, WindowId},
};

#[derive(Debug)]
pub enum WinitWindow {
    Pending(WindowDescriptor),
    Ready {
        window_id: WindowId,
        window_desc: WindowDescriptor,
    },
    Closed,
}

impl Display for WinitWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            WinitWindow::Pending(_) => f.write_str("Pending"),
            WinitWindow::Ready { .. } => f.write_str("Ready"),
            WinitWindow::Closed => f.write_str("Closed"),
        }
    }
}

#[cfg(feature = "egui")]
impl egui::Widget for &WinitWindow {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}

#[derive(Debug, Default, Clone)]
pub struct WindowDescriptor {
    pub inner_size: Option<Size>,
    pub min_inner_size: Option<Size>,
    pub max_inner_size: Option<Size>,
    pub resizable: Option<bool>,
    pub title: Option<String>,
    pub fullscreen: (),
    pub maximized: Option<bool>,
    pub visible: Option<bool>,
    pub transparent: Option<bool>,
    pub decorations: Option<bool>,
    pub always_on_top: Option<bool>,
    pub window_icon: (),
}

impl From<&WindowDescriptor> for WindowBuilder {
    fn from(window_desc: &WindowDescriptor) -> Self {
        let WindowDescriptor {
            inner_size,
            min_inner_size,
            max_inner_size,
            resizable,
            title,
            fullscreen: _,
            maximized,
            visible,
            transparent,
            decorations,
            always_on_top,
            window_icon: _,
        } = window_desc;

        let builder = WindowBuilder::new();

        let builder = if let Some(inner_size) = inner_size {
            builder.with_inner_size(*inner_size)
        } else {
            builder
        };

        let builder = if let Some(min_inner_size) = min_inner_size {
            builder.with_min_inner_size(*min_inner_size)
        } else {
            builder
        };

        let builder = if let Some(max_inner_size) = max_inner_size {
            builder.with_max_inner_size(*max_inner_size)
        } else {
            builder
        };

        let builder = if let Some(resizable) = resizable {
            builder.with_resizable(*resizable)
        } else {
            builder
        };

        let builder = if let Some(title) = title {
            builder.with_title(title)
        } else {
            builder
        };

        let builder = if let Some(maximized) = maximized {
            builder.with_maximized(*maximized)
        } else {
            builder
        };

        let builder = if let Some(visible) = visible {
            builder.with_visible(*visible)
        } else {
            builder
        };

        let builder = if let Some(transparent) = transparent {
            builder.with_transparent(*transparent)
        } else {
            builder
        };

        let builder = if let Some(decorations) = decorations {
            builder.with_decorations(*decorations)
        } else {
            builder
        };

        if let Some(always_on_top) = always_on_top {
            builder.with_always_on_top(*always_on_top)
        } else {
            builder
        }
    }
}

#[derive(Debug, Default)]
pub struct WinitWindows(BTreeMap<WindowId, Window>);

impl Deref for WinitWindows {
    type Target = BTreeMap<WindowId, Window>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WinitWindows {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
