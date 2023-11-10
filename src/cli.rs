use bevy::prelude::*;
use clap::Parser;

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CliArgs::parse());
    }
}

#[derive(Parser, Resource, Clone, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Load into game with the provided FEN
    pub fen: Option<String>,
}
