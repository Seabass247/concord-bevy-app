use bevy::{app::PluginGroupBuilder, prelude::*};

use concord_editor::ConcordEditorPlugin;

pub struct ConcordPlugins;

impl PluginGroup for ConcordPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(ConcordEditorPlugin::default());
    }
}
