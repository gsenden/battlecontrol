use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct ReferenceData {
    pub collision_cooldowns: CollisionCooldownScenario,
    pub collision_existing_cooldowns: CollisionCooldownScenario,
    pub energy: FrameScenario,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollisionCooldownScenario {
    pub turn_wait: i32,
    pub thrust_wait: i32,
}

#[derive(Deserialize)]
pub struct FrameScenario {
    pub frames: Vec<FrameData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameData {
    pub frame: i32,
    pub crew: i32,
    pub energy: i32,
    pub facing: i32,
    pub vx: i32,
    pub vy: i32,
    pub turn_wait: i32,
    pub thrust_wait: i32,
    pub weapon_counter: i32,
    pub special_counter: i32,
    pub energy_counter: i32,
}

pub fn load() -> ReferenceData {
    let json = fs::read_to_string("../testdata/reference.json")
        .expect("reference.json not found — run from game-logic/");
    serde_json::from_str(&json).expect("failed to parse reference.json")
}
