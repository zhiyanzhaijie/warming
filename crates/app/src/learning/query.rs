use std::sync::Arc;

use domain::{MusicPieceId, PracticeSession, PracticeSessionId};

use crate::app_error::{AppError, AppResult};
use crate::learning::PracticeSessionRepositoryPort;

#[derive(Clone)]
pub struct LearningQueryHandler {
    sessions: Arc<dyn PracticeSessionRepositoryPort>,
}

impl LearningQueryHandler {
    pub fn new(sessions: Arc<dyn PracticeSessionRepositoryPort>) -> Self {
        Self { sessions }
    }

    pub async fn get_session(
        &self,
        session_id: &PracticeSessionId,
    ) -> AppResult<Option<PracticeSession>> {
        self.sessions
            .find_session(session_id)
            .await
            .map_err(AppError::upstream)
    }

    pub async fn list_sessions_by_piece(
        &self,
        piece_id: &MusicPieceId,
    ) -> AppResult<Vec<PracticeSession>> {
        self.sessions
            .list_sessions_by_piece(piece_id)
            .await
            .map_err(AppError::upstream)
    }
}
