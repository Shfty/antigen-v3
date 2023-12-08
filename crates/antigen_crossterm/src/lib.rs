//! [`crossterm`] event handling for `antigen`

mod events;
mod key_events;
mod macros;
mod mouse_events;
mod resize_events;

pub use events::*;
pub use key_events::*;
pub use macros::*;
pub use mouse_events::*;
pub use resize_events::*;

pub use crossterm;
