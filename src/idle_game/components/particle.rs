use bevy::prelude::*;
use crate::idle_game::constants::TileType;

#[derive(Component, Debug)]
pub struct Particle {
    pub tile_type: TileType
}
