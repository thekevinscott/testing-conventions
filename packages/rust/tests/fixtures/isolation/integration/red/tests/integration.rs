//! Red: an integration test that doubles the crate under test (`widget`).
//! Integration tests run first-party code for real; mock external deps, never the
//! library itself.

use mockall_double::double;

// VIOLATION: `#[double]` swaps `widget::Renderer` (the crate under test) for its
// mock — first-party code an integration test must exercise for real.
#[double]
use widget::Renderer;

#[test]
fn renders() {
    let renderer = Renderer::default();
    let _ = renderer.render();
}
