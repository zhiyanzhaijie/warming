use app::learning::StartPracticeSessionCommand;
use domain::{ArrangementId, ExerciseId, MusicPieceId, PracticeSessionId};
use serde::{Deserialize, Serialize};

use crate::{local_state, ApiError, ApiResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PracticeSessionDTO {
    pub id: String,
    pub piece_id: String,
    pub arrangement_id: String,
    pub target_speed: f32,
    pub status: String,
    pub attempt_count: usize,
    pub started_at: String,
}

pub async fn start_demo_session(
    piece_id: String,
    arrangement_id: String,
) -> ApiResult<PracticeSessionDTO> {
    let state = local_state::state().await?;
    let session_id = PracticeSessionId::new_unchecked(format!(
        "session-{}",
        unix_timestamp_millis()
    ));

    let result = state
        .0
        .learning
        .command
        .start_session(StartPracticeSessionCommand {
            session_id,
            piece_id: MusicPieceId::parse(&piece_id).map_err(ApiError::new)?,
            arrangement_id: ArrangementId::parse(&arrangement_id).map_err(ApiError::new)?,
            exercise_id: ExerciseId::new_unchecked("full-score"),
            segment: None,
            target_speed: 1.0,
            playable_range: None,
            started_at: "2026-07-03T00:02:00Z".to_string(),
        })
        .await
        .map_err(ApiError::from)?;

    Ok(session_to_dto(result.session))
}

pub async fn list_sessions_by_piece(piece_id: &str) -> ApiResult<Vec<PracticeSessionDTO>> {
    let state = local_state::state().await?;
    let piece_id = MusicPieceId::parse(piece_id).map_err(ApiError::new)?;
    let sessions = state
        .0
        .learning
        .query
        .list_sessions_by_piece(&piece_id)
        .await
        .map_err(ApiError::from)?;

    Ok(sessions.into_iter().map(session_to_dto).collect())
}

fn session_to_dto(session: domain::PracticeSession) -> PracticeSessionDTO {
    PracticeSessionDTO {
        id: session.id.as_str().to_string(),
        piece_id: session.piece_id.as_str().to_string(),
        arrangement_id: session.arrangement_id.as_str().to_string(),
        target_speed: session.exercise.target_speed,
        status: format!("{:?}", session.status),
        attempt_count: session.attempts.len(),
        started_at: session.started_at,
    }
}

fn unix_timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
