mod split_panel;

pub struct DebugInspectorPlugin;

#[cfg(feature = "debug-inspector")]
#[allow(clippy::module_inception)]
mod debug_inspector;

#[cfg(not(feature = "debug-inspector"))]
#[allow(clippy::module_inception)]
mod debug_inspector {
    impl bevy::app::Plugin for super::DebugInspectorPlugin {
        #[inline(always)]
        fn build(&self, _app: &mut bevy::app::App) {}
    }
}
