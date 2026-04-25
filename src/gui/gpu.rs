use std::num::NonZeroUsize;

use baseview::Window;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};

use crate::gui::platform::{to_wgpu_display_handle, to_wgpu_window_handle};

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub render_view: wgpu::TextureView,
    pub blitter: wgpu::util::TextureBlitter,
    pub renderer: Renderer,
}

impl Gpu {
    pub fn new(window: &mut Window, width: u32, height: u32) -> Option<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let raw_window_handle = to_wgpu_window_handle(window.raw_window_handle());
        let raw_display_handle = to_wgpu_display_handle(window.raw_display_handle());

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .ok()?
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
        }))
        .ok()?;

        let optional = wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: adapter.features() & optional,
            ..Default::default()
        }))
        .ok()?;

        let surface_format = surface
            .get_capabilities(&adapter)
            .formats
            .iter()
            .find(|&&f| matches!(f, wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm))
            .copied()?;

        let mut surface_config = surface.get_default_config(&adapter, width, height)?;
        surface_config.format = surface_format;
        surface.configure(&device, &surface_config);

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("vello_render_target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blitter = wgpu::util::TextureBlitter::new(&device, surface_format);

        let renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )
        .ok()?;

        Some(Self {
            device,
            queue,
            surface,
            render_view,
            blitter,
            renderer,
        })
    }

    pub fn render(&mut self, scene: &Scene, width: u32, height: u32) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };

        let surface_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_params = RenderParams {
            base_color: vello::peniko::Color::from_rgba8(28, 28, 30, 255),
            width,
            height,
            antialiasing_method: AaConfig::Msaa16,
        };

        let _ = self
            .renderer
            .render_to_texture(&self.device, &self.queue, scene, &self.render_view, &render_params);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.blitter.copy(&self.device, &mut encoder, &self.render_view, &surface_view);
        self.queue.submit([encoder.finish()]);

        frame.present();
    }
}
