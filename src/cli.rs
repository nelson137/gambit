use bevy::prelude::Resource;
use clap::Parser;

#[derive(Parser, Resource, Clone, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Load into game with the provided FEN
    pub fen: Option<String>,
}
