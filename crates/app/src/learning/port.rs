use async_trait::async_trait;
use domain::{MusicPieceId, PracticeSession, PracticeSessionId};

#[async_trait]
pub trait PracticeSessionRepositoryPort: Send + Sync {
    async fn save_session(&self, session: &PracticeSession) -> Result<(), String>;

    async fn find_session(&self, id: &PracticeSessionId)
    -> Result<Option<PracticeSession>, String>;

    async fn list_sessions_by_piece(
        &self,
        piece_id: &MusicPieceId,
    ) -> Result<Vec<PracticeSession>, String>;
}
