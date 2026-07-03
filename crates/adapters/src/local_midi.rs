use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use app::music::{
    DiscoveredMidiFile, LocalMidiScannerPort, LocalMidiScoreParserPort, LocalMidiWatcherPort,
};
use async_trait::async_trait;
use domain::{
    BeatPosition, BeatSpan, Meter, Note, PianoScore, Pitch, ScorePart, Tempo,
};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Clone)]
pub struct LocalMidiFileAdapter {
    runtime: Arc<Mutex<WatchRuntime>>,
}

impl LocalMidiFileAdapter {
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(Mutex::new(WatchRuntime::new())),
        }
    }
}

impl Default for LocalMidiFileAdapter {
    fn default() -> Self {
        Self::new()
    }
}

struct WatchRuntime {
    directories: BTreeSet<PathBuf>,
    watchers: Vec<RecommendedWatcher>,
    dirty: bool,
}

impl WatchRuntime {
    fn new() -> Self {
        Self {
            directories: BTreeSet::new(),
            watchers: Vec::new(),
            dirty: false,
        }
    }
}

#[async_trait]
impl LocalMidiScannerPort for LocalMidiFileAdapter {
    async fn scan_directory(&self, directory: &str) -> Result<Vec<DiscoveredMidiFile>, String> {
        let directory = normalize_directory(directory)?;
        let mut files = Vec::new();
        collect_midi_files(&directory, &mut files)?;
        files.sort();

        Ok(files
            .into_iter()
            .map(|path| DiscoveredMidiFile {
                title: midi_title(&path),
                fingerprint: midi_fingerprint(&path),
                path: path.display().to_string(),
            })
            .collect())
    }
}

#[async_trait]
impl LocalMidiWatcherPort for LocalMidiFileAdapter {
    async fn watch_directory(&self, directory: &str) -> Result<(), String> {
        let directory = normalize_directory(directory)?;
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "local midi watcher lock poisoned".to_string())?;

        if runtime.directories.contains(&directory) {
            return Ok(());
        }

        let runtime_for_event = self.runtime.clone();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if event.is_ok() {
                if let Ok(mut runtime) = runtime_for_event.lock() {
                    runtime.dirty = true;
                }
            }
        })
        .map_err(|err| err.to_string())?;

        watcher
            .watch(&directory, RecursiveMode::Recursive)
            .map_err(|err| err.to_string())?;

        runtime.directories.insert(directory);
        runtime.watchers.push(watcher);
        runtime.dirty = true;

        Ok(())
    }

    async fn watched_directories(&self) -> Result<Vec<String>, String> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "local midi watcher lock poisoned".to_string())?;
        Ok(runtime
            .directories
            .iter()
            .map(|path| path.display().to_string())
            .collect())
    }

    async fn is_dirty(&self) -> Result<bool, String> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| "local midi watcher lock poisoned".to_string())?;
        Ok(runtime.dirty)
    }

    async fn clear_dirty(&self) -> Result<(), String> {
        let mut runtime = self
            .runtime
            .lock()
            .map_err(|_| "local midi watcher lock poisoned".to_string())?;
        runtime.dirty = false;
        Ok(())
    }
}

#[async_trait]
impl LocalMidiScoreParserPort for LocalMidiFileAdapter {
    async fn parse_score(&self, path: &str) -> Result<PianoScore, String> {
        let bytes = fs::read(path).map_err(|err| err.to_string())?;
        parse_midi_score(&bytes)
    }
}

fn normalize_directory(directory: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(directory.trim());
    if path.as_os_str().is_empty() {
        return Err("directory path is empty".to_string());
    }

    let path = fs::canonicalize(&path).map_err(|err| err.to_string())?;
    if !path.is_dir() {
        return Err(format!("path is not a directory: {}", path.display()));
    }

    Ok(path)
}

fn collect_midi_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| err.to_string())?;

        if file_type.is_dir() {
            collect_midi_files(&path, files)?;
        } else if file_type.is_file() && is_valid_midi_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_valid_midi_file(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    if !matches!(extension.to_ascii_lowercase().as_str(), "mid" | "midi") {
        return false;
    }

    fs::read(path)
        .map(|bytes| bytes.starts_with(b"MThd"))
        .unwrap_or(false)
}

fn midi_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn midi_fingerprint(path: &Path) -> String {
    let metadata = fs::metadata(path).ok();
    let len = metadata.as_ref().map(|item| item.len()).unwrap_or_default();
    let modified = metadata
        .and_then(|item| item.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    format!("{}:{len}:{modified}", path.display())
}

fn parse_midi_score(bytes: &[u8]) -> Result<PianoScore, String> {
    let smf = Smf::parse(bytes).map_err(|err| err.to_string())?;
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(ticks) => ticks.as_int() as f32,
        Timing::Timecode(_, _) => {
            return Err("SMPTE timecode MIDI timing is not supported yet".to_string());
        }
    };

    if ticks_per_beat <= 0.0 {
        return Err("MIDI ticks per beat is invalid".to_string());
    }

    let mut score = PianoScore::default();

    for (track_index, track) in smf.tracks.iter().enumerate() {
        let mut absolute_ticks = 0u64;
        let mut track_name = format!("Track {}", track_index + 1);
        let mut active_notes: HashMap<(u8, u8), Vec<(u64, u8)>> = HashMap::new();
        let mut notes = Vec::new();

        for event in track {
            absolute_ticks = absolute_ticks.saturating_add(event.delta.as_int() as u64);
            match event.kind {
                TrackEventKind::Meta(MetaMessage::TrackName(name)) => {
                    if let Ok(value) = std::str::from_utf8(name) {
                        let value = value.trim();
                        if !value.is_empty() {
                            track_name = value.to_string();
                        }
                    }
                }
                TrackEventKind::Meta(MetaMessage::Tempo(microseconds_per_quarter)) => {
                    let micros = microseconds_per_quarter.as_int();
                    if micros > 0 {
                        score.tempos.push(Tempo::new(
                            BeatPosition::new(ticks_to_beats(absolute_ticks, ticks_per_beat)),
                            60_000_000.0 / micros as f32,
                        ));
                    }
                }
                TrackEventKind::Meta(MetaMessage::TimeSignature(
                    numerator,
                    denominator_power,
                    _,
                    _,
                )) => {
                    let denominator = 2u8.saturating_pow(denominator_power as u32).max(1);
                    score.meters.push(Meter::new(numerator, denominator));
                }
                TrackEventKind::Midi { channel, message } => match message {
                    MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
                        active_notes
                            .entry((channel.as_int(), key.as_int()))
                            .or_default()
                            .push((absolute_ticks, vel.as_int()));
                    }
                    MidiMessage::NoteOff { key, vel: _ } | MidiMessage::NoteOn { key, vel: _ } => {
                        let note_key = (channel.as_int(), key.as_int());
                        let Some(starts) = active_notes.get_mut(&note_key) else {
                            continue;
                        };
                        let Some((start_ticks, velocity)) = starts.pop() else {
                            continue;
                        };
                        let duration_ticks = absolute_ticks.saturating_sub(start_ticks);
                        if duration_ticks == 0 {
                            continue;
                        }
                        notes.push(Note {
                            pitch: Pitch::new_unchecked(key.as_int()),
                            span: BeatSpan::new(
                                BeatPosition::new(ticks_to_beats(start_ticks, ticks_per_beat)),
                                duration_ticks as f32 / ticks_per_beat,
                            ),
                            velocity: Some(velocity),
                        });
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if !notes.is_empty() {
            notes.sort_by(|a, b| {
                a.span
                    .start
                    .beats
                    .partial_cmp(&b.span.start.beats)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.pitch.midi_number().cmp(&b.pitch.midi_number()))
            });
            score.parts.push(ScorePart {
                name: track_name,
                tonality: None,
                notes,
            });
        }
    }

    if score.tempos.is_empty() {
        score.tempos.push(Tempo::new(BeatPosition::new(0.0), 120.0));
    }
    if score.meters.is_empty() {
        score.meters.push(Meter::new(4, 4));
    }

    if score.parts.iter().all(|part| part.notes.is_empty()) {
        return Err("MIDI contains no note events".to_string());
    }

    Ok(score)
}

fn ticks_to_beats(ticks: u64, ticks_per_beat: f32) -> f32 {
    ticks as f32 / ticks_per_beat
}
