use std::sync::Arc;
use cgmath::{InnerSpace, Vector2, Vector3, Zero};
use winit::{application::ApplicationHandler, event::{DeviceEvent, Event, KeyEvent, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{physics::Entity, rendering::RenderState, settings::MOVE_SPEED, ui::UI, vectors::{replace_xz, xyz_to_xz}, world::{Coordinate, GameWorld}};

/// Stores top-level info on the entire app
pub struct App {
    render_state: Option<RenderState>,
    mouse_trapped: bool,

    world: GameWorld,
    ui: Option<UI>,
}

impl App {
    pub fn new() -> Self {
        Self { 
            render_state: None,
            mouse_trapped: false,
            world: GameWorld::new(),
            ui: None,
        }
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_atts = Window::default_attributes()
            .with_title("AZP MC")
            .with_inner_size(winit::dpi::LogicalSize::new(1800, 1200));

        let win = Arc::new(event_loop.create_window(win_atts).unwrap());
        self.ui = Some(UI::new(&win));

        self.render_state = Some(
            pollster::block_on(RenderState::new(win)).unwrap()
        );
    }

    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        let render_state = match &mut self.render_state {
            Some(x) => x,
            None => return,
        };

        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_trapped {
                    let (dx, dy) = delta;
                    render_state.camera.update_direction(dx, dy);
                    let direction = render_state.camera.get_direction();

                    let player = self.world.player_mut();
                    let player_v = player.get_velocity();
                    let old_player_dir = xyz_to_xz(player_v);
                    let new_player_dir = 
                        xyz_to_xz(direction).normalize()
                        * old_player_dir.magnitude();
                    player.set_velocity(replace_xz(player_v, new_player_dir));
                }
            },

            _ => {},
        }
    }

    fn window_event(&mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let render_state = match &mut self.render_state {
            Some(x) => x,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) =>
                render_state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                // Calculate physics and load new chunks
                self.world.player_mut().facing =
                    render_state.camera.get_direction();
                self.world.do_tick();
                self.world.update_chunks_to_player();

                // Update camera position to player's
                render_state.camera
                    .update_position(self.world.player_mut().get_precise_pos());

                // Update UI overlay
                self.ui.as_mut().unwrap().state.update(&self.world);

                // Render!
                render_state.update(self.world.get_highlight());
                let mut meshes = self.world.get_meshes_mut();
                let ui = self.ui.as_mut().unwrap();
                match render_state.render(&mut meshes[..], ui) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = render_state.window.inner_size();
                        render_state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            },

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                    ..
            } => {
                if key_state.is_pressed() {
                    match code {
                        KeyCode::KeyQ => {
                            event_loop.exit();
                        }

                        KeyCode::Escape => {
                            render_state.window.set_cursor_visible(true);
                            render_state.window
                                .set_cursor_grab(winit::window::CursorGrabMode::None)
                                .unwrap();
                            self.mouse_trapped = false;
                        }

                        KeyCode::KeyW => {
                            self.world.player_mut().w_pressed = true;
                        }

                        KeyCode::KeyA => {
                            self.world.player_mut().a_pressed = true;
                        }

                        KeyCode::KeyS => {
                            self.world.player_mut().s_pressed = true;
                        }

                        KeyCode::KeyD => {
                            self.world.player_mut().d_pressed = true;
                        }

                        KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                            self.world.player_mut().sprint = true;
                        }

                        KeyCode::Space => {
                            self.world.player_mut().jump = true;
                        }

                        _ => {},
                    };
                } else {
                    match code {
                        KeyCode::KeyW => {
                            self.world.player_mut().w_pressed = false;
                        }

                        KeyCode::KeyA => {
                            self.world.player_mut().a_pressed = false;
                        }

                        KeyCode::KeyS => {
                            self.world.player_mut().s_pressed = false;
                        }

                        KeyCode::KeyD => {
                            self.world.player_mut().d_pressed = false;
                        }

                        KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                            self.world.player_mut().sprint = false;
                        }

                        _ => {},
                    };
                }
            },

            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() && button == MouseButton::Left {
                    if self.mouse_trapped {
                        self.world.destroy_block();
                    }

                    render_state.window
                        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                        .unwrap();
                    render_state.window.set_cursor_visible(false);
                    self.mouse_trapped = true;
                }
            }

            _ => {},
        }

        if let Some(ui) = &mut self.ui {
            ui.handle_event(
                &render_state.window,
                &Event::WindowEvent { window_id, event }
            );
        }
    }
}
