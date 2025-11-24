use std::sync::Arc;
use anyhow::Context;
use log::{info};
use wgpu::{Buffer, Device, Queue, RenderPassDescriptor, RenderPipeline, Sampler, Surface, SurfaceConfiguration, Texture, TextureView, util::DeviceExt};
use winit::window::Window;

use crate::{rendering::{camera::{Camera, CameraUniform}, light::Sun, mesh::Mesh, textures::{DEPTH_FORMAT, DepthTexture, create_diffue_bind_group}, vertex::Vertex}, settings};

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
    camera_bind_group: wgpu::BindGroup,
    sun_bind_group: wgpu::BindGroup,
    shadow_bind_group: wgpu::BindGroup,

    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    camera_buffer: Buffer,

    sun: Sun,
    sun_buffer: Buffer,

    depth_texture: DepthTexture,

    shadow_texture: Texture,
    shadow_view: TextureView,
    shadow_sampler: Sampler,
    shadow_pipeline: RenderPipeline,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        // --- INITIAL SETUP --- 
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

        let (texture_bind_group_layout, diffuse_bind_group) =
            create_diffue_bind_group(&device, &queue);

        // --- CAMERA ---
        let camera = Camera::new(config.width as f32, config.height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
            }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
            }
            ],
            label: Some("camera_bind_group"),
        });

        // --- SUNLIGHT ---
        let mut sun = Sun::new([1.0, -2.0, 0.5], [1.0, 1.0, 1.0]);
        let sun_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("The Sun"),
                contents: bytemuck::cast_slice(&[sun]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sun_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let sun_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sun_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sun_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // --- SHADOW MAPPING ---
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: settings::SHADOW_RES,
                height: settings::SHADOW_RES,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                   wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_view =
            shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let shadow_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Bind Group Layout"),
                entries: &[
                    // shadow texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // comparison sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            },
        );

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // --- MAIN PIPELINE ---
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/main.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Main Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &sun_bind_group_layout,
                    &shadow_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        // See https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#writing-the-shaders
        // for specifics
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
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
            depth_stencil: Some(wgpu::DepthStencilState { // 1.
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // When to discard a px
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
        });

        // --- SHADOW PIPELINE ---
        let shadow_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/shadow.wgsl"));

        let shadow_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[
                    &sun_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let shadow_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Pipeline"),
                layout: Some(&shadow_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shadow_shader,
                    entry_point: Some("vs_shadow"),
                    buffers: &[Vertex::desc_layout()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: None, // depth-only
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2,
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            }
        );

        Ok(Self {
            window,

            depth_texture: DepthTexture::new(&device, &config, "depth_texture"),

            surface,
            device,
            queue,
            config,
            surface_configured: false,

            render_pipeline,
            diffuse_bind_group,
            camera_bind_group,
            sun_bind_group,
            shadow_bind_group,

            camera,
            camera_uniform,
            camera_buffer,

            sun,
            sun_buffer,

            shadow_texture,
            shadow_view,
            shadow_sampler,
            shadow_pipeline,
        })
    }

    pub fn update(&mut self) {
        self.depth_texture = DepthTexture::new(&self.device, &self.config, "depth_texture");

        let center = [160.0, 70.0, 160.0];
        self.sun.update_view_proj(center, 453.5);
        self.queue.write_buffer(&self.sun_buffer, 0, bytemuck::cast_slice(&[self.sun]));

        self.camera.update_position();
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
            self.surface_configured = true;
        }
    }

    pub fn render(&mut self, meshes: &mut [&mut Mesh]) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.surface_configured {
            return Ok(());
        }

        let mut shadow_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Command Encoder"),
            });

        let mut shadow_pass = shadow_encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("Shadow Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
        }
        );

        shadow_pass.set_pipeline(&self.shadow_pipeline);
        shadow_pass.set_bind_group(0, &self.sun_bind_group, &[]);

        for mesh in &mut *meshes {
            if !mesh.are_buffers_set() {
                mesh.set_buffers(&self.device);
            }
            mesh.draw(&mut shadow_pass);
        }
        drop(shadow_pass);
        self.queue.submit(std::iter::once(shadow_encoder.finish()));

        // Provides a SurfaceTexture for us to render to
        let output = self.surface.get_current_texture()?;
        // Creates a TextureView with default settigns. Allows us to control how
        // render code interacts w/ the texture
        let view =
            output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Command Encoder"),
            });

        let mut render_pass = render_encoder.begin_render_pass(&RenderPassDescriptor {
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(2, &self.sun_bind_group, &[]);
        render_pass.set_bind_group(3, &self.shadow_bind_group, &[]);
        for mesh in meshes {
            if !mesh.are_buffers_set() {
                mesh.set_buffers(&self.device);
            }
            mesh.draw(&mut render_pass);
        }
        drop(render_pass); // Release borrow on the encoder

        self.queue.submit(std::iter::once(render_encoder.finish()));
        output.present();

        Ok(())
    }
}
