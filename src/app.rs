use std::sync::Arc;
use cgmath::{InnerSpace, Vector2, Vector3, Zero};
use winit::{application::ApplicationHandler, event::{DeviceEvent, KeyEvent, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}};
use crate::{rendering::RenderState, settings::MOVE_SPEED, vectors::{replace_xz, xyz_to_xz}, world::{Coordinate, GameWorld}};

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

                    let player = self.world.player_mut();
                    let player_v = player.get_velocity();
                    let old_player_dir = xyz_to_xz(player_v);
                    let new_player_dir = 
                        xyz_to_xz(render_state.camera.get_direction()).normalize()
                        * old_player_dir.magnitude();
                    player.set_velocity(replace_xz(player_v, new_player_dir));
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
                // Calculate physics and load new chunks
                self.world.do_tick();
                self.world.update_chunks_to_player();

                // Update camera position to player's
                render_state.camera
                    .update_position(self.world.player_mut().get_precise_pos());

                // Render!
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
                if key_state.is_pressed() {
                    match code {
                        KeyCode::KeyQ => {
                            event_loop.exit();
                        }

                        KeyCode::Escape => {
                            render_state.window.set_cursor_visible(true);
                            render_state.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                            self.mouse_trapped = false;
                        }

                        KeyCode::KeyW => {
                            let player = self.world.player_mut();
                            let p_v_3d = player.get_velocity();
                            let mut p_v_2d = xyz_to_xz(p_v_3d);
                            let direction = if p_v_2d.magnitude().is_zero() {
                                xyz_to_xz(render_state.camera.get_direction())
                                    .normalize()
                            } else {
                                p_v_2d.normalize()
                            };

                            let movement_v = direction * MOVE_SPEED;
                            p_v_2d += movement_v;
                            player.set_velocity(replace_xz(p_v_3d, p_v_2d));
                        }

                        KeyCode::KeyA => {

                        }

                        KeyCode::KeyS => {

                        }

                        KeyCode::KeyD => {

                        }

                        KeyCode::Space => {

                        }

                        KeyCode::ControlLeft => {

                        }

                        _ => {},
                    };
                } else {
                    match code {
                        KeyCode::KeyW => {

                        }

                        KeyCode::KeyA => {

                        }

                        KeyCode::KeyS => {

                        }

                        KeyCode::KeyD => {

                        }

                        KeyCode::Space => {

                        }

                        KeyCode::ControlLeft => {

                        }

                        _ => {},
                    };
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
