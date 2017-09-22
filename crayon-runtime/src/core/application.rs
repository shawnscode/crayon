//! User-friendly facade for building applications.

use std::path::Path;
use std::sync::Arc;

use super::engine::Engine;
use super::settings::Settings;
use super::window;
use super::input;
use super::event;
use super::errors::*;

use graphics;
use resource;

/// Trait `ApplicationInstance` defines a number of event functions that get executed
/// in a pre-determined order.
pub trait ApplicationInstance {
    /// `ApplicationInstance::on_update` is called every frame. Its the main workhorse
    /// function for frame updates.
    fn on_update(&mut self, application: &mut Application) -> Result<()>;

    /// `ApplicationInstance::on_render` is called before we starts rendering the scene.
    fn on_render(&mut self, _: &mut Application) -> Result<()> {
        Ok(())
    }

    /// `ApplicationInstnace::on_post_render` is called after camera has rendered the scene.
    fn on_post_render(&mut self, _: &mut Application) -> Result<()> {
        Ok(())
    }
}

/// An `Application` is the root object of the game engne. It binds various subsystems in a
/// centeral place and takes care of trivial tasks like the execution order or life-time
/// management.
pub struct Application {
    pub input: input::Input,
    pub window: Arc<window::Window>,
    pub engine: Engine,
    pub graphics: graphics::GraphicsFrontend,
    pub resources: resource::ResourceFrontend,

    alive: bool,
}

impl Application {
    /// Setup application with default configurations.
    pub fn new() -> Result<Self> {
        Application::new_with(Settings::default())
    }

    /// Setup application from configurations load at path.
    pub fn setup<T>(path: T) -> Result<Self>
        where T: AsRef<Path>
    {
        let settings = Settings::load_from(path)
            .chain_err(|| "failed to parse arguments.")?;

        Application::new_with(settings)
    }

    /// Setup application with specified settings.
    pub fn new_with(settings: Settings) -> Result<Self> {
        let mut engine = Engine::new();
        engine.set_min_fps(settings.engine.min_fps);
        engine.set_max_fps(settings.engine.max_fps);
        engine.set_max_inactive_fps(settings.engine.max_inactive_fps);
        engine.set_time_smoothing_step(settings.engine.time_smooth_step);

        let mut wb = window::WindowBuilder::new();
        wb.with_title(settings.window.title.clone())
            .with_dimensions(settings.window.width, settings.window.height);

        let input = input::Input::new();
        let window = Arc::new(wb.build(&input)?);
        let graphics = graphics::GraphicsFrontend::new(window.clone())?;

        Ok(Application {
               input: input,
               window: window,
               engine: engine,
               graphics: graphics,
               resources: resource::ResourceFrontend::new()?,
               alive: true,
           })
    }

    /// Stop the whole application.
    pub fn stop(&mut self) {
        self.alive = false;
    }

    /// Run the main loop of `Application`, this will block the working
    /// thread until we finished.
    pub fn run(mut self, mut instance: &mut ApplicationInstance) -> Result<Self> {
        let dir = ::std::env::current_dir()?;
        println!("Run crayon-runtim with working directory {:?}.", dir);

        let mut events = Vec::new();
        'main: while self.alive {
            // Poll any possible events first.
            events.clear();

            self.input.run_one_frame(&mut events);
            for v in events.drain(..) {
                match v {
                    event::Event::Application(value) => {
                        match value {
                            event::ApplicationEvent::Closed => {
                                self.stop();
                                break 'main;
                            }
                            other => println!("Drop {:?}.", other),
                        };
                    }
                    event::Event::InputDevice(value) => self.input.process(value),
                    other => println!("Drop {:?}.", other),
                }
            }

            self.engine.run_one_frame();
            instance.on_update(&mut self)?;

            instance.on_render(&mut self)?;
            self.graphics.run_one_frame()?;
            instance.on_post_render(&mut self)?;
        }

        Ok(self)
    }
}