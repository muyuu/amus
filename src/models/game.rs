use serde::{Deserialize, Serialize};

use super::{Area, User, Wave};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub waves: Vec<Wave>,
    pub area: Area,
    pub users: Vec<User>,
}

impl Game {
    pub fn new(area: Area, users: Vec<User>) -> Self {
        Self {
            waves: Vec::new(),
            area,
            users,
        }
    }

    pub fn add_wave(&mut self) {
        self.waves.push(Wave::new());
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

    pub fn get_users_and_wave_mut(
        &mut self,
        wave_index: usize,
    ) -> (&mut Vec<User>, Option<&mut Wave>) {
        (&mut self.users, self.waves.get_mut(wave_index))
    }
}
