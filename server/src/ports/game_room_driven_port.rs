use common::dto::GameDto;

pub trait GameRoomDrivenPort: Clone + Send + Sync + 'static {
    fn create_room(&self, game: &GameDto);
    fn update_room(&self, game: &GameDto);
    fn start_room(&self, game: &GameDto);
    fn cancel_room(&self, game_id: &str);
    fn remove_rooms(&self, game_ids: &[String]);
}
