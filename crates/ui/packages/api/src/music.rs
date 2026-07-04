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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ScorePreviewDTO {
    pub piece_id: String,
    pub arrangement_id: String,
    pub title: String,
    pub lowest_pitch: u8,
    pub highest_pitch: u8,
    pub total_beats: f32,
    pub bpm: f32,
    pub notes: Vec<FallingNoteDTO>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FallingNoteDTO {
    pub pitch: u8,
    pub start_beats: f32,
    pub duration_beats: f32,
    pub velocity: Option<u8>,
    pub part_name: String,
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
    let _ = crate::watch::refresh_if_dirty().await?;

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

pub async fn get_score_preview(
    piece_id: &str,
    arrangement_id: &str,
) -> ApiResult<Option<ScorePreviewDTO>> {
    let state = local_state::state().await?;
    let piece_id = MusicPieceId::parse(piece_id).map_err(ApiError::new)?;
    let arrangement_id = ArrangementId::parse(arrangement_id).map_err(ApiError::new)?;
    let arrangement = state
        .0
        .music
        .query
        .get_arrangement(&piece_id, &arrangement_id)
        .await
        .map_err(ApiError::from)?;

    Ok(arrangement.map(|arrangement| {
        let mut notes = Vec::new();
        for part in &arrangement.score.parts {
            for note in &part.notes {
                notes.push(FallingNoteDTO {
                    pitch: note.pitch.midi_number(),
                    start_beats: note.span.start.beats,
                    duration_beats: note.span.duration_beats,
                    velocity: note.velocity,
                    part_name: part.name.clone(),
                });
            }
        }

        notes.sort_by(|a, b| {
            a.start_beats
                .partial_cmp(&b.start_beats)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.pitch.cmp(&b.pitch))
        });

        let lowest_pitch = notes.iter().map(|note| note.pitch).min().unwrap_or(21);
        let highest_pitch = notes.iter().map(|note| note.pitch).max().unwrap_or(108);
        let total_beats = notes
            .iter()
            .map(|note| note.start_beats + note.duration_beats)
            .fold(0.0, f32::max)
            .max(1.0);
        let bpm = arrangement
            .score
            .tempos
            .first()
            .map(|tempo| tempo.bpm)
            .unwrap_or(120.0);

        ScorePreviewDTO {
            piece_id: arrangement.piece_id.as_str().to_string(),
            arrangement_id: arrangement.id.as_str().to_string(),
            title: arrangement.title,
            lowest_pitch,
            highest_pitch,
            total_beats,
            bpm,
            notes,
        }
    }))
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
