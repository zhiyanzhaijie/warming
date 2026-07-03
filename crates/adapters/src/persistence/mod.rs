pub mod sqlite;

use std::sync::Arc;

use app::learning::PracticeSessionRepositoryPort;
use app::music::MusicPieceRepositoryPort;

pub struct DbRepos {
    pub db: Arc<tokio::sync::Mutex<toasty::Db>>,
    pub music_piece_repo: Arc<dyn MusicPieceRepositoryPort>,
    pub practice_session_repo: Arc<dyn PracticeSessionRepositoryPort>,
}

pub async fn build_repos_by_url(database_url: &str) -> Result<DbRepos, String> {
    sqlite::build_repos(database_url).await
}
