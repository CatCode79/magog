/*!
Window-wrangling, polygon-pushing and input-grabbing
*/

#[macro_use] extern crate glium;
extern crate image;
extern crate calx_window;
extern crate calx_color;
extern crate calx_layout;

//pub use canvas::{CanvasBuilder, Canvas};
//pub use canvas::{Image};
//pub use canvas_util::{CanvasUtil};
//pub use fonter::{Fonter, Align};

//mod canvas;
//mod canvas_util;
//mod fonter;
pub mod mesh;

/// Drawable images stored in the atlas texture of a Mesh.
///
/// By convention, Default Image is assumed to contain a solid-color texel for
/// drawing solid polygons.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Image(usize);

pub trait RenderTarget {
    fn add_mesh(&mut self, vertices: Vec<mesh::Vertex>, faces: Vec<[u16; 3]>);
}

/// UI Widget static identifier, unique for a specific site in source code.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidgetId {
    filename: &'static str,
    line: u32,
    column: u32,
}

impl WidgetId {
    pub fn new(filename: &'static str, line: u32, column: u32) -> WidgetId {
        WidgetId {
            filename: filename,
            line: line,
            column: column,
        }
    }

    pub fn dummy() -> WidgetId {
        WidgetId {
            filename: "n/a",
            line: 666666,
            column: 666666,
        }
    }
}

#[macro_export]
/// Generate a static identifier for the current source code position. Used
/// with imgui API.
macro_rules! widget_id {
    () => {
        ::calx::backend::WidgetId::new(concat!(module_path!(), "/", file!()), line!(), column!())
    }
}
