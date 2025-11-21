use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum Area {
    Skeld = 0,
    Mira = 1,
    Polus = 2,
    #[default]
    AirShip = 3,
}

impl Area {
    pub fn id(&self) -> String {
        match self {
            Area::Skeld => "skeld".to_string(),
            Area::Mira => "mira".to_string(),
            Area::Polus => "polus".to_string(),
            Area::AirShip => "airship".to_string(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Area::Skeld => "The Skeld".to_string(),
            Area::Mira => "Mira HQ".to_string(),
            Area::Polus => "Polus".to_string(),
            Area::AirShip => "AirShip".to_string(),
        }
    }

    pub fn all() -> Vec<Area> {
        vec![Area::Skeld, Area::Mira, Area::Polus, Area::AirShip]
    }
}
