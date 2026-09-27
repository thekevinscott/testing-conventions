//! The same first-party `#[double]` as the `red` fixture, written inside a macro's
//! token body. The tokens are written here and land here, so the rule must see it.

use mockall_double::double;

with_setup! {
    #[double]
    use widget::Renderer;

    #[test]
    fn renders() {
        let renderer = Renderer::default();
        let _ = renderer.render();
    }
}
