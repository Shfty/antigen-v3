use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use wgpu::Device;

#[derive(Debug)]
pub struct WgpuDevice {
    pub device: Device,
}

impl From<Device> for WgpuDevice {
    fn from(device: Device) -> Self {
        WgpuDevice { device }
    }
}

impl From<WgpuDevice> for Device {
    fn from(device: WgpuDevice) -> Self {
        device.device
    }
}

impl Display for WgpuDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WgpuDevice")
    }
}

impl Deref for WgpuDevice {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DerefMut for WgpuDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}
