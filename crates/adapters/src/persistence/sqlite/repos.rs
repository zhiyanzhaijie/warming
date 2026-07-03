use std::sync::Arc;

use app::learning::PracticeSessionRepositoryPort;
use app::music::MusicPieceRepositoryPort;
use async_trait::async_trait;
use domain::{MusicPiece, MusicPieceId, PracticeSession, PracticeSessionId};
use tokio::sync::Mutex;

use super::models::{MusicPieceRow, PracticeSessionRow};

#[derive(Clone)]
pub struct SqliteMusicPieceRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqliteMusicPieceRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MusicPieceRepositoryPort for SqliteMusicPieceRepo {
    async fn save_piece(&self, piece: &MusicPiece) -> Result<(), String> {
        let mut db = self.db.lock().await;

        match MusicPieceRow::get_by_id(&mut *db, piece.id.as_str()).await {
            Ok(mut current) => {
                current
                    .update()
                    .title(piece.title.clone())
                    .creator(piece.creator.clone())
                    .piece(toasty::Json(piece.clone()))
                    .created_at(piece.created_at.clone())
                    .updated_at(piece.updated_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(MusicPieceRow {
                    id: piece.id.as_str().to_string(),
                    title: piece.title.clone(),
                    creator: piece.creator.clone(),
                    piece: toasty::Json(piece.clone()),
                    created_at: piece.created_at.clone(),
                    updated_at: piece.updated_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }

        Ok(())
    }

    async fn find_piece(&self, id: &MusicPieceId) -> Result<Option<MusicPiece>, String> {
        let mut db = self.db.lock().await;

        match MusicPieceRow::get_by_id(&mut *db, id.as_str()).await {
            Ok(row) => Ok(Some(row.piece.0)),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn list_pieces(&self) -> Result<Vec<MusicPiece>, String> {
        let mut db = self.db.lock().await;
        let rows = MusicPieceRow::filter(MusicPieceRow::fields().id().ne(""))
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())?;

        let mut pieces: Vec<_> = rows.into_iter().map(|row| row.piece.0).collect();
        pieces.sort_by(|a, b| a.title.cmp(&b.title).then(a.id.as_str().cmp(b.id.as_str())));
        Ok(pieces)
    }
}

#[derive(Clone)]
pub struct SqlitePracticeSessionRepo {
    db: Arc<Mutex<toasty::Db>>,
}

impl SqlitePracticeSessionRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PracticeSessionRepositoryPort for SqlitePracticeSessionRepo {
    async fn save_session(&self, session: &PracticeSession) -> Result<(), String> {
        let mut db = self.db.lock().await;

        match PracticeSessionRow::get_by_id(&mut *db, session.id.as_str()).await {
            Ok(mut current) => {
                current
                    .update()
                    .piece_id(session.piece_id.as_str().to_string())
                    .session(toasty::Json(session.clone()))
                    .started_at(session.started_at.clone())
                    .ended_at(session.ended_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(PracticeSessionRow {
                    id: session.id.as_str().to_string(),
                    piece_id: session.piece_id.as_str().to_string(),
                    session: toasty::Json(session.clone()),
                    started_at: session.started_at.clone(),
                    ended_at: session.ended_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }

        Ok(())
    }

    async fn find_session(
        &self,
        id: &PracticeSessionId,
    ) -> Result<Option<PracticeSession>, String> {
        let mut db = self.db.lock().await;

        match PracticeSessionRow::get_by_id(&mut *db, id.as_str()).await {
            Ok(row) => Ok(Some(row.session.0)),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn list_sessions_by_piece(
        &self,
        piece_id: &MusicPieceId,
    ) -> Result<Vec<PracticeSession>, String> {
        let mut db = self.db.lock().await;
        let rows = PracticeSessionRow::filter(
            PracticeSessionRow::fields().piece_id().eq(piece_id.as_str()),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        let mut sessions: Vec<_> = rows.into_iter().map(|row| row.session.0).collect();
        sessions.sort_by(|a, b| {
            a.started_at
                .cmp(&b.started_at)
                .then(a.id.as_str().cmp(b.id.as_str()))
        });
        Ok(sessions)
    }
}
