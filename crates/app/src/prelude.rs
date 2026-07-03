//! Application use cases and service traits.

pub use crate::app_error::{AppError, AppResult};
pub use crate::learning::{
    AbandonPracticeSessionCommand, CompletePracticeSessionCommand, LearningCommandHandler,
    LearningQueryHandler, PracticeJudge, PracticeSessionRepositoryPort,
    RecordPracticeAttemptCommand, RecordPracticeAttemptResult, StartPracticeSessionCommand,
    StartPracticeSessionResult,
};
pub use crate::music::{
    CreateMusicPieceCommand, CreateMusicPieceResult, ImportPianoArrangementCommand,
    ImportPianoArrangementResult, MusicCommandHandler, MusicEventHandler, MusicPieceRepositoryPort,
    MusicQueryHandler,
};
