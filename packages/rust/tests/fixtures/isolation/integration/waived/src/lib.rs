//! The library under test.

#[derive(Default)]
pub struct Renderer;

impl Renderer {
    pub fn render(&self) -> String {
        "widget".to_string()
    }
}
