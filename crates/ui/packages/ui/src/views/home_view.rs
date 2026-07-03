use dioxus::prelude::*;

#[component]
pub fn HomeView() -> Element {
    let nav = navigator();
    let mut selected_piece_id = use_signal(String::new);
    let mut action_message = use_signal(String::new);

    let mut pieces = use_resource(move || async move {
        let _ = api::music::ensure_demo_piece().await;
        api::music::list_pieces().await
    });

    let mut watched_directories = use_resource(move || async move {
        api::watch::list_watch_directories().await
    });

    let mut arrangements = use_resource(move || {
        let piece_id = selected_piece_id();
        async move {
            if piece_id.is_empty() {
                Ok(Vec::new())
            } else {
                api::music::list_arrangements(&piece_id).await
            }
        }
    });

    let mut score_preview = use_resource(move || {
        let piece_id = selected_piece_id();
        async move {
            if piece_id.is_empty() {
                return Ok(None);
            }
            let arrangements = api::music::list_arrangements(&piece_id).await?;
            let Some(arrangement) = arrangements.first() else {
                return Ok(None);
            };
            api::music::get_score_preview(&piece_id, &arrangement.id).await
        }
    });

    let mut sessions = use_resource(move || {
        let piece_id = selected_piece_id();
        async move {
            if piece_id.is_empty() {
                Ok(Vec::new())
            } else {
                api::learning::list_sessions_by_piece(&piece_id).await
            }
        }
    });

    let piece_state = pieces.read();
    let piece_result = piece_state.as_ref();
    let pieces_list = piece_result.and_then(|result| result.as_ref().ok());
    let selected_piece = pieces_list
        .and_then(|items| items.iter().find(|piece| piece.id == selected_piece_id()).cloned())
        .or_else(|| pieces_list.and_then(|items| items.first().cloned()));

    if selected_piece_id().is_empty() {
        if let Some(piece) = &selected_piece {
            selected_piece_id.set(piece.id.clone());
        }
    }

    rsx! {
        main { class: "min-h-screen bg-background text-foreground",
            div { class: "mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-6 px-6 py-6 md:px-8",
                header { class: "flex flex-col gap-4 border-b border-border pb-5 md:flex-row md:items-center md:justify-between",
                    div { class: "space-y-2",
                        p { class: "text-xs font-bold uppercase text-chart-3", "Local training workspace" }
                        h1 { class: "text-4xl font-black leading-none md:text-5xl", "Warming" }
                    }
                    div { class: "flex items-center gap-3 border border-border bg-card px-4 py-2 text-sm text-muted-foreground shadow-sm",
                        span { class: "h-2.5 w-2.5 rounded-full bg-chart-4 shadow-[0_0_18px_rgba(15,122,77,0.7)]" }
                        span { "SQLite local" }
                    }
                }

                section { class: "border border-border bg-card p-5 shadow-sm",
                    div { class: "flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between",
                        div { class: "space-y-2",
                            p { class: "text-xs font-bold uppercase text-chart-3", "Local folder watch" }
                            h2 { class: "text-xl font-black", "监听 MIDI 目录" }
                            p { class: "text-sm text-muted-foreground", "添加本地目录后，识别有效 .mid/.midi 文件并登记到曲库。" }
                        }
                        div { class: "flex w-full flex-col gap-2 sm:flex-row lg:w-auto",
                            button {
                                class: "h-10 border border-primary bg-primary px-4 text-sm font-bold text-primary-foreground disabled:cursor-not-allowed disabled:opacity-40",
                                disabled: !desktop_folder_picker_enabled(),
                                onclick: move |_| {
                                    spawn(async move {
                                        match pick_watch_directories().await {
                                            Some(directories) if !directories.is_empty() => {
                                                match api::watch::add_watch_directories(directories).await {
                                                    Ok(report) => {
                                                        action_message.set(format!(
                                                            "Watching {} folders · scanned {} MIDI files · registered {}",
                                                            report.watched_directories.len(),
                                                            report.discovered_files,
                                                            report.registered_files
                                                        ));
                                                        pieces.restart();
                                                        watched_directories.restart();
                                                    }
                                                    Err(err) => action_message.set(err.to_string()),
                                                }
                                            }
                                            Some(_) => action_message.set("No folder selected".to_string()),
                                            None => action_message.set("Folder picker is not available on this platform".to_string()),
                                        }
                                    });
                                },
                                "Choose folders"
                            }
                            button {
                                class: "h-10 border border-border bg-secondary px-4 text-sm font-bold text-secondary-foreground hover:bg-accent",
                                onclick: move |_| {
                                    spawn(async move {
                                        match api::watch::refresh_watched_directories().await {
                                            Ok(report) => {
                                                action_message.set(format!(
                                                    "Refreshed {} watched folders",
                                                    report.watched_directories.len()
                                                ));
                                                pieces.restart();
                                                watched_directories.restart();
                                            }
                                            Err(err) => action_message.set(err.to_string()),
                                        }
                                    });
                                },
                                "Scan"
                            }
                        }
                    }

                    match watched_directories.read().as_ref() {
                        Some(Ok(items)) if items.is_empty() => rsx! {
                            div { class: "mt-4 text-sm text-muted-foreground", "尚未添加监听目录" }
                        },
                        Some(Ok(items)) => rsx! {
                            div { class: "mt-4 flex flex-wrap gap-2",
                                for directory in items {
                                    span { class: "border border-border bg-background px-3 py-1 text-xs text-muted-foreground",
                                        "{directory}"
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => rsx! {
                            div { class: "mt-4 text-sm text-destructive", "{err}" }
                        },
                        None => rsx! {
                            div { class: "mt-4 text-sm text-muted-foreground", "Loading watched folders" }
                        },
                    }
                }

                section { class: "grid flex-1 gap-5 lg:grid-cols-[360px_minmax(0,1fr)]",
                    LibraryPane {
                        pieces_result: piece_result.cloned(),
                        selected_piece_id: selected_piece_id(),
                        on_select: move |id: String| {
                            selected_piece_id.set(id);
                            arrangements.restart();
                            score_preview.restart();
                            sessions.restart();
                        },
                        on_refresh: move |_| pieces.restart(),
                    }

                    section { class: "border border-border bg-card shadow-sm",
                        div { class: "flex flex-col gap-4 border-b border-border p-5 md:flex-row md:items-start md:justify-between",
                            div { class: "space-y-2",
                                p { class: "text-xs font-bold uppercase text-chart-3", "Selected score" }
                                h2 { class: "text-2xl font-black md:text-3xl",
                                    {selected_piece.as_ref().map(|piece| piece.title.as_str()).unwrap_or("No score selected")}
                                }
                            }
                            button {
                                class: "h-10 border border-primary bg-primary px-4 text-sm font-bold text-primary-foreground disabled:cursor-not-allowed disabled:opacity-40",
                                disabled: selected_piece.is_none(),
                                onclick: move |_| {
                                    let Some(piece) = selected_piece.clone() else {
                                        return;
                                    };
                                    spawn(async move {
                                        match api::music::list_arrangements(&piece.id).await {
                                            Ok(items) if !items.is_empty() => {
                                                let arrangement = items[0].clone();
                                                let route = format!("/practice/{}/{}", piece.id, arrangement.id);
                                                match api::learning::start_demo_session(piece.id, arrangement.id).await {
                                                    Ok(_) => {
                                                        nav.push(route);
                                                    }
                                                    Err(err) => action_message.set(err.to_string()),
                                                }
                                            }
                                            Ok(_) => action_message.set("No arrangement available".to_string()),
                                            Err(err) => action_message.set(err.to_string()),
                                        }
                                    });
                                },
                                "Start practice"
                            }
                        }

                        if !action_message().is_empty() {
                            div { class: "mx-5 mt-5 border border-chart-4/40 bg-chart-4/10 px-4 py-3 text-sm text-chart-4",
                                "{action_message()}"
                            }
                        }

                        div { class: "grid gap-3 p-5 md:grid-cols-3",
                            Metric {
                                label: "Arrangements",
                                value: selected_piece.as_ref().map(|piece| piece.arrangement_count.to_string()).unwrap_or_else(|| "0".to_string()),
                            }
                            Metric {
                                label: "Notes",
                                value: selected_piece.as_ref().map(|piece| piece.note_count.to_string()).unwrap_or_else(|| "0".to_string()),
                            }
                            Metric {
                                label: "Updated",
                                value: selected_piece.as_ref().map(|piece| piece.updated_at.clone()).unwrap_or_else(|| "-".to_string()),
                            }
                        }

                        div { class: "grid gap-5 px-5 pb-5 xl:grid-cols-[minmax(0,1.2fr)_minmax(280px,0.8fr)]",
                            ArrangementPanel { arrangements: arrangements.read().as_ref().cloned() }
                            SessionPanel { sessions: sessions.read().as_ref().cloned() }
                        }

                        div { class: "px-5 pb-5",
                            ScorePreviewPanel { preview: score_preview.read().as_ref().cloned() }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "desktop")]
fn desktop_folder_picker_enabled() -> bool {
    true
}

#[cfg(not(feature = "desktop"))]
fn desktop_folder_picker_enabled() -> bool {
    false
}

#[cfg(feature = "desktop")]
async fn pick_watch_directories() -> Option<Vec<String>> {
    rfd::AsyncFileDialog::new()
        .set_title("Choose MIDI folders")
        .pick_folders()
        .await
        .map(|folders| {
            folders
                .into_iter()
                .map(|folder| folder.path().display().to_string())
                .collect()
        })
}

#[cfg(not(feature = "desktop"))]
async fn pick_watch_directories() -> Option<Vec<String>> {
    None
}

#[component]
fn LibraryPane(
    pieces_result: Option<api::ApiResult<Vec<api::music::MusicPieceDTO>>>,
    selected_piece_id: String,
    on_select: EventHandler<String>,
    on_refresh: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        aside { class: "border border-border bg-card shadow-sm",
            div { class: "flex items-start justify-between gap-4 border-b border-border p-5",
                div { class: "space-y-1",
                    h2 { class: "text-xl font-black", "曲库" }
                    p { class: "text-sm text-muted-foreground", "通过 app port 写入 Toasty + SQLite" }
                }
                button {
                    class: "h-9 border border-border bg-secondary px-3 text-sm font-bold text-secondary-foreground hover:bg-accent",
                    onclick: move |event| on_refresh.call(event),
                    "Refresh"
                }
            }

            match pieces_result {
                Some(Ok(items)) if items.is_empty() => rsx! {
                    div { class: "p-5 text-sm text-muted-foreground", "暂无曲目" }
                },
                Some(Ok(items)) => rsx! {
                    div { class: "grid gap-2 p-3",
                        for piece in items {
                            button {
                                class: if selected_piece_id == piece.id {
                                    "grid gap-1 border border-chart-3 bg-chart-3/10 p-4 text-left"
                                } else {
                                    "grid gap-1 border border-border bg-background p-4 text-left hover:bg-accent"
                                },
                                onclick: {
                                    let id = piece.id.clone();
                                    move |_| on_select.call(id.clone())
                                },
                                span { class: "font-black", "{piece.title}" }
                                span { class: "text-sm text-muted-foreground",
                                    "{piece.arrangement_count} arrangements · {piece.note_count} notes"
                                }
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    div { class: "p-5 text-sm text-destructive", "{err}" }
                },
                None => rsx! {
                    div { class: "p-5 text-sm text-muted-foreground", "Loading local library" }
                },
            }
        }
    }
}

#[component]
fn Metric(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "border border-border bg-background p-4",
            span { class: "text-sm text-muted-foreground", "{label}" }
            strong { class: "mt-2 block truncate text-xl font-black", "{value}" }
        }
    }
}

#[component]
fn ArrangementPanel(arrangements: Option<api::ApiResult<Vec<api::music::ArrangementDTO>>>) -> Element {
    rsx! {
        div { class: "border border-border bg-background p-5",
            h3 { class: "mb-4 text-lg font-black", "编配" }
            match arrangements {
                Some(Ok(items)) if items.is_empty() => rsx! {
                    p { class: "text-sm text-muted-foreground", "当前曲目没有编配" }
                },
                Some(Ok(items)) => rsx! {
                    div { class: "grid gap-3",
                        for item in items {
                            div { class: "flex items-center justify-between gap-4 border border-border bg-card p-4",
                                div { class: "grid gap-1",
                                    strong { "{item.title}" }
                                    span { class: "text-sm text-muted-foreground", "{item.part_count} parts · {item.note_count} notes" }
                                }
                                span { class: "whitespace-nowrap text-sm font-bold text-chart-3",
                                    "{item.tempo_label}"
                                }
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    p { class: "text-sm text-destructive", "{err}" }
                },
                None => rsx! {
                    p { class: "text-sm text-muted-foreground", "Loading arrangements" }
                },
            }
        }
    }
}

#[component]
fn ScorePreviewPanel(preview: Option<api::ApiResult<Option<api::music::ScorePreviewDTO>>>) -> Element {
    rsx! {
        div { class: "border border-border bg-background p-5",
            div { class: "mb-4 flex flex-col gap-1 md:flex-row md:items-end md:justify-between",
                div {
                    h3 { class: "text-lg font-black", "落柱子预览" }
                    p { class: "text-sm text-muted-foreground", "静态时间轴，后续可替换为 JS bridge 动画层" }
                }
            }

            match preview {
                Some(Ok(Some(preview))) => rsx! {
                    FallingNotesPreview { preview }
                },
                Some(Ok(None)) => rsx! {
                    p { class: "text-sm text-muted-foreground", "当前曲目没有可预览的谱面" }
                },
                Some(Err(err)) => rsx! {
                    p { class: "text-sm text-destructive", "{err}" }
                },
                None => rsx! {
                    p { class: "text-sm text-muted-foreground", "Loading score preview" }
                },
            }
        }
    }
}

#[component]
fn FallingNotesPreview(preview: api::music::ScorePreviewDTO) -> Element {
    let low = preview.lowest_pitch.saturating_sub(2).max(21);
    let high = preview.highest_pitch.saturating_add(2).min(108);
    let pitch_span = (high.saturating_sub(low) as f32).max(1.0);
    let total_beats = preview.total_beats.max(1.0);
    let white_keys: Vec<u8> = (low..=high).filter(|pitch| !is_black_key(*pitch)).collect();

    rsx! {
        div { class: "overflow-hidden border border-border bg-card",
            div { class: "flex items-center justify-between border-b border-border px-4 py-3",
                div { class: "grid gap-1",
                    strong { "{preview.title}" }
                    span { class: "text-xs text-muted-foreground",
                        "Range {low}-{high} · {preview.notes.len()} notes · {total_beats:.1} beats"
                    }
                }
            }
            div { class: "relative h-[420px] overflow-hidden bg-background",
                div { class: "absolute left-0 right-0 top-1/2 border-t border-chart-3/50" }
                div { class: "absolute bottom-0 left-0 right-0 h-12 border-t border-border bg-card" }

                for beat in beat_markers(total_beats) {
                    div {
                        class: "absolute left-0 right-0 border-t border-border/70",
                        style: "top: {beat_to_top_percent(beat, total_beats):.3}%;",
                    }
                }

                for note in preview.notes.iter() {
                    div {
                        class: if is_black_key(note.pitch) {
                            "absolute rounded-sm bg-foreground"
                        } else {
                            "absolute rounded-sm border border-chart-3/60 bg-chart-3/80"
                        },
                        title: "{note.part_name} · {note.pitch}",
                        style: note_style(note, low, pitch_span, total_beats),
                    }
                }
            }
            div { class: "relative flex h-16 border-t border-border bg-card px-3 pb-3 pt-2",
                for key in white_keys {
                    div { class: "relative min-w-0 flex-1 border border-border bg-background",
                        span { class: "absolute bottom-1 left-1/2 -translate-x-1/2 text-[10px] text-muted-foreground",
                            "{pitch_label(key)}"
                        }
                        if key < high && is_black_key(key + 1) {
                            div { class: "absolute -right-[18%] top-0 z-10 h-9 w-[36%] bg-foreground" }
                        }
                    }
                }
            }
        }
    }
}

fn note_style(
    note: &api::music::FallingNoteDTO,
    low: u8,
    pitch_span: f32,
    total_beats: f32,
) -> String {
    let left = ((note.pitch.saturating_sub(low) as f32 / pitch_span) * 96.0 + 2.0).clamp(0.0, 97.0);
    let width = (92.0 / pitch_span).clamp(0.45, 5.0);
    let top = beat_to_top_percent(note.start_beats + note.duration_beats, total_beats);
    let height = (note.duration_beats / total_beats * 82.0).clamp(1.0, 40.0);
    format!("left: {left:.3}%; top: {top:.3}%; width: {width:.3}%; height: {height:.3}%;")
}

fn beat_to_top_percent(beat: f32, total_beats: f32) -> f32 {
    86.0 - (beat / total_beats.max(1.0) * 82.0)
}

fn beat_markers(total_beats: f32) -> Vec<f32> {
    let max = total_beats.ceil() as usize;
    (0..=max.min(32)).step_by(4).map(|beat| beat as f32).collect()
}

fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

fn pitch_label(pitch: u8) -> String {
    let name = match pitch % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        _ => "B",
    };
    format!("{name}{}", pitch / 12)
}

#[component]
fn SessionPanel(sessions: Option<api::ApiResult<Vec<api::learning::PracticeSessionDTO>>>) -> Element {
    rsx! {
        div { class: "border border-border bg-background p-5",
            h3 { class: "mb-4 text-lg font-black", "练习记录" }
            match sessions {
                Some(Ok(items)) if items.is_empty() => rsx! {
                    p { class: "text-sm text-muted-foreground", "还没有练习会话" }
                },
                Some(Ok(items)) => rsx! {
                    div { class: "grid gap-3",
                        for item in items {
                            div { class: "grid gap-1 border border-border bg-card p-4",
                                strong { "{item.status}" }
                                span { class: "truncate text-sm text-muted-foreground", "{item.id}" }
                                small { class: "text-xs text-muted-foreground", "{item.attempt_count} attempts · speed {item.target_speed}" }
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    p { class: "text-sm text-destructive", "{err}" }
                },
                None => rsx! {
                    p { class: "text-sm text-muted-foreground", "Loading sessions" }
                },
            }
        }
    }
}
