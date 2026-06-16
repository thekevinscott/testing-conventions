//! Waived: doubles the crate under test (`no-first-party-double`), but the file is
//! lifted by a `[[rust.exempt]]` entry in testing-conventions.toml.

use mockall_double::double;

#[double]
use widget::Renderer;

#[test]
fn renders() {
    let renderer = Renderer::default();
    let _ = renderer.render();
}
