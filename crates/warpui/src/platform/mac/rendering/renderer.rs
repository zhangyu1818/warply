use crate::platform::mac::rendering::is_integrated_gpu;
use crate::platform::mac::window::WindowState;
use cocoa::base::id;
use warpui_core::rendering::{
    GPUBackend, GPUDeviceInfo, GPUDeviceType, GPUPowerPreference, OnGPUDeviceSelected,
};
use warpui_core::{Scene, fonts};

/// Trait to render the [`Scene`] onto the screen using the provided [`WindowState`].
pub trait Renderer {
    fn render(&mut self, scene: &Scene, window: &WindowState, font_cache: &fonts::Cache);

    fn resize(&mut self, window: &WindowState);
}

/// Set of available physical graphics devices that can be used to render.
#[allow(clippy::upper_case_acronyms)]
pub enum Device {
    #[allow(dead_code)]
    Metal(metal::Device),
}
impl Device {
    pub fn new(
        metal_device: metal::Device,
        _native_view: id,
        _native_window: id,
        _gpu_power_preference: GPUPowerPreference,
        on_gpu_device_info: Box<OnGPUDeviceSelected>,
    ) -> Self {
        let gpu_device_info = get_gpu_device_info(&metal_device);
        on_gpu_device_info(gpu_device_info);
        Device::Metal(metal_device)
    }
}

fn get_gpu_device_info(device: &metal::Device) -> GPUDeviceInfo {
    let device_type = if is_integrated_gpu(device) {
        GPUDeviceType::IntegratedGpu
    } else {
        GPUDeviceType::DiscreteGpu
    };
    GPUDeviceInfo {
        device_type,
        device_name: device.name().into(),
        driver_name: String::new(),
        driver_info: String::new(),
        backend: GPUBackend::Metal,
    }
}
