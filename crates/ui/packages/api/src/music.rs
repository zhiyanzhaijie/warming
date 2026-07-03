use app::music::{CreateMusicPieceCommand, ImportPianoArrangementCommand};
use domain::{
    ArrangementId, BeatPosition, BeatSpan, Meter, MusicPiece, MusicPieceId, Note, PianoScore,
    Pitch, ScorePart, Tempo,
};
use serde::{Deserialize, Serialize};

use crate::{local_state, ApiError, ApiResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MusicPieceDTO {
    pub id: String,
    pub title: String,
    pub creator: Option<String>,
    pub arrangement_count: usize,
    pub note_count: usize,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArrangementDTO {
    pub id: String,
    pub piece_id: String,
    pub title: String,
    pub part_count: usize,
    pub note_count: usize,
    pub first_tempo_bpm: Option<f32>,
    pub tempo_label: String,
}

pub async fn ensure_demo_piece() -> ApiResult<MusicPieceDTO> {
    let state = local_state::state().await?;
    let piece_id = MusicPieceId::new_unchecked("demo-twinkle");
    let arrangement_id = ArrangementId::new_unchecked("demo-twinkle-piano");

    if state
        .0
        .music
        .query
        .get_piece(&piece_id)
        .await
        .map_err(ApiError::from)?
        .is_none()
    {
        state
            .0
            .music
            .command
            .create_piece(CreateMusicPieceCommand {
                piece_id: piece_id.clone(),
                title: "Twinkle Practice Demo".to_string(),
                creator: Some("warming".to_string()),
                now: "2026-07-03T00:00:00Z".to_string(),
            })
            .await
            .map_err(ApiError::from)?;
    }

    let piece = state
        .0
        .music
        .query
        .get_piece(&piece_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new("demo piece was not created"))?;

    if !piece
        .arrangements
        .iter()
        .any(|arrangement| arrangement.id == arrangement_id)
    {
        state
            .0
            .music
            .command
            .import_arrangement(ImportPianoArrangementCommand {
                piece_id: piece_id.clone(),
                arrangement_id,
                title: "Right hand starter".to_string(),
                score: demo_score(),
                now: "2026-07-03T00:01:00Z".to_string(),
            })
            .await
            .map_err(ApiError::from)?;
    }

    get_piece(piece_id.as_str())
        .await?
        .ok_or_else(|| ApiError::new("demo piece was not found after import"))
}

pub async fn list_pieces() -> ApiResult<Vec<MusicPieceDTO>> {
    let state = local_state::state().await?;
    let pieces = state
        .0
        .music
        .query
        .list_pieces()
        .await
        .map_err(ApiError::from)?;

    Ok(pieces.into_iter().map(piece_to_dto).collect())
}

pub async fn get_piece(id: &str) -> ApiResult<Option<MusicPieceDTO>> {
    let state = local_state::state().await?;
    let id = MusicPieceId::parse(id).map_err(ApiError::new)?;
    let piece = state
        .0
        .music
        .query
        .get_piece(&id)
        .await
        .map_err(ApiError::from)?;

    Ok(piece.map(piece_to_dto))
}

pub async fn list_arrangements(piece_id: &str) -> ApiResult<Vec<ArrangementDTO>> {
    let state = local_state::state().await?;
    let piece_id = MusicPieceId::parse(piece_id).map_err(ApiError::new)?;
    let piece = state
        .0
        .music
        .query
        .get_piece(&piece_id)
        .await
        .map_err(ApiError::from)?;

    Ok(piece
        .map(|piece| {
            piece.arrangements
                .into_iter()
                .map(|arrangement| {
                    let note_count = score_note_count(&arrangement.score);
                    ArrangementDTO {
                        id: arrangement.id.as_str().to_string(),
                        piece_id: arrangement.piece_id.as_str().to_string(),
                        title: arrangement.title,
                        part_count: arrangement.score.parts.len(),
                        note_count,
                        first_tempo_bpm: arrangement.score.tempos.first().map(|tempo| tempo.bpm),
                        tempo_label: arrangement
                            .score
                            .tempos
                            .first()
                            .map(|tempo| format!("{:.0} BPM", tempo.bpm))
                            .unwrap_or_else(|| "No tempo".to_string()),
                    }
                })
                .collect()
        })
        .unwrap_or_default())
}

fn piece_to_dto(piece: MusicPiece) -> MusicPieceDTO {
    MusicPieceDTO {
        id: piece.id.as_str().to_string(),
        title: piece.title,
        creator: piece.creator,
        arrangement_count: piece.arrangements.len(),
        note_count: piece
            .arrangements
            .iter()
            .map(|arrangement| score_note_count(&arrangement.score))
            .sum(),
        updated_at: piece.updated_at,
    }
}

fn score_note_count(score: &PianoScore) -> usize {
    score.parts.iter().map(|part| part.notes.len()).sum()
}

fn demo_score() -> PianoScore {
    let pitches = [60, 60, 67, 67, 69, 69, 67, 65, 65, 64, 64, 62, 62, 60];
    let notes = pitches
        .into_iter()
        .enumerate()
        .map(|(index, pitch)| Note {
            pitch: Pitch::new_unchecked(pitch),
            span: BeatSpan::new(BeatPosition::new(index as f32), 1.0),
            velocity: Some(90),
        })
        .collect();

    PianoScore {
        parts: vec![ScorePart {
            name: "Right hand".to_string(),
            tonality: None,
            notes,
        }],
        tempos: vec![Tempo::new(BeatPosition::new(0.0), 84.0)],
        meters: vec![Meter::new(4, 4)],
        key_signatures: Vec::new(),
    }
}
