use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MusicPieceId(String);

impl MusicPieceId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, MusicIdInvalidError> {
        parse_id(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MusicPieceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArrangementId(String);

impl ArrangementId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, MusicIdInvalidError> {
        parse_id(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ArrangementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn parse_id(value: &str) -> Result<String, MusicIdInvalidError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(MusicIdInvalidError {
            value: value.to_string(),
        });
    }
    Ok(value.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicIdInvalidError {
    value: String,
}

impl std::fmt::Display for MusicIdInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "music: invalid id: '{}'", self.value)
    }
}

impl std::error::Error for MusicIdInvalidError {}

impl crate::error::DomainError for MusicIdInvalidError {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Pitch(u8);

impl Pitch {
    pub fn new_unchecked(value: u8) -> Self {
        Self(value)
    }

    pub fn parse(value: i16) -> Result<Self, PitchInvalidError> {
        if !(0..=127).contains(&value) {
            return Err(PitchInvalidError { value });
        }
        Ok(Self(value as u8))
    }

    pub fn midi_number(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PitchInvalidError {
    value: i16,
}

impl std::fmt::Display for PitchInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "music: invalid pitch: '{}'", self.value)
    }
}

impl std::error::Error for PitchInvalidError {}

impl crate::error::DomainError for PitchInvalidError {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BeatPosition {
    pub beats: f32,
}

impl BeatPosition {
    pub fn new(beats: f32) -> Self {
        Self {
            beats: beats.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BeatSpan {
    pub start: BeatPosition,
    pub duration_beats: f32,
}

impl BeatSpan {
    pub fn new(start: BeatPosition, duration_beats: f32) -> Self {
        Self {
            start,
            duration_beats: duration_beats.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Tempo {
    pub at: BeatPosition,
    pub bpm: f32,
}

impl Tempo {
    pub fn new(at: BeatPosition, bpm: f32) -> Self {
        Self {
            at,
            bpm: bpm.max(1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Meter {
    pub numerator: u8,
    pub denominator: u8,
}

impl Meter {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator: numerator.max(1),
            denominator: denominator.max(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeySignature {
    pub tonic: String,
    pub tonality: Tonality,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tonality {
    Major,
    Minor,
}
