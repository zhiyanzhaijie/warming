use std::sync::Arc;

use domain::{
    ArrangementId, MusicPiece, MusicPieceCreated, MusicPieceId, PianoArrangement, PianoScore,
};

use crate::app_error::{AppError, AppResult};
use crate::music::{MusicEventHandler, MusicPieceRepositoryPort};

#[derive(Debug, Clone)]
pub struct CreateMusicPieceCommand {
    pub piece_id: MusicPieceId,
    pub title: String,
    pub creator: Option<String>,
    pub now: String,
}

#[derive(Debug, Clone)]
pub struct ImportPianoArrangementCommand {
    pub piece_id: MusicPieceId,
    pub arrangement_id: ArrangementId,
    pub title: String,
    pub score: PianoScore,
    pub now: String,
}

#[derive(Debug, Clone)]
pub struct CreateMusicPieceResult {
    pub piece: MusicPiece,
}

#[derive(Debug, Clone)]
pub struct ImportPianoArrangementResult {
    pub piece: MusicPiece,
    pub arrangement: PianoArrangement,
}

#[derive(Clone)]
pub struct MusicCommandHandler {
    pieces: Arc<dyn MusicPieceRepositoryPort>,
    event_handler: Option<MusicEventHandler>,
}

impl MusicCommandHandler {
    pub fn new(pieces: Arc<dyn MusicPieceRepositoryPort>) -> Self {
        Self {
            pieces,
            event_handler: None,
        }
    }

    pub fn with_event_handler(mut self, event_handler: MusicEventHandler) -> Self {
        self.event_handler = Some(event_handler);
        self
    }

    pub async fn create_piece(
        &self,
        input: CreateMusicPieceCommand,
    ) -> AppResult<CreateMusicPieceResult> {
        let title = input.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::validation("music piece title is empty"));
        }

        if self
            .pieces
            .find_piece(&input.piece_id)
            .await
            .map_err(AppError::upstream)?
            .is_some()
        {
            return Err(AppError::validation(format!(
                "music piece already exists: {}",
                input.piece_id.as_str()
            )));
        }

        let piece = MusicPiece {
            id: input.piece_id,
            title,
            creator: input.creator.and_then(|value| {
                let value = value.trim().to_string();
                (!value.is_empty()).then_some(value)
            }),
            arrangements: Vec::new(),
            created_at: input.now.clone(),
            updated_at: input.now,
        };

        self.pieces
            .save_piece(&piece)
            .await
            .map_err(AppError::upstream)?;

        if let Some(event_handler) = &self.event_handler {
            event_handler
                .handle_music_piece_created(&MusicPieceCreated {
                    piece_id: piece.id.clone(),
                })
                .await?;
        }

        Ok(CreateMusicPieceResult { piece })
    }

    pub async fn import_arrangement(
        &self,
        input: ImportPianoArrangementCommand,
    ) -> AppResult<ImportPianoArrangementResult> {
        let mut piece = self
            .pieces
            .find_piece(&input.piece_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("music piece".to_string()))?;

        let title = input.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::validation("arrangement title is empty"));
        }
        if score_note_count(&input.score) == 0 {
            return Err(AppError::validation("arrangement score contains no notes"));
        }

        let arrangement = PianoArrangement {
            id: input.arrangement_id,
            piece_id: input.piece_id,
            title,
            score: input.score,
        };

        piece.arrangements.retain(|item| item.id != arrangement.id);
        piece.arrangements.push(arrangement.clone());
        piece.updated_at = input.now;

        self.pieces
            .save_piece(&piece)
            .await
            .map_err(AppError::upstream)?;

        Ok(ImportPianoArrangementResult { piece, arrangement })
    }
}

fn score_note_count(score: &PianoScore) -> usize {
    score.parts.iter().map(|part| part.notes.len()).sum()
}
