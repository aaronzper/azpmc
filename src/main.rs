use anyhow::Context;
use app::App;
use render_state::RenderState;
use winit::event_loop::EventLoop;

/// A bunch of constant values used throughout the app
mod settings;
mod render_state;
mod app;

fn main() -> anyhow::Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    println!("Bye!");

    Ok(())
}
