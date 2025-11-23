use std::sync::Arc;
use anyhow::Context;
use log::{info};
use wgpu::{Device, Queue, RenderPassDescriptor, RenderPipeline, Surface, SurfaceConfiguration, util::DeviceExt};
use winit::window::Window;
use image::GenericImageView;

use crate::{rendering::vertex::Vertex, settings};

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
    diffuse_bind_group: wgpu::BindGroup,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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

        let diffuse_bytes = include_bytes!("../../assets/textures.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB, so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                // This is the same as with the SurfaceConfig. It
                // specifies what texture formats can be used to
                // create TextureViews for this texture. The base
                // texture format (Rgba8UnormSrgb in this case) is
                // always supported. Note that using a different
                // texture format is not supported on the WebGL2
                // backend.
                view_formats: &[],
            }
        );

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        // See https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#writing-the-shaders
        // for specifics
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc_layout()], // 2.
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

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Text Vertex Buffer"),
                contents: bytemuck::cast_slice(settings::TEST_MODEL_V),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(settings::TEST_MODEL_I),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Ok(Self {
            window,

            surface,
            device,
            queue,
            config,
            surface_configured: false,

            render_pipeline,
            diffuse_bind_group,

            vertex_buffer,
            index_buffer,
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
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..settings::TEST_MODEL_I.len() as u32, 0, 0..1);

        drop(render_pass); // Release borrow on the encoder

        self.queue.submit(std::iter::once(cmd_encoder.finish()));
        output.present();

        Ok(())
    }
}
