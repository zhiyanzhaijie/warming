use serde::{Deserialize, Serialize};

use super::{
    ArrangementId, BeatSpan, KeySignature, Meter, MusicPieceId, Pitch, Tempo, Tonality,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MusicPiece {
    pub id: MusicPieceId,
    pub title: String,
    pub creator: Option<String>,
    pub arrangements: Vec<PianoArrangement>,
    pub created_at: String,
    pub updated_at: String,
}

impl MusicPiece {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: MusicPieceId::new_unchecked(id),
            title: title.into(),
            creator: None,
            arrangements: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PianoArrangement {
    pub id: ArrangementId,
    pub piece_id: MusicPieceId,
    pub title: String,
    pub score: PianoScore,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PianoScore {
    pub parts: Vec<ScorePart>,
    pub tempos: Vec<Tempo>,
    pub meters: Vec<Meter>,
    pub key_signatures: Vec<KeySignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScorePart {
    pub name: String,
    pub tonality: Option<Tonality>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub pitch: Pitch,
    pub span: BeatSpan,
    pub velocity: Option<u8>,
}
