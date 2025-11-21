use std::sync::Arc;
use anyhow::Context;
use log::{error, info};
use wgpu::{Device, Queue, RenderPassDescriptor, RenderPipeline, Surface, SurfaceConfiguration};
use winit::window::Window;

/// Stores state of the window and rendering
pub struct RenderState {
    /// The game window
    pub window: Arc<Window>,

    /// The part of the window that we draw to
    surface: Surface<'static>,
    /// Handle to the GPU
    device: Device,
    /// GPU command queue
    queue: Queue,
    /// Defines how the surface creates its underlying textures
    config: SurfaceConfiguration,
    surface_configured: bool,

    render_pipeline: RenderPipeline,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;

        // handle to the GPU
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            // If we wanna favor battery life or performance
            power_preference: wgpu::PowerPreference::default(),
            // Find an adapter that works for our surface
            compatible_surface: Some(&surface),
            // Find an adapter that works on all hardware, if needed (off here)
            force_fallback_adapter: false,
        }).await.context("Couldn't find suitable GPU")?;

        info!("Using GPU \"{}\"", adapter.get_info().name);

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            // Extra features we want
            required_features: wgpu::Features::empty(),
            // Non-stable extra features we want
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            // Limits certain types of resources for us to create
            required_limits: wgpu::Limits::default(),
            // Preferred memory allocation strategy
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        }).await.context("Failed to get rendering device & queue")?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|x| x.is_srgb())
            .copied()
            .context("Couldn't find sRGB rendering surface")?;

        let config = SurfaceConfiguration {
            // What are textures used for? Here, writing to screen.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // How are they stores on GPU?
            format: surface_format,
            // In pixels
            width: size.width,
            height: size.height,
            // How to sync surface with display. Here, we choose Vsync.
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            // Usable `TextureFormat`s when doing textures
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // default
        };

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        Ok(Self {
            window,

            surface,
            device,
            queue,
            config,
            surface_configured: false,

            render_pipeline,
        })
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
            self.surface_configured = true;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.surface_configured {
            return Ok(());
        }

        // Provides a SurfaceTexture for us to render to
        let output = self.surface.get_current_texture()?;
        // Creates a TextureView with default settigns. Allows us to control how
        // render code interacts w/ the texture
        let view =
            output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut cmd_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Command Encoder"),
            });

        let mut render_pass = cmd_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                // What texture to save this to
                view: &view,
                // Texture to receive resolved output if multisampling
                resolve_target: None,
                // See docs
                depth_slice: None,
                // What to do with the colors
                ops: wgpu::Operations {
                    // What to do with previous frame's colors (clear & replace)
                    load: wgpu::LoadOp::Clear(crate::settings::SKY_COLOR),
                    // What to do with these frame's colors
                    store: wgpu::StoreOp::Store,
                }
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);

        drop(render_pass); // Release borrow on the encoder

        self.queue.submit(std::iter::once(cmd_encoder.finish()));
        output.present();

        Ok(())
    }
}
