use vulkano::swapchain::PresentMode;

use crate::{
    vulkan::{application::Application, WindowConfig},
    UserSettings,
};

pub struct RayTracer {
    _application: Application,
    _user_settings: UserSettings,
}

impl RayTracer {
    pub fn new(
        user_settings: UserSettings,
        window_config: WindowConfig,
        present_mode: PresentMode,
        visible_devices: &Option<Vec<u32>>,
    ) -> RayTracer {
        RayTracer {
            _application: Application::new(window_config, present_mode, visible_devices).unwrap(),
            _user_settings: user_settings,
        }
    }
}
