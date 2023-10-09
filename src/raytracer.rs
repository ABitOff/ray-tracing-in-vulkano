use vulkano::swapchain::PresentMode;

use crate::{
    vulkan::{
        application::{Application, ApplicationCreationError},
        WindowConfig,
    },
    UserSettings,
};

pub struct RayTracer {
    application: Application,
    _user_settings: UserSettings,
}

impl RayTracer {
    pub fn new(
        user_settings: UserSettings,
        window_config: WindowConfig,
        present_mode: PresentMode,
        visible_devices: &Option<Vec<u32>>,
    ) -> Result<RayTracer, ApplicationCreationError> {
        Ok(RayTracer {
            application: Application::new(window_config, present_mode, visible_devices)?,
            _user_settings: user_settings,
        })
    }

    pub fn run(self) {
        self.application.run();
    }
}
