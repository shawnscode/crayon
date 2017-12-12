pub mod event;
pub mod input;

pub use self::event::{Event, ApplicationEvent, WindowEvent, InputDeviceEvent};
pub use self::event::{KeyboardButton, MouseButton};
pub use self::input::{InputSystem, InputSystemShared};