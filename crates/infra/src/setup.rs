//! Application container setup.

use app::app_error::{AppError, AppResult};
use app::learning::{LearningCommandHandler, LearningEventHandler, LearningQueryHandler};
use app::music::{
    LocalMidiLibraryHandler, MusicCommandHandler, MusicEventHandler, MusicQueryHandler,
};
use std::sync::Arc;

pub struct MusicState {
    pub command: MusicCommandHandler,
    pub query: MusicQueryHandler,
    pub local_library: LocalMidiLibraryHandler,
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

    let music_command = MusicCommandHandler::new(repos.music_piece_repo.clone())
        .with_event_handler(MusicEventHandler::new());
    let local_midi = Arc::new(adapters::local_midi::LocalMidiFileAdapter::new());

    let music = MusicState {
        command: music_command.clone(),
        query: MusicQueryHandler::new(repos.music_piece_repo.clone()),
        local_library: LocalMidiLibraryHandler::new(
            local_midi.clone(),
            local_midi.clone(),
            local_midi,
            repos.music_piece_repo.clone(),
            music_command,
        ),
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
