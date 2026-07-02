use super::PracticeSessionId;
use crate::MusicPieceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Practiced {
    pub session_id: PracticeSessionId,
    pub piece_id: MusicPieceId,
}
