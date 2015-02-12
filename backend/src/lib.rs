/*!
Window-wrangling, polygon-pushing and input-grabbing.

*/
#![crate_name="calx_backend"]
#![feature(collections, io)]
#![feature(plugin)]
#![plugin(glium_macros)]

extern crate glutin;
extern crate glium_macros;
extern crate glium;
extern crate "calx_util" as util;
extern crate time;
extern crate image;

pub use canvas::{CanvasBuilder, Canvas};
pub use canvas::{Image};
pub use canvas_util::{CanvasUtil};
pub use key::Key;
pub use fonter::{Fonter, CanvasWriter};
pub use event::{Event, MouseButton};

mod canvas;
mod canvas_util;
mod event;
mod fonter;
mod key;
mod renderer;

#[cfg(target_os = "macos")]
mod scancode_macos;
#[cfg(target_os = "linux")]
mod scancode_linux;
#[cfg(target_os = "windows")]
mod scancode_windows;

mod scancode {
#[cfg(target_os = "macos")]
    pub use scancode_macos::MAP;
#[cfg(target_os = "linux")]
    pub use scancode_linux::MAP;
#[cfg(target_os = "windows")]
    pub use scancode_windows::MAP;
}
