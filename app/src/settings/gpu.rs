use settings::{SupportedPlatforms, macros::define_settings_group};

define_settings_group!(GPUSettings, settings: [
   prefer_low_power_gpu: PreferLowPowerGPU {
       type: bool,
       default: false,
       supported_platforms: SupportedPlatforms::ALL,
       private: false,
       toml_path: "system.prefer_low_power_gpu",
       description: "Whether to prefer the integrated (low-power) GPU.",
   },
]);
