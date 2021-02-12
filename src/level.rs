use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::states::main_game::Disruption;
use crate::engine::util::Bitmap;

fn default_offset() -> Vector2<f64> {
    [0.0, 0.0].into()
}

fn default_width() -> f64 {
    1.0
}

fn default_restore_time() -> f64 {
    3.0
}

fn default_color() -> String {
    "black".into()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyRing {
    pub radius: f64,

    #[serde(default = "default_offset")]
    pub offset: Vector2<f64>,
    #[serde(default = "default_width")]
    pub width: f64,
    #[serde(default = "default_color")]
    pub color: String,

    pub base_energy: f64,

    #[serde(default = "default_restore_time")]
    pub restore_time: f64,

    #[serde(skip)]
    pub disrupted_time: f64,
}

impl Clone for EnergyRing {
    fn clone(&self) -> Self {
        Self {
            disrupted_time: 0.0,
            color: self.color.clone(),
            ..*self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLevel {
    pub name: String,
    pub energy: f64,
    pub rings: Vec<EnergyRing>,
}

impl EnergyRing {
    pub fn intersects(&self, center: Vector2<f64>, disruption: &Disruption) -> bool {
        let min_dim = center.min() * 2.0;
        let center = center + self.offset * min_dim;
        let d1 = disruption.start.metric_distance(&center);
        let d2 = disruption.end.metric_distance(&center);
        let r = min_dim * self.radius;
        d1.min(d2) <= r && d1.max(d2) >= r
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredData {
    pub passed_tutorial: bool,
    pub unlocked_level: usize,
    pub best_scores: Vec<f64>,
    pub sounds_enabled: bool,
    pub music_enabled: bool,
}

impl Default for StoredData {
    fn default() -> Self {
        Self {
            passed_tutorial: false,
            unlocked_level: 0,
            best_scores: Vec::new(),
            sounds_enabled: true,
            music_enabled: true,
        }
    }
}

impl StoredData {
    pub fn get_enabled_sounds(&self) -> Bitmap {
        Bitmap::empty()
            .with_set(0, self.sounds_enabled)
            .with_set(1, self.music_enabled)
    }
}
