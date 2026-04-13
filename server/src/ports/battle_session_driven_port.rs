use common::dto::GameDto;

use crate::adapters::{BattleClientMessage, BattleSnapshotDto};

pub trait BattleSessionDrivenPort {
    fn start_battle(&self, game: &GameDto) -> Result<(), String>;
    fn remove_battle(&self, game_id: &str);
    fn has_battle(&self, game_id: &str) -> bool;
    fn snapshot_for(&self, game_id: &str, user_name: &str) -> Option<BattleSnapshotDto>;
    fn apply_message(
        &self,
        game_id: &str,
        user_name: &str,
        message: BattleClientMessage,
    ) -> Result<(), String>;
}
