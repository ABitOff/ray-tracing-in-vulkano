use std::sync::Arc;

use super::WindowConfig;
use winit::window::Window as WinitWindow;

pub struct Window {
    pub config: WindowConfig,
    pub window: Arc<WinitWindow>,
}
