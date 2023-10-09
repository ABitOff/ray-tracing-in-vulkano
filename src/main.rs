use raytracer::RayTracer;
use vulkano::swapchain::PresentMode;

mod raytracer;
mod vulkan;

struct Options {
    pub benchmark: bool,
    pub benchmark_next_scenes: bool,
    pub benchmark_max_time: u32,
    pub samples: u32,
    pub bounces: u32,
    pub max_samples: u32,
    pub scene_index: u32,
    pub visible_devices: Option<Vec<u32>>,
    pub width: u32,
    pub height: u32,
    pub present_mode: u32,
    pub fullscreen: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            benchmark: false,
            benchmark_next_scenes: false,
            benchmark_max_time: 60,
            samples: 8,
            bounces: 16,
            max_samples: 65_536,
            scene_index: 1,
            visible_devices: None,
            width: 1280,
            height: 720,
            present_mode: 2,
            fullscreen: false,
        }
    }
}

pub struct UserSettings {
    pub benchmark: bool,
    pub benchmark_next_scenes: bool,
    pub benchmark_max_time: u32,
    pub scene_index: usize,
    pub is_ray_traced: bool,
    pub accumulate_rays: bool,
    pub number_of_samples: u32,
    pub number_of_bounces: u32,
    pub max_number_of_samples: u32,
    pub field_of_view: f32,
    pub aperture: f32,
    pub focus_distance: f32,
    pub show_heatmap: bool,
    pub heatmap_scale: f32,
    pub show_settings: bool,
    pub show_overlay: bool,
}

impl UserSettings {
    pub const FOV_MIN: f32 = 10.0;
    pub const FOV_MAX: f32 = 90.0;

    pub fn requires_accumulation_reset(&self, prev: &UserSettings) -> bool {
        return self.is_ray_traced != prev.is_ray_traced
            || self.accumulate_rays != prev.accumulate_rays
            || self.number_of_bounces != prev.number_of_bounces
            || self.field_of_view != prev.field_of_view
            || self.aperture != prev.aperture
            || self.focus_distance != prev.focus_distance;
    }
}

impl From<&Options> for UserSettings {
    fn from(opts: &Options) -> Self {
        UserSettings {
            benchmark: opts.benchmark,
            benchmark_next_scenes: opts.benchmark_next_scenes,
            benchmark_max_time: opts.benchmark_max_time,
            scene_index: opts.scene_index as usize,
            is_ray_traced: true,
            accumulate_rays: true,
            number_of_samples: opts.samples,
            number_of_bounces: opts.bounces,
            max_number_of_samples: opts.max_samples,
            field_of_view: 0.0,
            aperture: 0.0,
            focus_distance: 0.0,
            show_heatmap: false,
            heatmap_scale: 1.5,
            show_settings: !opts.benchmark,
            show_overlay: true,
        }
    }
}

fn main() {
    let options = Options::default();
    let settings = UserSettings::from(&options);
    let window_config = vulkan::WindowConfig {
        title: "Vulkan Window".into(),
        width: options.width,
        height: options.height,
        cursor_disabled: options.benchmark && options.fullscreen,
        fullscreen: options.fullscreen,
        resizable: !options.fullscreen,
    };

    let application = match RayTracer::new(
        settings,
        window_config,
        match options.present_mode {
            0 => PresentMode::Immediate,
            1 => PresentMode::Mailbox,
            2 => PresentMode::Fifo,
            3 => PresentMode::FifoRelaxed,
            _ => panic!(),
        },
        &options.visible_devices,
    ) {
        Ok(rt) => rt,
        Err(e) => {
            let e_str = format!("{}", e).to_string();
            let e_str = e_str.replace("\n", "\n\t");
            eprintln!("Failed to create application:\n\t{}", e_str);
            return;
        }
    };

    print_vulkan_sdk_info();
    print_vulkan_instance_info(&application, options.benchmark);
    print_vulkan_layers_info(&application, options.benchmark);
    print_vulkan_devices(&application, &options.visible_devices);
    print_vulkan_swapchain_info(&application);

    application.run();
}

fn print_vulkan_sdk_info() {
    println!(
        "Vulkan SDK Header Version: {}",
        vulkano::Version::HEADER_VERSION
    );
    println!("");
}

fn print_vulkan_instance_info(app: &RayTracer, benchmark: bool) {
    if benchmark {
        return;
    }

    println!("Vulkan Instance Extensions:");

    println!("{:?}", app.application.instance.enabled_extensions());

    println!("");
}

fn print_vulkan_layers_info(app: &RayTracer, benchmark: bool) {
    if benchmark {
        return;
    }

    println!("Vulkan Instance Layers:");

    println!("{:?}", app.application.instance.enabled_layers());

    println!("");
}

fn print_vulkan_devices(app: &RayTracer, visible_devices: &Option<Vec<u32>>) {
    println!("Vulkan Devices:");

    match app.application.instance.enumerate_physical_devices() {
        Err(e) => {
            eprintln!(
                "Failed to enumerate physical devices. Cannot print devices... {}",
                e
            );
            return;
        }
        Ok(pds) => pds.for_each(|pd| {
            let props = pd.properties();

            if visible_devices.as_ref().map_or(false, |v| !v.contains(&props.device_id)) {
                return;
            }

            with_vendor_id_string(props.vendor_id, |vendor_id| {
                println!(
                    "- [{}] {} '{}' ({:?}; Vulkan: {}; Driver: {}, '{}' - {})",
                    props.device_id,
                    vendor_id,
                    props.device_name,
                    props.device_type,
                    props.api_version,
                    props
                        .driver_name
                        .as_ref()
                        .unwrap_or(&"Unnamed Driver".into()),
                    props
                        .driver_info
                        .as_ref()
                        .unwrap_or(&"No Driver Info".into()),
                    props.driver_version,
                );
            })
        }),
    }

    println!("");
}

fn print_vulkan_swapchain_info(app: &RayTracer) {
    println!("Swapchain:");
    println!("- image count: {}", app.application.swapchain.image_count());
    println!(
        "- present mode: {:?}",
        app.application.swapchain.present_mode()
    );
    println!("");
}

fn with_vendor_id_string(vendor_id: u32, f: impl FnOnce(&str) -> ()) {
    let s = match vendor_id {
        0x1002 => "AMD",
        0x1010 => "ImgTec",
        0x10DE => "NVIDIA",
        0x13B5 => "ARM",
        0x5143 => "Qualcomm",
        0x8086 => "INTEL",
        _ => "UnknownVendor",
    };
    f(s);
}
