use std::path::Path;
use super::engine::Engine;
use super::arguments::Arguments;
use super::window;
use super::input;
use super::errors::*;

use graphics;
use resource;

/// User-friendly facade for building applications.
pub struct Application {
    pub input: input::Input,
    pub engine: Engine,
    pub window: window::Window,
    pub graphics: graphics::Graphics,
    pub resources: resource::Resources,
}

impl Application {
    /// Creates empty `Application`.
    pub fn new() -> Result<Self> {
        let window = window::WindowBuilder::new().build()?;
        Ok(Application {
            input: input::Input::new(),
            engine: Engine::new(),
            graphics: graphics::Graphics::new(window.underlaying())?,
            window: window,
            resources: resource::Resources::new(),
        })
    }

    /// Setup application from configuration.
    pub fn setup<T>(path: T) -> Result<Self>
        where T: AsRef<Path>
    {
        let args = Arguments::new(path).chain_err(|| "failed to parse arguments.")?;

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
            let name = slice.load_as_str("Title").unwrap_or("Lemon3D - Application");
            let width = slice.load_as_i32("Width").unwrap_or(128) as u32;
            let height = slice.load_as_i32("Height").unwrap_or(128) as u32;
            wb.with_title(name.to_string()).with_dimensions(width, height);
        }

        let window = wb.build()?;
        Ok(Application {
            input: input::Input::new(),
            engine: engine,
            graphics: graphics::Graphics::new(window.underlaying())?,
            window: window,
            resources: resource::Resources::new(),
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
        'main: while exec {
            // Poll any possible events first.
            self.input.run_one_frame();
            for event in self.window.poll_events() {
                match event {
                    window::Event::Application(value) => {
                        match value {
                            window::ApplicationEvent::Closed => break 'main,
                            other => println!("Drop {:?}.", other),
                        };
                    }
                    window::Event::InputDevice(value) => self.input.process(value),
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