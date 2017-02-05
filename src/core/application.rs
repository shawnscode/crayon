use std::path::Path;
use super::engine::Engine;
use super::arguments::Arguments;
use gl;
use glutin;

#[derive(Debug)]
pub enum Error {
    ArgumentsBreak(::std::io::Error),
    WindowContextBreak(glutin::CreationError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// User-friendly facade for building applications.
pub struct Application {
    engine: Engine,
    window: glutin::Window,
}

impl Application {
    /// Creates empty `Application`.
    pub fn new() -> Result<Self> {
        Ok(Application {
            engine: Engine::new(),
            window: glutin::Window::new().map_err(|e| Error::WindowContextBreak(e))?,
        })
    }

    /// Setup application from configuration.
    pub fn setup<T>(path: T) -> Result<Self>
        where T: AsRef<Path>
    {
        let args = Arguments::new(path).map_err(|e| Error::ArgumentsBreak(e))?;

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

        let window = if let Some(slice) = args.load_as_slice("Window") {
                let name = slice.load_as_str("Title").unwrap_or("Lemon3D - Application");
                let width = slice.load_as_i32("Width").unwrap_or(128) as u32;
                let height = slice.load_as_i32("Height").unwrap_or(128) as u32;

                glutin::WindowBuilder::new()
                    .with_title(name)
                    .with_dimensions(width, height)
                    .build()
            } else {
                glutin::Window::new()
            }.map_err(|e| Error::WindowContextBreak(e))?;

        Ok(Application {
            engine: engine,
            window: window,
        })
    }

    /// Perform custom logics after engine initialization.
    pub fn perform<F>(mut self, closure: F) -> Self
        where F: FnOnce(&mut Engine)
    {
        closure(&mut self.engine);
        self
    }

    /// Run the main loop of `Application`, this will block the working
    /// thread until we finished.
    pub fn run<F>(mut self, closure: F) -> Self
        where F: Fn(&mut Engine) -> bool
    {
        println!("Launch Lemon3D.");
        println!("PWD: {:?}", ::std::env::current_dir());

        unsafe {
            self.window.make_current().unwrap();
            gl::load_with(|symbol| self.window.get_proc_address(symbol) as *const _);
        }

        'main: while closure(&mut self.engine) {
            for event in self.window.wait_events() {
                match event {
                    glutin::Event::Closed => break 'main,
                    _ => (),
                }
            }

            self.engine.run_one_frame();
            self.window.swap_buffers().unwrap();
        }
        self
    }
}