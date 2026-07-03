//! Application container setup.

use app::app_error::{AppError, AppResult};
use app::learning::{LearningCommandHandler, LearningEventHandler, LearningQueryHandler};
use app::music::{MusicCommandHandler, MusicEventHandler, MusicQueryHandler};

pub struct MusicState {
    pub command: MusicCommandHandler,
    pub query: MusicQueryHandler,
}

pub struct LearningState {
    pub command: LearningCommandHandler,
    pub query: LearningQueryHandler,
}

pub struct AppContainer {
    pub music: MusicState,
    pub learning: LearningState,
    pub repos: adapters::persistence::DbRepos,
}

pub type AppState = AppContainer;

pub async fn init_app_container(database_url: &str) -> AppResult<AppContainer> {
    let repos = adapters::persistence::build_repos_by_url(database_url)
        .await
        .map_err(AppError::database)?;

    let music = MusicState {
        command: MusicCommandHandler::new(repos.music_piece_repo.clone())
            .with_event_handler(MusicEventHandler::new()),
        query: MusicQueryHandler::new(repos.music_piece_repo.clone()),
    };

    let learning = LearningState {
        command: LearningCommandHandler::new(
            repos.practice_session_repo.clone(),
            repos.music_piece_repo.clone(),
        )
        .with_event_handler(LearningEventHandler::new()),
        query: LearningQueryHandler::new(repos.practice_session_repo.clone()),
    };

    Ok(AppContainer {
        music,
        learning,
        repos,
    })
}
