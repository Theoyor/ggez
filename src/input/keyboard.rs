//! Keyboard utility functions; allow querying state of keyboard keys and modifiers.
//!
//! Example:
//!
//! ```rust, compile
//! use ggez::event::{self, EventHandler, KeyCode, KeyMods};
//! use ggez::{graphics, nalgebra as na, timer};
//! use ggez::input::keyboard;
//! use ggez::{Context, GameResult};
//!
//! struct MainState {
//!     position_x: f32,
//! }
//!
//! impl EventHandler for MainState {
//!     fn update(&mut self, ctx: &mut Context) -> GameResult {
//!         // Increase or decrease `position_x` by 0.5, or by 5.0 if Shift is held.
//!         if keyboard::is_key_pressed(ctx, KeyCode::Right) {
//!             if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
//!                 self.position_x += 4.5;
//!             }
//!             self.position_x += 0.5;
//!         } else if keyboard::is_key_pressed(ctx, KeyCode::Left) {
//!             if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
//!                 self.position_x -= 4.5;
//!             }
//!             self.position_x -= 0.5;
//!         }
//!         Ok(())
//!     }
//!
//!     fn draw(&mut self, ctx: &mut Context) -> GameResult {
//!         graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
//!         // Create a circle at `position_x` and draw
//!         let circle = graphics::Mesh::new_circle(
//!             ctx,
//!             graphics::DrawMode::fill(),
//!             na::Point2::new(self.position_x, 380.0),
//!             100.0,
//!             2.0,
//!             graphics::WHITE,
//!         )?;
//!         graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
//!         graphics::present(ctx)?;
//!         timer::yield_now();
//!         Ok(())
//!     }
//!
//!     fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
//!         match key {
//!             // Quit if Shift+Ctrl+Q is pressed.
//!             KeyCode::Q => {
//!                 if mods.contains(KeyMods::SHIFT & KeyMods::CTRL) {
//!                     println!("Terminating!");
//!                     event::quit(ctx);
//!                 } else if mods.contains(KeyMods::SHIFT) || mods.contains(KeyMods::CTRL) {
//!                     println!("You need to hold both Shift and Control to quit.");
//!                 } else {
//!                     println!("Now you're not even trying!");
//!                 }
//!             }
//!             _ => (),
//!         }
//!     }
//! }
//! ```

use crate::context::Context;

use glutin::event::ModifiersState;
/// A key code.
pub use glutin::event::VirtualKeyCode as KeyCode;
use std::collections::HashSet;

/// Tracks held down keyboard keys, active keyboard modifiers,
/// and figures out if the system is sending repeat keystrokes.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    active_modifiers: ModifiersState,
    /// A simple mapping of which key code has been pressed.
    /// We COULD use a `Vec<bool>` but turning Rust enums to and from
    /// integers is unsafe and a set really is what we want anyway.
    pressed_keys_set: HashSet<KeyCode>,

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<KeyCode>,
    current_pressed: Option<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            active_modifiers: ModifiersState::empty(),
            // We just use 256 as a number Big Enough For Keyboard Keys to try to avoid resizing.
            pressed_keys_set: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
        }
    }
    pub(crate) fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_keys_set.insert(key);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            let _ = self.pressed_keys_set.remove(&key);
            self.current_pressed = None;
        }
    }

    pub(crate) fn set_modifiers(&mut self, keymods: ModifiersState) {
        self.active_modifiers = keymods;
    }

    pub(crate) fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys_set.contains(&key)
    }

    pub(crate) fn is_key_repeated(&self) -> bool {
        if self.last_pressed.is_some() {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }

    pub(crate) fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys_set
    }

    pub(crate) fn active_mods(&self) -> ModifiersState {
        self.active_modifiers
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a key is currently pressed down.
pub fn is_key_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard_context.is_key_pressed(key)
}

/// Checks if the last keystroke sent by the system is repeated,
/// like when a key is held down for a period of time.
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard_context.is_key_repeated()
}

/// Returns a reference to the set of currently pressed keys.
pub fn pressed_keys(ctx: &Context) -> &HashSet<KeyCode> {
    ctx.keyboard_context.pressed_keys()
}

/// Checks if keyboard modifier (or several) is active.
pub fn is_mod_active(ctx: &Context, keymods: ModifiersState) -> bool {
    ctx.keyboard_context.active_mods().contains(keymods)
}

/// Returns currently active keyboard modifiers.
pub fn active_mods(ctx: &Context) -> ModifiersState {
    ctx.keyboard_context.active_mods()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.pressed_keys(), &[].iter().cloned().collect());
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().cloned().collect()
        );
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.pressed_keys(), &[].iter().cloned().collect());
        assert!(!keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().cloned().collect()
        );
        assert!(keyboard.is_key_pressed(KeyCode::A));
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A, KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::A, KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(
            keyboard.pressed_keys(),
            &[KeyCode::B].iter().cloned().collect()
        );
        keyboard.set_key(KeyCode::B, false);
        assert_eq!(keyboard.pressed_keys(), &[].iter().cloned().collect());
    }

    /*
     * TODO: This should probably be fixed
    #[test]
    fn keyboard_modifiers() {
        let mut keyboard = KeyboardContext::new();

        // this test is mostly useless and is primarily for code coverage
        assert_eq!(keyboard.active_mods(), KeyMods::default());
        keyboard.set_modifiers(KeyMods::from(ModifiersState {
            shift: true,
            ctrl: true,
            alt: true,
            logo: true,
        }));

        // these test the workaround for https://github.com/tomaka/winit/issues/600
        assert_eq!(
            keyboard.active_mods(),
            KeyMods::SHIFT | KeyMods::CTRL | KeyMods::ALT | KeyMods::LOGO
        );
        keyboard.set_key(KeyCode::LControl, false);
        assert_eq!(
            keyboard.active_mods(),
            KeyMods::SHIFT | KeyMods::ALT | KeyMods::LOGO
        );
        keyboard.set_key(KeyCode::RAlt, false);
        assert_eq!(keyboard.active_mods(), KeyMods::SHIFT | KeyMods::LOGO);
        keyboard.set_key(KeyCode::LWin, false);
        assert_eq!(keyboard.active_mods(), KeyMods::SHIFT);
    }
    */

    #[test]
    fn repeated_keys_tracking() {
        let mut keyboard = KeyboardContext::new();
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), true);
        keyboard.set_key(KeyCode::A, false);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::A, true);
        assert_eq!(keyboard.is_key_repeated(), true);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.is_key_repeated(), false);
        keyboard.set_key(KeyCode::B, true);
        assert_eq!(keyboard.is_key_repeated(), true);
    }
}
