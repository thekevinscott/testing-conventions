//! An integration test that doubles the crate under test (`widget`) — a
//! violation the lint must find from the derived crate root, even when the
//! scanned `path` is the sibling `src/` directory.

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
