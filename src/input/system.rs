use std::sync::{Arc, RwLock};

use crate::application::prelude::{LifecycleListener, LifecycleListenerHandle};
use crate::window::prelude::{Event, EventListener, EventListenerHandle};

use super::events::InputEvent;
use super::keyboard::{Key, Keyboard};
use super::mouse::{Mouse, MouseButton};
use super::touchpad::{GesturePan, GestureTap, TouchPad, TouchState};
use super::InputParams;

use crate::math::prelude::Vector2;

/// The `InputSystem` struct are used to manage all the events and corresponding
/// internal states.
pub struct InputSystem {
    events: EventListenerHandle,
    lifecycle: LifecycleListenerHandle,
    state: Arc<InputState>,
}

struct InputState {
    touch_emulation: bool,
    touch_emulation_button: RwLock<Option<MouseButton>>,
    mouse: RwLock<Mouse>,
    keyboard: RwLock<Keyboard>,
    touchpad: RwLock<TouchPad>,
}

impl EventListener for Arc<InputState> {
    fn on(&mut self, v: &Event) -> Result<(), failure::Error> {
        if let Event::InputDevice(v) = *v {
            match v {
                InputEvent::MouseMoved { position } => {
                    if self.touch_emulation_button.read().unwrap().is_some() {
                        self.touchpad.write().unwrap().on_touch(
                            255,
                            TouchState::Move,
                            self.mouse.read().unwrap().position(),
                        );
                    }

                    self.mouse.write().unwrap().on_move(position)
                }

                InputEvent::MousePressed { button } => {
                    if self.touch_emulation {
                        *self.touch_emulation_button.write().unwrap() = Some(button);
                        self.touchpad.write().unwrap().on_touch(
                            255,
                            TouchState::Start,
                            self.mouse.read().unwrap().position(),
                        );
                    }

                    self.mouse.write().unwrap().on_button_pressed(button)
                }

                InputEvent::MouseReleased { button } => {
                    if *self.touch_emulation_button.read().unwrap() == Some(button) {
                        *self.touch_emulation_button.write().unwrap() = None;

                        self.touchpad.write().unwrap().on_touch(
                            255,
                            TouchState::End,
                            self.mouse.read().unwrap().position(),
                        );
                    }

                    self.mouse.write().unwrap().on_button_released(button)
                }

                InputEvent::MouseWheel { delta } => {
                    self.mouse.write().unwrap().on_wheel_scroll(delta)
                }

                InputEvent::KeyboardPressed { key } => {
                    self.keyboard.write().unwrap().on_key_pressed(key)
                }

                InputEvent::KeyboardReleased { key } => {
                    self.keyboard.write().unwrap().on_key_released(key)
                }

                InputEvent::ReceivedCharacter { character } => {
                    self.keyboard.write().unwrap().on_char(character)
                }

                InputEvent::Touch {
                    id,
                    state,
                    position,
                } => {
                    self.touchpad.write().unwrap().on_touch(id, state, position);
                }
            }
        }

        Ok(())
    }
}

impl LifecycleListener for Arc<InputState> {
    fn on_post_update(&mut self) -> Result<(), failure::Error> {
        self.mouse.write().unwrap().advance();
        self.keyboard.write().unwrap().advance();
        self.touchpad.write().unwrap().advance();
        Ok(())
    }
}

impl Drop for InputSystem {
    fn drop(&mut self) {
        crate::application::detach(self.lifecycle);
        crate::window::detach(self.events);
    }
}

impl InputSystem {
    pub fn new(setup: InputParams) -> Self {
        debug_assert!(crate::application::valid(), "");

        let state = Arc::new(InputState {
            touch_emulation: setup.touch_emulation,
            touch_emulation_button: RwLock::new(None),
            mouse: RwLock::new(Mouse::new(setup.mouse)),
            keyboard: RwLock::new(Keyboard::new(setup.keyboard)),
            touchpad: RwLock::new(TouchPad::new(setup.touchpad)),
        });

        InputSystem {
            state: state.clone(),
            lifecycle: crate::application::attach(state.clone()),
            events: crate::window::attach(state),
        }
    }

    /// Reset input to initial states.
    pub fn reset(&self) {
        self.state.mouse.write().unwrap().reset();
        self.state.keyboard.write().unwrap().reset();
        self.state.touchpad.write().unwrap().reset();

        *self.state.touch_emulation_button.write().unwrap() = None;
    }

    /// Returns true if a keyboard is attached
    #[inline]
    pub fn has_keyboard_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if a key is currently held down.
    #[inline]
    pub fn is_key_down(&self, key: Key) -> bool {
        self.state.keyboard.read().unwrap().is_key_down(key)
    }

    /// Checks if a key has been pressed down during the last frame.
    #[inline]
    pub fn is_key_press(&self, key: Key) -> bool {
        self.state.keyboard.read().unwrap().is_key_press(key)
    }

    /// Checks if a key has been released during the last frame.
    #[inline]
    pub fn is_key_release(&self, key: Key) -> bool {
        self.state.keyboard.read().unwrap().is_key_release(key)
    }

    /// Checks if a key has been repeated during the last frame.
    #[inline]
    pub fn is_key_repeat(&self, key: Key) -> bool {
        self.state.keyboard.read().unwrap().is_key_repeat(key)
    }

    /// Gets captured text during the last frame.
    #[inline]
    pub fn text(&self) -> String {
        use std::iter::FromIterator;
        String::from_iter(self.state.keyboard.read().unwrap().captured_chars())
    }

    /// Returns true if a mouse is attached
    #[inline]
    pub fn has_mouse_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if a mouse buttoAn is held down.
    #[inline]
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.state.mouse.read().unwrap().is_button_down(button)
    }

    /// Checks if a mouse button has been pressed during last frame.
    #[inline]
    pub fn is_mouse_press(&self, button: MouseButton) -> bool {
        self.state.mouse.read().unwrap().is_button_press(button)
    }

    /// Checks if a mouse button has been released during last frame.
    #[inline]
    pub fn is_mouse_release(&self, button: MouseButton) -> bool {
        self.state.mouse.read().unwrap().is_button_release(button)
    }

    /// Checks if a mouse button has been clicked during last frame.
    #[inline]
    pub fn is_mouse_click(&self, button: MouseButton) -> bool {
        self.state.mouse.read().unwrap().is_button_click(button)
    }

    /// Checks if a mouse button has been double clicked during last frame.
    #[inline]
    pub fn is_mouse_double_click(&self, button: MouseButton) -> bool {
        self.state
            .mouse
            .read()
            .unwrap()
            .is_button_double_click(button)
    }

    /// Gets the mouse position relative to the lower-left hand corner of the window.
    #[inline]
    pub fn mouse_position(&self) -> Vector2<f32> {
        self.state.mouse.read().unwrap().position()
    }

    /// Gets mouse movement since last frame.
    #[inline]
    pub fn mouse_movement(&self) -> Vector2<f32> {
        self.state.mouse.read().unwrap().movement()
    }

    /// Gets the scroll movement of mouse, usually provided by mouse wheel.
    #[inline]
    pub fn mouse_scroll(&self) -> Vector2<f32> {
        self.state.mouse.read().unwrap().scroll()
    }

    /// Returns true if a touchpad is attached
    #[inline]
    pub fn has_touchpad_attached(&self) -> bool {
        // TODO
        true
    }

    /// Checks if the `n`th finger is touched during last frame.
    #[inline]
    pub fn is_finger_touched(&self, n: usize) -> bool {
        self.state.touchpad.read().unwrap().is_touched(n)
    }

    /// Gets the position of the `n`th touched finger.
    #[inline]
    pub fn finger_position(&self, n: usize) -> Option<Vector2<f32>> {
        self.state.touchpad.read().unwrap().position(n)
    }

    /// Gets the tap gesture.
    #[inline]
    pub fn finger_tap(&self) -> GestureTap {
        self.state.touchpad.read().unwrap().tap()
    }

    /// Gets the double tap gesture.
    #[inline]
    pub fn finger_double_tap(&self) -> GestureTap {
        self.state.touchpad.read().unwrap().double_tap()
    }

    /// Gets the panning gesture.
    #[inline]
    pub fn finger_pan(&self) -> GesturePan {
        self.state.touchpad.read().unwrap().pan()
    }
}
