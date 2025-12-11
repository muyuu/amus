use serde::{Deserialize, Serialize};

use super::{Area, Player, Wave};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub waves: Vec<Wave>,
    pub current_wave_index: usize,
    pub area: Area,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new(area: Area, players: Vec<Player>) -> Self {
        Self {
            waves: Vec::new(),
            current_wave_index: 0,
            area,
            players,
        }
    }

    pub fn add_wave(&mut self) {
        self.waves.push(Wave::default());
        self.current_wave_index += 1;
    }

    pub fn get_wave(&self, index: usize) -> Option<&Wave> {
        self.waves.get(index)
    }

    pub fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }

    #[allow(dead_code)]
    pub fn get_players(&self) -> &Vec<Player> {
        &self.players
    }

    #[allow(dead_code)]
    pub fn get_playrs_mut(&mut self) -> &mut Vec<Player> {
        &mut self.players
    }
}
