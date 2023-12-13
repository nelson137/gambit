pub struct DebugInspectorPlugin;

#[cfg(feature = "debug-inspector")]
mod debug_inspector_impl;

#[cfg(feature = "debug-inspector")]
pub use debug_inspector_impl::*;

#[cfg(not(feature = "debug-inspector"))]
mod debug_inspector_impl {
    impl bevy::app::Plugin for super::DebugInspectorPlugin {
        #[inline(always)]
        fn build(&self, _app: &mut bevy::app::App) {}
    }
}
