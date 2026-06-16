//! Red: doubles the crate under test (`widget`). An integration test runs
//! first-party code for real; mock external deps, never the library itself.

use mockall_double::double;

// VIOLATION (`no-first-party-double`): `#[double]` swaps `widget::Renderer` (the
// crate under test) for its mock.
#[double]
use widget::Renderer;

#[test]
fn renders() {
    let renderer = Renderer::default();
    let _ = renderer.render();
}
