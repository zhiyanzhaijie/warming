use std::sync::Arc;

use domain::{ArrangementId, MusicPiece, MusicPieceId, PianoArrangement};

use crate::app_error::{AppError, AppResult};
use crate::music::MusicPieceRepositoryPort;

#[derive(Clone)]
pub struct MusicQueryHandler {
    pieces: Arc<dyn MusicPieceRepositoryPort>,
}

impl MusicQueryHandler {
    pub fn new(pieces: Arc<dyn MusicPieceRepositoryPort>) -> Self {
        Self { pieces }
    }

    pub async fn get_piece(&self, id: &MusicPieceId) -> AppResult<Option<MusicPiece>> {
        self.pieces.find_piece(id).await.map_err(AppError::upstream)
    }

    pub async fn list_pieces(&self) -> AppResult<Vec<MusicPiece>> {
        self.pieces.list_pieces().await.map_err(AppError::upstream)
    }

    pub async fn get_arrangement(
        &self,
        piece_id: &MusicPieceId,
        arrangement_id: &ArrangementId,
    ) -> AppResult<Option<PianoArrangement>> {
        let piece = self
            .pieces
            .find_piece(piece_id)
            .await
            .map_err(AppError::upstream)?;
        Ok(piece.and_then(|piece| {
            piece
                .arrangements
                .into_iter()
                .find(|item| item.id == *arrangement_id)
        }))
    }
}
