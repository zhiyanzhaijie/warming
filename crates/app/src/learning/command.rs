use std::sync::Arc;

use domain::{
    ArrangementId, AttemptId, BeatSpan, Exercise, ExerciseId, Judgement, JudgementKind,
    MusicPieceId, Note, PlayableRange, PracticeAttempt, PracticeSession, PracticeSessionId,
    PracticeSessionStatus, Practiced, UserPerformance,
};

use crate::app_error::{AppError, AppResult};
use crate::learning::{LearningEventHandler, PracticeSessionRepositoryPort};
use crate::music::MusicPieceRepositoryPort;

#[derive(Debug, Clone)]
pub struct StartPracticeSessionCommand {
    pub session_id: PracticeSessionId,
    pub piece_id: MusicPieceId,
    pub arrangement_id: ArrangementId,
    pub exercise_id: ExerciseId,
    pub segment: Option<BeatSpan>,
    pub target_speed: f32,
    pub playable_range: Option<PlayableRange>,
    pub started_at: String,
}

#[derive(Debug, Clone)]
pub struct RecordPracticeAttemptCommand {
    pub session_id: PracticeSessionId,
    pub attempt_id: AttemptId,
    pub performance: UserPerformance,
    pub timing_tolerance_beats: f32,
    pub recorded_at: String,
}

#[derive(Debug, Clone)]
pub struct CompletePracticeSessionCommand {
    pub session_id: PracticeSessionId,
    pub ended_at: String,
}

#[derive(Debug, Clone)]
pub struct AbandonPracticeSessionCommand {
    pub session_id: PracticeSessionId,
    pub ended_at: String,
}

#[derive(Debug, Clone)]
pub struct StartPracticeSessionResult {
    pub session: PracticeSession,
}

#[derive(Debug, Clone)]
pub struct RecordPracticeAttemptResult {
    pub session: PracticeSession,
    pub attempt: PracticeAttempt,
}

#[derive(Clone)]
pub struct LearningCommandHandler {
    sessions: Arc<dyn PracticeSessionRepositoryPort>,
    pieces: Arc<dyn MusicPieceRepositoryPort>,
    event_handler: Option<LearningEventHandler>,
}

impl LearningCommandHandler {
    pub fn new(
        sessions: Arc<dyn PracticeSessionRepositoryPort>,
        pieces: Arc<dyn MusicPieceRepositoryPort>,
    ) -> Self {
        Self {
            sessions,
            pieces,
            event_handler: None,
        }
    }

    pub fn with_event_handler(mut self, event_handler: LearningEventHandler) -> Self {
        self.event_handler = Some(event_handler);
        self
    }

    pub async fn start_session(
        &self,
        input: StartPracticeSessionCommand,
    ) -> AppResult<StartPracticeSessionResult> {
        if input.target_speed <= 0.0 {
            return Err(AppError::validation(
                "practice target speed must be positive",
            ));
        }

        let piece = self
            .pieces
            .find_piece(&input.piece_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("music piece".to_string()))?;

        if !piece
            .arrangements
            .iter()
            .any(|arrangement| arrangement.id == input.arrangement_id)
        {
            return Err(AppError::NotFound("piano arrangement".to_string()));
        }

        if self
            .sessions
            .find_session(&input.session_id)
            .await
            .map_err(AppError::upstream)?
            .is_some()
        {
            return Err(AppError::validation(format!(
                "practice session already exists: {}",
                input.session_id.as_str()
            )));
        }

        let session = PracticeSession {
            id: input.session_id,
            piece_id: input.piece_id,
            arrangement_id: input.arrangement_id,
            exercise: Exercise {
                id: input.exercise_id,
                segment: input.segment,
                target_speed: input.target_speed,
                playable_range: input.playable_range,
            },
            attempts: Vec::new(),
            status: PracticeSessionStatus::Active,
            started_at: input.started_at,
            ended_at: None,
        };

        self.sessions
            .save_session(&session)
            .await
            .map_err(AppError::upstream)?;

        Ok(StartPracticeSessionResult { session })
    }

    pub async fn record_attempt(
        &self,
        input: RecordPracticeAttemptCommand,
    ) -> AppResult<RecordPracticeAttemptResult> {
        let mut session = self
            .sessions
            .find_session(&input.session_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("practice session".to_string()))?;

        if session.status != PracticeSessionStatus::Active {
            return Err(AppError::validation("practice session is not active"));
        }

        let piece = self
            .pieces
            .find_piece(&session.piece_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("music piece".to_string()))?;

        let arrangement = piece
            .arrangements
            .iter()
            .find(|arrangement| arrangement.id == session.arrangement_id)
            .ok_or_else(|| AppError::NotFound("piano arrangement".to_string()))?;

        let target_notes = collect_target_notes(
            &arrangement.score.parts,
            session.exercise.segment,
            session.exercise.playable_range.as_ref(),
        );
        let judgements = PracticeJudge::judge(
            &target_notes,
            &input.performance,
            input.timing_tolerance_beats.max(0.0),
        );
        let attempt = PracticeAttempt {
            id: input.attempt_id,
            performance: input.performance,
            judgements,
            recorded_at: input.recorded_at,
        };

        session.attempts.push(attempt.clone());
        self.sessions
            .save_session(&session)
            .await
            .map_err(AppError::upstream)?;

        if let Some(event_handler) = &self.event_handler {
            event_handler
                .handle_practiced(&Practiced {
                    session_id: session.id.clone(),
                    piece_id: session.piece_id.clone(),
                })
                .await?;
        }

        Ok(RecordPracticeAttemptResult { session, attempt })
    }

    pub async fn complete_session(
        &self,
        input: CompletePracticeSessionCommand,
    ) -> AppResult<PracticeSession> {
        self.finish_session(
            input.session_id,
            PracticeSessionStatus::Completed,
            input.ended_at,
        )
        .await
    }

    pub async fn abandon_session(
        &self,
        input: AbandonPracticeSessionCommand,
    ) -> AppResult<PracticeSession> {
        self.finish_session(
            input.session_id,
            PracticeSessionStatus::Abandoned,
            input.ended_at,
        )
        .await
    }

    async fn finish_session(
        &self,
        session_id: PracticeSessionId,
        status: PracticeSessionStatus,
        ended_at: String,
    ) -> AppResult<PracticeSession> {
        let mut session = self
            .sessions
            .find_session(&session_id)
            .await
            .map_err(AppError::upstream)?
            .ok_or_else(|| AppError::NotFound("practice session".to_string()))?;

        if session.status != PracticeSessionStatus::Active {
            return Err(AppError::validation("practice session is already finished"));
        }

        session.status = status;
        session.ended_at = Some(ended_at);

        self.sessions
            .save_session(&session)
            .await
            .map_err(AppError::upstream)?;

        Ok(session)
    }
}

pub struct PracticeJudge;

impl PracticeJudge {
    pub fn judge(
        target_notes: &[Note],
        performance: &UserPerformance,
        timing_tolerance_beats: f32,
    ) -> Vec<Judgement> {
        let mut judgements = Vec::new();
        let mut matched_performance = vec![false; performance.notes.len()];

        for target in target_notes {
            let target_start = target.span.start.beats;
            let match_index = performance
                .notes
                .iter()
                .enumerate()
                .filter(|(index, performed)| {
                    !matched_performance[*index] && performed.pitch == target.pitch
                })
                .filter_map(|(index, performed)| {
                    let offset = performed.span.start.beats - target_start;
                    (offset.abs() <= timing_tolerance_beats).then_some((index, offset))
                })
                .min_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()));

            match match_index {
                Some((index, offset)) => {
                    matched_performance[index] = true;
                    judgements.push(Judgement {
                        kind: JudgementKind::Hit,
                        timing_offset_beats: offset,
                    });
                }
                None => judgements.push(Judgement {
                    kind: JudgementKind::Missed,
                    timing_offset_beats: 0.0,
                }),
            }
        }

        for (index, performed) in performance.notes.iter().enumerate() {
            if matched_performance[index] {
                continue;
            }

            let performed_start = performed.span.start.beats;
            let nearest_target_offset = target_notes
                .iter()
                .map(|target| performed_start - target.span.start.beats)
                .min_by(|left, right| left.abs().total_cmp(&right.abs()));

            let kind = nearest_target_offset
                .filter(|offset| offset.abs() <= timing_tolerance_beats)
                .map(|_| JudgementKind::WrongPitch)
                .unwrap_or(JudgementKind::Extra);

            judgements.push(Judgement {
                kind,
                timing_offset_beats: nearest_target_offset.unwrap_or(0.0),
            });
        }

        judgements
    }
}

fn collect_target_notes(
    parts: &[domain::ScorePart],
    segment: Option<BeatSpan>,
    playable_range: Option<&PlayableRange>,
) -> Vec<Note> {
    let mut notes = parts
        .iter()
        .flat_map(|part| part.notes.iter())
        .filter(|note| {
            playable_range
                .map(|range| range.contains(note.pitch))
                .unwrap_or(true)
        })
        .filter(|note| {
            segment
                .map(|segment| {
                    let start = note.span.start.beats;
                    let segment_start = segment.start.beats;
                    let segment_end = segment_start + segment.duration_beats;
                    segment_start <= start && start <= segment_end
                })
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    notes.sort_by(|left, right| left.span.start.beats.total_cmp(&right.span.start.beats));
    notes
}
