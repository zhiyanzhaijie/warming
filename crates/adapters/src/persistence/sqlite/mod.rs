mod db;
mod models;
mod repos;

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::persistence::DbRepos;

pub async fn build_repos(database_url: &str) -> Result<DbRepos, String> {
    let db = Arc::new(Mutex::new(
        db::connect_sqlite(database_url)
            .await
            .map_err(|err| err.to_string())?,
    ));
    let music_piece_repo = Arc::new(repos::SqliteMusicPieceRepo::new(db.clone()));
    let practice_session_repo = Arc::new(repos::SqlitePracticeSessionRepo::new(db.clone()));

    Ok(DbRepos {
        db,
        music_piece_repo,
        practice_session_repo,
    })
}
