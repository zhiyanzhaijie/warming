use async_trait::async_trait;
use domain::{MusicPiece, MusicPieceId};

#[async_trait]
pub trait MusicPieceRepositoryPort: Send + Sync {
    async fn save_piece(&self, piece: &MusicPiece) -> Result<(), String>;

    async fn find_piece(&self, id: &MusicPieceId) -> Result<Option<MusicPiece>, String>;

    async fn list_pieces(&self) -> Result<Vec<MusicPiece>, String>;
}
