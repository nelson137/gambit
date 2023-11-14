#[cfg(feature = "debug-inspector")]
#[allow(clippy::module_inception)]
mod debug_inspector;

#[cfg(not(feature = "debug-inspector"))]
mod debug_inspector {
    impl bevy::app::Plugin for super::DebugInspectorPlugin {
        #[inline(always)]
        fn build(&self, _app: &mut bevy::app::App) {}
    }
}

pub struct DebugInspectorPlugin;
