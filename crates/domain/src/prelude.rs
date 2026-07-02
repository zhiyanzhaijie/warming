//! Domain types shared across the application.
pub use crate::{
    ArrangementId, AttemptId, BeatPosition, BeatSpan, Exercise, ExerciseId, Judgement,
    JudgementKind, KeySignature, LearningIdInvalidError, Meter, MusicIdInvalidError, MusicPiece,
    MusicPieceCreated, MusicPieceId, Note, PianoArrangement, PianoScore, Pitch,
    PitchInvalidError, PracticeAttempt, PracticeSession, PracticeSessionId,
    PracticeSessionStatus, Practiced, ScorePart, Tempo, Tonality, UserPerformance,
};
