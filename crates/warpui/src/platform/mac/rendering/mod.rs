mod metal;
mod renderer;
mod renderer_manager;

pub use self::metal::is_integrated_gpu;
pub use renderer::{Device, Renderer};
pub use renderer_manager::RendererManager;

/// Returns `true` if a low power GPU is available for rendering. Typically, this is true for
/// machines with two GPUs -- a dedicated discrete high-performance GPU and a lower power
/// integrated GPU.
pub fn is_low_power_gpu_available() -> bool {
    let devices = ::metal::Device::all();
    let gpu_count = devices.len();
    gpu_count > 1 && devices.iter().any(metal::is_integrated_gpu)
}
