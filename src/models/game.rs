use serde::{Deserialize, Serialize};

use super::{Area, User, Wave};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub waves: Vec<Wave>,
    pub current_wave_index: usize,
    pub area: Area,
    pub users: Vec<User>,
}

impl Game {
    pub fn new(area: Area, users: Vec<User>) -> Self {
        Self {
            waves: Vec::new(),
            current_wave_index: 0,
            area,
            users,
        }
    }

    pub fn add_wave(&mut self) {
        self.waves.push(Wave::new());
        self.current_wave_index += 1;
    }

    pub fn get_wave(&self, index: usize) -> Option<&Wave> {
        self.waves.get(index)
    }

    #[allow(dead_code)]
    pub fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }

    #[allow(dead_code)]
    pub fn get_users(&self) -> &Vec<User> {
        &self.users
    }

    #[allow(dead_code)]
    pub fn get_users_mut(&mut self) -> &mut Vec<User> {
        &mut self.users
    }
}
