use std::time::Instant;
use imgui::{FontSource, MouseCursor};
use imgui_wgpu::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use wgpu::{CommandEncoder, TextureView};
use winit::{event::Event, window::Window};
use crate::{settings, ui::state::UIState};

mod state;

pub struct UI {
    context: imgui::Context,
    platform: WinitPlatform,
    renderer: Option<Renderer>,
    last_frame: Instant,
    last_cursor: Option<MouseCursor>,

    pub state: UIState,
}

impl UI {
    pub fn new(
        win: &Window,
    ) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(context.io_mut(), win, HiDpiMode::Default);

        let dpi_factor = win.scale_factor();
        let font_sz = (settings::FONT_SZ * dpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / dpi_factor) as f32;
        
        context.set_ini_filename(None);
        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_sz,
                ..Default::default()
            }),
        }]);

        Self {
            context,
            platform,
            renderer: None,
            last_frame: Instant::now(),
            last_cursor: None,
            state: UIState::default(),
        }
    }

    pub fn draw(&mut self,
        win: &Window,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        let renderer = match &mut self.renderer {
            None => {
                let r = imgui_wgpu::Renderer::new(
                    &mut self.context,
                    device,
                    queue,
                    imgui_wgpu::RendererConfig { 
                        texture_format: surface_config.format,
                        ..Default::default()
                    }
                );

                self.renderer = Some(r);
                self.renderer.as_mut().unwrap()
            },
            Some(r) => r,
        };

        let now = Instant::now();
        self.context.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;

        self.platform
            .prepare_frame(self.context.io_mut(), win)
            .unwrap();

        let gui = self.context.new_frame();
        self.state.generate(gui);

        if self.last_cursor != gui.mouse_cursor() {
            self.last_cursor = gui.mouse_cursor();
            self.platform.prepare_render(gui, win);
        }
        
        let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderer.render(self.context.render(), queue, device, &mut ui_pass)
            .unwrap();
    }

    pub fn handle_event(&mut self, win: &Window, e: &Event<()>) {
        self.platform.handle_event(
            self.context.io_mut(), 
            win, 
            e,
        );
    }
}
