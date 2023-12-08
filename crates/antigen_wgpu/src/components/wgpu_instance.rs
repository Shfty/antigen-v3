use wgpu::{BackendBit, Instance};

#[derive(Debug)]
pub enum WgpuInstance {
    Pending,
    Ready(Instance),
    Dropped,
}

impl Default for WgpuInstance {
    fn default() -> Self {
        WgpuInstance::Pending
    }
}

impl WgpuInstance {
    pub fn init(&mut self, backend_bit: BackendBit) {
        *self = WgpuInstance::Ready(Instance::new(backend_bit));
    }

    pub fn deinit(&mut self) {
        *self = WgpuInstance::Dropped;
    }
}
