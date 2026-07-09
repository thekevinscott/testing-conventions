//! The library under test (first-party code an integration test must run for real).

#[derive(Default)]
pub struct Renderer;

impl Renderer {
    pub fn render(&self) -> String {
        "widget".to_string()
    }
}
