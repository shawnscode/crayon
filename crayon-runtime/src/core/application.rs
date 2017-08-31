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

/// User-friendly facade for building applications.
pub struct Application {
    pub input: input::Input,
    pub window: Arc<window::Window>,
    pub engine: Engine,
    pub graphics: graphics::Graphics,
    pub resources: resource::ResourceFrontend,
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
        let graphics = graphics::Graphics::new(window.clone())?;

        Ok(Application {
               input: input,
               window: window,
               engine: engine,
               graphics: graphics,
               resources: resource::ResourceFrontend::new()?,
           })
    }


    /// Perform custom logics after engine initialization.
    pub fn perform<F>(mut self, mut closure: F) -> Self
        where F: FnMut(&mut Application)
    {
        closure(&mut self);
        self
    }

    /// Run the main loop of `Application`, this will block the working
    /// thread until we finished.
    pub fn run<F>(mut self, mut closure: F) -> Self
        where F: FnMut(&mut Application) -> bool
    {
        println!("Launch crayon-runtim with working directory {:?}.",
                 ::std::env::current_dir());
        let mut exec = true;
        let mut events = Vec::new();
        'main: while exec {
            // Poll any possible events first.
            events.clear();
            self.input.run_one_frame(&mut events);

            for v in events.drain(..) {
                match v {
                    event::Event::Application(value) => {
                        match value {
                            event::ApplicationEvent::Closed => break 'main,
                            other => println!("Drop {:?}.", other),
                        };
                    }
                    event::Event::InputDevice(value) => self.input.process(value),
                    other => println!("Drop {:?}.", other),
                }
            }

            self.engine.run_one_frame();
            self.graphics.run_one_frame().unwrap();
            exec = closure(&mut self);
        }
        self
    }
}