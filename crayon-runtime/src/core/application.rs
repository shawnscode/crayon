use std::path::Path;
use std::sync::Arc;

use super::engine::Engine;
use super::arguments::Arguments;
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
    pub resources: resource::ResourceSystem,
}

impl Application {
    /// Creates empty `Application`.
    pub fn new() -> Result<Self> {
        let input = input::Input::new();
        let window = Arc::new(window::WindowBuilder::new().build(&input)?);
        let graphics = graphics::Graphics::new(window.clone())?;

        Ok(Application {
               input: input,
               window: window,
               graphics: graphics,
               engine: Engine::new(),
               resources: resource::ResourceSystem::new()?,
           })
    }

    /// Setup application from configuration.
    pub fn setup<T>(path: T) -> Result<Self>
        where T: AsRef<Path>
    {
        let args = Arguments::new(path)
            .chain_err(|| "failed to parse arguments.")?;

        let mut engine = Engine::new();
        if let Some(slice) = args.load_as_slice("Engine") {
            let v = slice.load_as_i32("MinFPS").unwrap_or(0) as u32;
            engine.set_min_fps(v);

            let v = slice.load_as_i32("MaxFPS").unwrap_or(0) as u32;
            engine.set_max_fps(v);

            let v = slice.load_as_i32("MaxInactiveFPS").unwrap_or(0) as u32;
            engine.set_max_inactive_fps(v);

            let v = slice.load_as_i32("TimeSmoothingStep").unwrap_or(0) as u32;
            engine.set_time_smoothing_step(v);
        }

        let mut wb = window::WindowBuilder::new();
        if let Some(slice) = args.load_as_slice("Window") {
            let name = slice
                .load_as_str("Title")
                .unwrap_or("Lemon3D - Application");
            let width = slice.load_as_i32("Width").unwrap_or(128) as u32;
            let height = slice.load_as_i32("Height").unwrap_or(128) as u32;
            wb.with_title(name.to_string())
                .with_dimensions(width, height);
        }

        let input = input::Input::new();
        let window = Arc::new(wb.build(&input)?);
        let graphics = graphics::Graphics::new(window.clone())?;

        Ok(Application {
               input: input,
               window: window,
               engine: engine,
               graphics: graphics,
               resources: resource::ResourceSystem::new()?,
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
        println!("Launch Lemon3D.");
        println!("PWD: {:?}", ::std::env::current_dir());

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