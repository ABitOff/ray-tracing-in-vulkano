use super::{window::Window, WindowConfig};
use std::sync::Arc;
use vulkano::{
    device::{
        physical::{PhysicalDeviceError, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceCreationError, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    image::ImageUsage,
    instance::{Instance, InstanceCreateInfo, InstanceCreationError},
    swapchain::{
        PresentMode, Surface, SurfaceCreationError, Swapchain, SwapchainCreateInfo,
        SwapchainCreationError,
    },
    LoadingError, VulkanError, VulkanLibrary,
};
use winit::{error::OsError, event_loop::EventLoop, window::WindowBuilder};

pub struct Application {
    present_mode: PresentMode,
    window: Window,
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    uniform_buffers: Vec<usize>,            // TODO
    depth_buffer: usize,                    // TODO
    graphics_pipeline: usize,               // TODO
    swapchain_frame_buffers: Vec<usize>,    // TODO
    command_pool: usize,                    // TODO
    command_buffers: usize,                 // TODO
    image_available_semaphores: Vec<usize>, // TODO
    render_finished_semaphores: Vec<usize>, // TODO
    in_flight_fences: Vec<usize>,           // TODO
    current_frame: usize,
}

impl Application {
    pub fn new(
        window_config: WindowConfig,
        present_mode: PresentMode,
        visible_devices: &Option<Vec<u32>>,
    ) -> Result<Application, ApplicationCreationError> {
        // mostly taken from vulkano examples.

        let library = VulkanLibrary::new().map_err(ApplicationCreationError::LoadingError)?;
        let required_extensions = vulkano_win::required_extensions(&library);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enumerate_portability: true,
                ..Default::default()
            },
        )
        .map_err(ApplicationCreationError::InstanceCreationError)?;

        let el = EventLoop::new();

        let fullscreen = if window_config.fullscreen {
            Some(winit::window::Fullscreen::Exclusive(
                el.primary_monitor()
                    .ok_or(ApplicationCreationError::NoPrimaryMonitorError)?
                    .video_modes()
                    .max()
                    .ok_or(ApplicationCreationError::NoVideoModeError)?,
            ))
        } else {
            None
        };
        let window = Arc::new(
            WindowBuilder::new()
                .with_resizable(window_config.resizable)
                .with_title(window_config.title.clone())
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    window_config.width,
                    window_config.height,
                ))
                .with_fullscreen(fullscreen)
                .build(&el)
                .map_err(ApplicationCreationError::OsError)?,
        );

        let surface = vulkano_win::create_surface_from_winit(window.clone(), instance.clone())
            .map_err(ApplicationCreationError::SurfaceCreationError)?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_ray_tracing_pipeline: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .map_err(ApplicationCreationError::VulkanError)?
            .filter(|p| {
                p.supported_extensions().contains(&device_extensions)
                    && p.properties().max_geometry_count.is_some_and(|c| c > 0)
                    && !visible_devices
                        .as_ref()
                        .is_some_and(|v| !v.contains(&p.properties().device_id))
            })
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .ok_or(ApplicationCreationError::NoPhysicalDevicesError)?;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .map_err(ApplicationCreationError::DeviceCreationError)?;
        let _queue = queues
            .next()
            .ok_or(ApplicationCreationError::NoQueuesCreatedError)?;

        let (swapchain, _images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .map_err(ApplicationCreationError::PhysicalDeviceError)?;

            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&surface, Default::default())
                    .map_err(ApplicationCreationError::PhysicalDeviceError)?[0]
                    .0,
            );

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: [window_config.width, window_config.height],
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .ok_or(ApplicationCreationError::NoSupportedCompositeAlphasError)?,
                    present_mode,
                    ..Default::default()
                },
            )
            .map_err(ApplicationCreationError::SwapchainCreationError)?
        };

        Ok(Application {
            present_mode,
            window: Window {
                config: window_config,
                window: window.clone(),
            },
            instance,
            surface,
            device,
            swapchain,
            uniform_buffers: Default::default(),
            depth_buffer: Default::default(),
            graphics_pipeline: Default::default(),
            swapchain_frame_buffers: Default::default(),
            command_pool: Default::default(),
            command_buffers: Default::default(),
            image_available_semaphores: Default::default(),
            render_finished_semaphores: Default::default(),
            in_flight_fences: Default::default(),
            current_frame: Default::default(),
        })
    }
}

#[derive(Debug)]
pub enum ApplicationCreationError {
    NoPrimaryMonitorError,
    NoVideoModeError,
    NoPhysicalDevicesError,
    NoQueuesCreatedError,
    NoSupportedCompositeAlphasError,
    LoadingError(LoadingError),
    InstanceCreationError(InstanceCreationError),
    OsError(OsError),
    SurfaceCreationError(SurfaceCreationError),
    VulkanError(VulkanError),
    DeviceCreationError(DeviceCreationError),
    PhysicalDeviceError(PhysicalDeviceError),
    SwapchainCreationError(SwapchainCreationError),
}
impl std::fmt::Display for ApplicationCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationCreationError::NoPrimaryMonitorError => {
                write!(f, "{:?}: Could not find a primary monitor.", self)
            }
            ApplicationCreationError::NoVideoModeError => {
                write!(
                    f,
                    "{:?}: Could not find a fullscreen video mode for the primary monitor.",
                    self
                )
            }
            ApplicationCreationError::NoPhysicalDevicesError => {
                write!(f, "{:?}: Could not find a physical device.", self)
            }
            ApplicationCreationError::NoQueuesCreatedError => {
                write!(
                    f,
                    "{:?}: Could not create any queues on the physical device.",
                    self
                )
            }
            ApplicationCreationError::NoSupportedCompositeAlphasError => {
                write!(
                    f,
                    "{:?}: Could not find any supported composite alphas for the surface.",
                    self
                )
            }
            ApplicationCreationError::LoadingError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::InstanceCreationError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::OsError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::SurfaceCreationError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::VulkanError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::DeviceCreationError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::PhysicalDeviceError(e) => std::fmt::Display::fmt(e, f),
            ApplicationCreationError::SwapchainCreationError(e) => std::fmt::Display::fmt(e, f),
        }
    }
}
impl std::error::Error for ApplicationCreationError {}