use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use wgpu::Queue;

#[derive(Debug)]
pub struct WgpuQueue {
    pub queue: Queue,
}

impl From<Queue> for WgpuQueue {
    fn from(queue: Queue) -> Self {
        WgpuQueue { queue }
    }
}

impl From<WgpuQueue> for Queue {
    fn from(queue: WgpuQueue) -> Self {
        queue.queue
    }
}

impl Display for WgpuQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WgpuQueue")
    }
}

impl Deref for WgpuQueue {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl DerefMut for WgpuQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}
