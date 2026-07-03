use serde::{Deserialize, Serialize};

use crate::Pitch;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PracticeSessionId(String);

impl PracticeSessionId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, LearningIdInvalidError> {
        parse_id(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PracticeSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExerciseId(String);

impl ExerciseId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, LearningIdInvalidError> {
        parse_id(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ExerciseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttemptId(String);

impl AttemptId {
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: &str) -> Result<Self, LearningIdInvalidError> {
        parse_id(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AttemptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn parse_id(value: &str) -> Result<String, LearningIdInvalidError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(LearningIdInvalidError {
            value: value.to_string(),
        });
    }
    Ok(value.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningIdInvalidError {
    value: String,
}

impl std::fmt::Display for LearningIdInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "learning: invalid id: '{}'", self.value)
    }
}

impl std::error::Error for LearningIdInvalidError {}

impl crate::error::DomainError for LearningIdInvalidError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PracticeSessionStatus {
    Active,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Judgement {
    pub kind: JudgementKind,
    pub timing_offset_beats: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JudgementKind {
    Hit,
    Missed,
    WrongPitch,
    Extra,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayableRange {
    pub lowest: Pitch,
    pub highest: Pitch,
}

impl PlayableRange {
    pub fn new(lowest: Pitch, highest: Pitch) -> Result<Self, PlayableRangeInvalidError> {
        if lowest.midi_number() > highest.midi_number() {
            return Err(PlayableRangeInvalidError {
                lowest: lowest.midi_number(),
                highest: highest.midi_number(),
            });
        }
        Ok(Self { lowest, highest })
    }

    pub fn contains(&self, pitch: Pitch) -> bool {
        let pitch = pitch.midi_number();
        self.lowest.midi_number() <= pitch && pitch <= self.highest.midi_number()
    }

    pub fn key_count(&self) -> u8 {
        self.highest.midi_number() - self.lowest.midi_number() + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayableRangeInvalidError {
    lowest: u8,
    highest: u8,
}

impl std::fmt::Display for PlayableRangeInvalidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "learning: invalid playable range: lowest '{}' is above highest '{}'",
            self.lowest, self.highest
        )
    }
}

impl std::error::Error for PlayableRangeInvalidError {}

impl crate::error::DomainError for PlayableRangeInvalidError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PracticeInstrument {
    pub playable_range: PlayableRange,
}

impl PracticeInstrument {
    pub fn new(playable_range: PlayableRange) -> Self {
        Self { playable_range }
    }
}
