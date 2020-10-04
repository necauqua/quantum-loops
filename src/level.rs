use nalgebra::Point2;
use serde::{Deserialize, Serialize};

use crate::states::main_game::Disruption;

fn default_offset() -> Point2<f32> {
    [0.0, 0.0].into()
}

fn default_width() -> f32 {
    1.0
}

fn default_restore_time() -> f32 {
    3.0
}

fn default_color() -> String {
    "black".into()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyRing {
    pub radius: f32,

    #[serde(default = "default_offset")]
    pub offset: Point2<f32>,
    #[serde(default = "default_width")]
    pub width: f32,
    #[serde(default = "default_color")]
    pub color: String,

    pub base_energy: f32,

    #[serde(default = "default_restore_time")]
    pub restore_time: f32,

    #[serde(skip)]
    pub disrupted_time: f32,
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
    pub energy: f32,
    pub rings: Vec<EnergyRing>,
}

impl EnergyRing {
    pub fn intersects(&self, center: Point2<f32>, disruption: &Disruption) -> bool {
        let d1 = disruption.start.coords.metric_distance(&center.coords);
        let d2 = disruption.end.coords.metric_distance(&center.coords);
        let r = center.coords.min() * 2.0 * self.radius;
        d1.min(d2) <= r && d1.max(d2) >= r
    }
}

