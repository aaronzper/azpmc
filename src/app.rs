use std::sync::Arc;
use winit::{application::ApplicationHandler, event::{DeviceEvent, KeyEvent, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{rendering::RenderState, world::GameWorld};

/// Stores top-level info on the entire app
pub struct App {
    render_state: Option<RenderState>,
    mouse_trapped: bool,

    world: GameWorld,
}

impl App {
    pub fn new() -> Self {
        Self { 
            render_state: None,
            mouse_trapped: false,
            world: GameWorld::new(),
        }
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_atts = Window::default_attributes()
            .with_title("AZP MC")
            .with_inner_size(winit::dpi::LogicalSize::new(1800, 1200));

        let win = Arc::new(event_loop.create_window(win_atts).unwrap());

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
                }
            },

            _ => {},
        }
    }

    fn window_event(&mut self,
        event_loop: &ActiveEventLoop,
        _win_id: WindowId,
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
                render_state.update();

                let mut meshes = self.world.get_meshes_mut();
                match render_state.render(&mut meshes[..]) {
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
                if let Some(state) = &mut self.render_state {
                    if key_state.is_pressed() {
                        match code {
                            KeyCode::KeyQ => {
                                event_loop.exit();
                            }

                            KeyCode::Escape => {
                                state.window.set_cursor_visible(true);
                                state.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                                self.mouse_trapped = false;
                            }

                            KeyCode::KeyW => {
                                state.camera.controller.is_forward_pressed = true;
                            }

                            KeyCode::KeyA => {
                                state.camera.controller.is_left_pressed = true;
                            }

                            KeyCode::KeyS => {
                                state.camera.controller.is_backward_pressed = true;
                            }

                            KeyCode::KeyD => {
                                state.camera.controller.is_right_pressed = true;
                            }

                            KeyCode::Space => {
                                state.camera.controller.is_up_pressed = true;
                            }

                            KeyCode::ControlLeft => {
                                state.camera.controller.is_down_pressed = true;
                            }

                            _ => {},
                        };
                    } else {
                        match code {
                            KeyCode::KeyW => {
                                state.camera.controller.is_forward_pressed = false;
                            }

                            KeyCode::KeyA => {
                                state.camera.controller.is_left_pressed = false;
                            }

                            KeyCode::KeyS => {
                                state.camera.controller.is_backward_pressed = false;
                            }

                            KeyCode::KeyD => {
                                state.camera.controller.is_right_pressed = false;
                            }

                            KeyCode::Space => {
                                state.camera.controller.is_up_pressed = false;
                            }

                            KeyCode::ControlLeft => {
                                state.camera.controller.is_down_pressed = false;
                            }

                            _ => {},
                        };
                    }
                }
            },

            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() && button == MouseButton::Left {
                    if let Some(result_state) = &self.render_state {
                        result_state.window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
                        result_state.window.set_cursor_visible(false);
                        self.mouse_trapped = true;
                    }
                }
            }

            _ => {},
        }
    }
}
