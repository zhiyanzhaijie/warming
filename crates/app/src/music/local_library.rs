use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use domain::{ArrangementId, MusicPiece, MusicPieceId, PianoScore};

use crate::app_error::{AppError, AppResult};
use crate::music::{
    CreateMusicPieceCommand, ImportPianoArrangementCommand, MusicCommandHandler,
    MusicPieceRepositoryPort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredMidiFile {
    pub path: String,
    pub title: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiScanReport {
    pub watched_directories: Vec<String>,
    pub discovered_files: usize,
    pub registered_files: usize,
}

#[derive(Debug, Clone)]
pub struct RegisterLocalMidiFileCommand {
    pub file: DiscoveredMidiFile,
    pub discovered_at: String,
}

#[async_trait]
pub trait LocalMidiScannerPort: Send + Sync {
    async fn scan_directory(&self, directory: &str) -> Result<Vec<DiscoveredMidiFile>, String>;
}

#[async_trait]
pub trait LocalMidiWatcherPort: Send + Sync {
    async fn watch_directory(&self, directory: &str) -> Result<(), String>;

    async fn watched_directories(&self) -> Result<Vec<String>, String>;

    async fn is_dirty(&self) -> Result<bool, String>;

    async fn clear_dirty(&self) -> Result<(), String>;
}

#[async_trait]
pub trait LocalMidiScoreParserPort: Send + Sync {
    async fn parse_score(&self, path: &str) -> Result<PianoScore, String>;
}

#[derive(Clone)]
pub struct LocalMidiLibraryHandler {
    scanner: Arc<dyn LocalMidiScannerPort>,
    watcher: Arc<dyn LocalMidiWatcherPort>,
    parser: Arc<dyn LocalMidiScoreParserPort>,
    pieces: Arc<dyn MusicPieceRepositoryPort>,
    music_command: MusicCommandHandler,
}

impl LocalMidiLibraryHandler {
    pub fn new(
        scanner: Arc<dyn LocalMidiScannerPort>,
        watcher: Arc<dyn LocalMidiWatcherPort>,
        parser: Arc<dyn LocalMidiScoreParserPort>,
        pieces: Arc<dyn MusicPieceRepositoryPort>,
        music_command: MusicCommandHandler,
    ) -> Self {
        Self {
            scanner,
            watcher,
            parser,
            pieces,
            music_command,
        }
    }

    pub async fn add_watch_directory(&self, directory: String) -> AppResult<MidiScanReport> {
        self.add_watch_directories(vec![directory]).await
    }

    pub async fn add_watch_directories(
        &self,
        directories: Vec<String>,
    ) -> AppResult<MidiScanReport> {
        if directories.is_empty() {
            return self.refresh_watched_directories().await;
        }

        for directory in directories {
            self.watcher
                .watch_directory(&directory)
                .await
                .map_err(AppError::upstream)?;
        }

        self.refresh_watched_directories().await
    }

    pub async fn list_watch_directories(&self) -> AppResult<Vec<String>> {
        self.watcher
            .watched_directories()
            .await
            .map_err(AppError::upstream)
    }

    pub async fn refresh_if_dirty(&self) -> AppResult<Option<MidiScanReport>> {
        if self.watcher.is_dirty().await.map_err(AppError::upstream)? {
            self.refresh_watched_directories().await.map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn refresh_watched_directories(&self) -> AppResult<MidiScanReport> {
        let directories = self
            .watcher
            .watched_directories()
            .await
            .map_err(AppError::upstream)?;
        let mut discovered_files = 0usize;
        let mut registered_files = 0usize;

        for directory in &directories {
            let files = self
                .scanner
                .scan_directory(directory)
                .await
                .map_err(AppError::upstream)?;
            discovered_files += files.len();

            for file in files {
                let result = self
                    .register_local_midi_file(RegisterLocalMidiFileCommand {
                        file,
                        discovered_at: unix_timestamp_seconds(),
                    })
                    .await?;
                if result.created {
                    registered_files += 1;
                }
            }
        }

        self.watcher
            .clear_dirty()
            .await
            .map_err(AppError::upstream)?;

        Ok(MidiScanReport {
            watched_directories: directories,
            discovered_files,
            registered_files,
        })
    }

    pub async fn register_local_midi_file(
        &self,
        input: RegisterLocalMidiFileCommand,
    ) -> AppResult<RegisterLocalMidiFileResult> {
        let piece_id = local_midi_piece_id(&input.file.fingerprint);
        let arrangement_id = local_midi_arrangement_id(&input.file.fingerprint);
        if let Some(piece) = self
            .pieces
            .find_piece(&piece_id)
            .await
            .map_err(AppError::upstream)?
        {
            if piece
                .arrangements
                .iter()
                .any(|arrangement| arrangement.id == arrangement_id)
            {
                return Ok(RegisterLocalMidiFileResult {
                    piece,
                    created: false,
                });
            }

            let score = self
                .parser
                .parse_score(&input.file.path)
                .await
                .map_err(AppError::upstream)?;
            let result = self
                .music_command
                .import_arrangement(ImportPianoArrangementCommand {
                    piece_id,
                    arrangement_id,
                    title: "MIDI import".to_string(),
                    score,
                    now: input.discovered_at,
                })
                .await?;

            return Ok(RegisterLocalMidiFileResult {
                piece: result.piece,
                created: false,
            });
        }

        let score = self
            .parser
            .parse_score(&input.file.path)
            .await
            .map_err(AppError::upstream)?;
        self.music_command
            .create_piece(CreateMusicPieceCommand {
                piece_id: piece_id.clone(),
                title: input.file.title,
                creator: Some(input.file.path),
                now: input.discovered_at.clone(),
            })
            .await?;
        let result = self
            .music_command
            .import_arrangement(ImportPianoArrangementCommand {
                piece_id,
                arrangement_id,
                title: "MIDI import".to_string(),
                score,
                now: input.discovered_at,
            })
            .await?;

        Ok(RegisterLocalMidiFileResult {
            piece: result.piece,
            created: true,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RegisterLocalMidiFileResult {
    pub piece: MusicPiece,
    pub created: bool,
}

fn local_midi_piece_id(fingerprint: &str) -> MusicPieceId {
    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    MusicPieceId::new_unchecked(format!("midi-{:016x}", hasher.finish()))
}

fn local_midi_arrangement_id(fingerprint: &str) -> ArrangementId {
    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    ArrangementId::new_unchecked(format!("midi-arrangement-{:016x}", hasher.finish()))
}

fn unix_timestamp_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
