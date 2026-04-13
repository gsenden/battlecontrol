#![allow(dead_code)]

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct ReferenceData {
    pub collision_cooldowns: CollisionCooldownScenario,
    pub collision_existing_cooldowns: CollisionCooldownScenario,
    pub energy: FrameScenario,
    pub human_nuke_straight: HumanNukeScenario,
    pub human_nuke_homing: HumanNukeHomingScenario,
    pub androsynth_bubble_targeted: AndrosynthBubbleScenario,
    pub androsynth_bubble_two_shots: AndrosynthBubbleTwoShotsScenario,
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
pub struct HumanNukeScenario {
    pub frames: Vec<HumanNukeFrameData>,
}

#[derive(Deserialize)]
pub struct HumanNukeHomingScenario {
    pub frames: Vec<HumanNukeHomingFrameData>,
}

#[derive(Deserialize)]
pub struct AndrosynthBubbleScenario {
    pub frames: Vec<AndrosynthBubbleFrameData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndrosynthBubbleTwoShotsScenario {
    pub first: SimpleProjectileFrameData,
    pub second_frames: Vec<SimpleProjectileFrameData>,
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

#[derive(Deserialize)]
pub struct HumanNukeFrameData {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanNukeHomingFrameData {
    pub target_x: i32,
    pub target_y: i32,
    pub facing: i32,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndrosynthBubbleFrameData {
    pub target_x: i32,
    pub target_y: i32,
    pub facing: i32,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: i32,
}

#[derive(Deserialize)]
pub struct SimpleProjectileFrameData {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
}

pub fn load() -> ReferenceData {
    let reference_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../frontend/testdata/reference.json");
    let json = fs::read_to_string(reference_path)
        .expect("reference.json not found");
    serde_json::from_str(&json).expect("failed to parse reference.json")
}
