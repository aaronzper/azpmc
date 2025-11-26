use app::App;
use winit::event_loop::EventLoop;

/// A bunch of constant values used throughout the app
mod settings;
/// Data types and code for rendering
mod rendering;
/// Data types and code for the game world
mod world;
/// The core App struct which drives the game
mod app;
/// Vector math helpers
mod vectors;

fn main() -> anyhow::Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    println!("Bye!");

    Ok(())
}
