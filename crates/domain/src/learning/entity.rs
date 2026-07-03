use serde::{Deserialize, Serialize};

use super::{
    AttemptId, ExerciseId, Judgement, PlayableRange, PracticeSessionId, PracticeSessionStatus,
};
use crate::{ArrangementId, BeatSpan, MusicPieceId, Note};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PracticeSession {
    pub id: PracticeSessionId,
    pub piece_id: MusicPieceId,
    pub arrangement_id: ArrangementId,
    pub exercise: Exercise,
    pub attempts: Vec<PracticeAttempt>,
    pub status: PracticeSessionStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: ExerciseId,
    pub segment: Option<BeatSpan>,
    pub target_speed: f32,
    pub playable_range: Option<PlayableRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PracticeAttempt {
    pub id: AttemptId,
    pub performance: UserPerformance,
    pub judgements: Vec<Judgement>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UserPerformance {
    pub notes: Vec<Note>,
}
