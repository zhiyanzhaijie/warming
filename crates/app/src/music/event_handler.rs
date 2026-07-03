use domain::music::event::MusicPieceCreated;

use crate::app_error::AppResult;

#[derive(Clone, Default)]
pub struct MusicEventHandler;

impl MusicEventHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_music_piece_created(&self, _event: &MusicPieceCreated) -> AppResult<()> {
        Ok(())
    }
}
