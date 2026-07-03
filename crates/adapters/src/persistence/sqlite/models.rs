#[derive(Debug, Clone, toasty::Model)]
pub struct MusicPieceRow {
    #[key]
    pub id: String,
    pub title: String,
    pub creator: Option<String>,
    pub piece: toasty::Json<domain::MusicPiece>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct PracticeSessionRow {
    #[key]
    pub id: String,
    #[index]
    pub piece_id: String,
    pub session: toasty::Json<domain::PracticeSession>,
    pub started_at: String,
    pub ended_at: Option<String>,
}
