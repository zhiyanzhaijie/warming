use super::MusicPieceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicPieceCreated {
    pub piece_id: MusicPieceId,
}
