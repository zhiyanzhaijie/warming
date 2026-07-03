use dioxus::prelude::*;

#[derive(serde::Deserialize)]
struct PlaybackTick {
    beat: f32,
    ended: bool,
}

#[component]
pub fn PracticeView(piece_id: String, arrangement_id: String) -> Element {
    let nav = navigator();
    let mut playing = use_signal(|| false);
    let mut speed = use_signal(|| 1.0_f32);
    let mut progress_beats = use_signal(|| 0.0_f32);
    let preview = use_resource({
        let piece_id = piece_id.clone();
        let arrangement_id = arrangement_id.clone();
        move || {
            let piece_id = piece_id.clone();
            let arrangement_id = arrangement_id.clone();
            async move { api::music::get_score_preview(&piece_id, &arrangement_id).await }
        }
    });

    use_future(move || async move {
        let mut eval = document::eval(
            r#"
            if (window.__warmingPracticeTickHandler) {
                window.removeEventListener("warming-practice-tick", window.__warmingPracticeTickHandler);
            }
            window.__warmingPracticeTickHandler = (event) => {
                dioxus.send(event.detail || { beat: 0, ended: false });
            };
            window.addEventListener("warming-practice-tick", window.__warmingPracticeTickHandler);
            await new Promise(() => {});
            "#,
        );

        while let Ok(tick) = eval.recv::<PlaybackTick>().await {
            progress_beats.set(tick.beat.max(0.0));
            if tick.ended {
                playing.set(false);
            }
        }
    });

    rsx! {
        main { class: "min-h-screen bg-background text-foreground",
            div { class: "mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-5 px-6 py-6 md:px-8",
                header { class: "flex flex-col gap-4 border-b border-border pb-5 md:flex-row md:items-center md:justify-between",
                    div { class: "space-y-2",
                        p { class: "text-xs font-bold uppercase text-chart-3", "Practice" }
                        h1 { class: "text-3xl font-black leading-none md:text-5xl", "练习模式" }
                    }
                    div { class: "flex flex-wrap gap-2",
                        button {
                            class: "h-10 border border-border bg-secondary px-4 text-sm font-bold text-secondary-foreground hover:bg-accent",
                            onclick: move |_| {
                                nav.push("/");
                            },
                            "Back"
                        }
                        button {
                            class: "h-10 border border-primary bg-primary px-4 text-sm font-bold text-primary-foreground",
                            onclick: move |_| {
                                if playing() {
                                    stop_audio();
                                    playing.set(false);
                                } else if let Some(Ok(Some(preview))) = preview.read().as_ref() {
                                    start_audio(preview, progress_beats(), speed());
                                    playing.set(true);
                                }
                            },
                            if playing() { "Pause" } else { "Play" }
                        }
                    }
                }

                section { class: "grid gap-4 border border-border bg-card p-4 shadow-sm md:grid-cols-[1fr_auto]",
                    div { class: "flex flex-wrap items-center gap-2",
                        span { class: "text-sm font-bold text-muted-foreground", "Speed" }
                        for value in [0.5_f32, 0.75, 1.0, 1.25, 1.5] {
                            button {
                                class: if (speed() - value).abs() < 0.01 {
                                    "h-9 border border-chart-3 bg-chart-3/10 px-3 text-sm font-bold text-chart-3"
                                } else {
                                    "h-9 border border-border bg-background px-3 text-sm font-bold text-foreground hover:bg-accent"
                                },
                                onclick: move |_| {
                                    speed.set(value);
                                    if playing() {
                                        stop_audio();
                                        if let Some(Ok(Some(preview))) = preview.read().as_ref() {
                                            start_audio(preview, progress_beats(), value);
                                        }
                                    }
                                },
                                "{value:.2}x"
                            }
                        }
                    }
                    div { class: "flex items-center gap-3 text-sm text-muted-foreground",
                        span { class: if playing() { "h-2.5 w-2.5 rounded-full bg-chart-4" } else { "h-2.5 w-2.5 rounded-full bg-muted-foreground" } }
                        span { if playing() { "Playing" } else { "Paused" } }
                    }
                }

                match preview.read().as_ref() {
                    Some(Ok(Some(preview))) => rsx! {
                        PracticeRoll {
                            preview: preview.clone(),
                            playing: playing(),
                            speed: speed(),
                            progress_beats: progress_beats(),
                            on_seek: move |beat| {
                                progress_beats.set(beat);
                                stop_audio();
                                playing.set(false);
                            },
                        }
                    },
                    Some(Ok(None)) => rsx! {
                        div { class: "border border-border bg-card p-6 text-sm text-muted-foreground", "没有可练习的谱面" }
                    },
                    Some(Err(err)) => rsx! {
                        div { class: "border border-border bg-card p-6 text-sm text-destructive", "{err}" }
                    },
                    None => rsx! {
                        div { class: "border border-border bg-card p-6 text-sm text-muted-foreground", "Loading practice score" }
                    },
                }
            }
        }
    }
}

#[component]
fn PracticeRoll(
    preview: api::music::ScorePreviewDTO,
    playing: bool,
    speed: f32,
    progress_beats: f32,
    on_seek: EventHandler<f32>,
) -> Element {
    let low = preview.lowest_pitch.saturating_sub(2).max(21);
    let high = preview.highest_pitch.saturating_add(2).min(108);
    let pitch_span = (high.saturating_sub(low) as f32).max(1.0);
    let total_beats = preview.total_beats.max(1.0);
    let progress_percent = (progress_beats / total_beats * 100.0).clamp(0.0, 100.0);
    let white_keys: Vec<u8> = (low..=high).filter(|pitch| !is_black_key(*pitch)).collect();
    let active_pitches: Vec<u8> = preview
        .notes
        .iter()
        .filter(|note| note.start_beats <= progress_beats && progress_beats <= note.start_beats + note.duration_beats)
        .map(|note| note.pitch)
        .collect();

    rsx! {
        section { class: "overflow-hidden border border-border bg-card shadow-sm",
            div { class: "flex flex-col gap-2 border-b border-border px-5 py-4 md:flex-row md:items-center md:justify-between",
                div {
                    h2 { class: "text-xl font-black", "{preview.title}" }
                    p { class: "text-sm text-muted-foreground",
                        "Range {low}-{high} · {preview.notes.len()} notes · {total_beats:.1} beats · {preview.bpm:.0} BPM · speed {speed:.2}x"
                    }
                }
                div { class: "text-sm font-bold text-chart-3",
                    if playing { "落键预览运行中" } else { "已暂停" }
                }
            }
            div { class: "grid gap-2 border-b border-border px-5 py-4",
                div { class: "flex items-center justify-between text-xs font-bold text-muted-foreground",
                    span { "Progress" }
                    span { "{progress_beats:.1} / {total_beats:.1} beats" }
                }
                input {
                    class: "w-full accent-[var(--chart-3)]",
                    r#type: "range",
                    min: "0",
                    max: "1000",
                    value: "{(progress_percent * 10.0) as i32}",
                    oninput: move |event| {
                        if let Ok(raw) = event.value().parse::<f32>() {
                            on_seek.call((raw / 1000.0) * total_beats);
                        }
                    },
                }
            }

            div { class: "relative h-[62vh] min-h-[520px] overflow-hidden bg-background",
                div { class: "absolute bottom-20 left-0 right-0 z-20 border-t-2 border-chart-3" }
                div { class: "absolute bottom-0 left-0 right-0 z-10 h-20 border-t border-border bg-card" }

                for beat in beat_markers(progress_beats, total_beats) {
                    div {
                        class: "absolute left-0 right-0 border-t border-border/70",
                        style: "top: {beat_to_top_percent(beat, progress_beats):.3}%;",
                    }
                }

                for note in preview.notes.iter().filter(|note| is_visible_note(note, progress_beats)) {
                    div {
                        class: if is_black_key(note.pitch) {
                            "absolute rounded-sm bg-foreground shadow-sm"
                        } else {
                            "absolute rounded-sm border border-chart-3/70 bg-chart-3/80 shadow-sm"
                        },
                        title: "{note.part_name} · {note.pitch}",
                        style: note_style(note, low, pitch_span, progress_beats),
                    }
                }
            }

            div { class: "relative flex h-20 border-t border-border bg-card px-4 pb-4 pt-2",
                for key in white_keys {
                    div { class: if active_pitches.contains(&key) {
                            "relative min-w-0 flex-1 border border-chart-3 bg-chart-3/30"
                        } else {
                            "relative min-w-0 flex-1 border border-border bg-background"
                        },
                        span { class: "absolute bottom-1 left-1/2 -translate-x-1/2 text-[10px] text-muted-foreground",
                            "{pitch_label(key)}"
                        }
                        if key < high && is_black_key(key + 1) {
                            div { class: "absolute -right-[18%] top-0 z-10 h-11 w-[36%] bg-foreground" }
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
    progress_beats: f32,
) -> String {
    const VISIBLE_BEATS: f32 = 16.0;
    let left = ((note.pitch.saturating_sub(low) as f32 / pitch_span) * 96.0 + 2.0).clamp(0.0, 97.0);
    let width = (92.0 / pitch_span).clamp(0.45, 5.0);
    let top = beat_to_top_percent(note.start_beats, progress_beats);
    let height = (note.duration_beats / VISIBLE_BEATS * 82.0).clamp(1.0, 44.0);
    format!("left: {left:.3}%; top: {top:.3}%; width: {width:.3}%; height: {height:.3}%;")
}

fn beat_to_top_percent(beat: f32, progress_beats: f32) -> f32 {
    const VISIBLE_BEATS: f32 = 16.0;
    86.0 - ((beat - progress_beats) / VISIBLE_BEATS * 82.0)
}

fn beat_markers(progress_beats: f32, total_beats: f32) -> Vec<f32> {
    let start = (progress_beats.floor() as i32 - 4).max(0);
    let end = (progress_beats.ceil() as i32 + 18).min(total_beats.ceil() as i32);
    (start..=end).filter(|beat| beat % 4 == 0).map(|beat| beat as f32).collect()
}

fn is_visible_note(note: &&api::music::FallingNoteDTO, progress_beats: f32) -> bool {
    note.start_beats + note.duration_beats >= progress_beats - 2.0
        && note.start_beats <= progress_beats + 16.0
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

fn start_audio(preview: &api::music::ScorePreviewDTO, start_beat: f32, speed: f32) {
    let Ok(notes_json) = serde_json::to_string(&preview.notes) else {
        return;
    };
    let script = format!(
        r#"
        (() => {{
            const notes = {notes_json};
            const bpm = {bpm};
            const totalBeats = {total_beats};
            const startBeat = {start_beat};
            const speed = {speed};
            const secondsPerBeat = 60 / bpm / speed;
            const nowMs = () => performance.now();

            window.__warmingAudioStop?.();
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            if (!AudioContext) return;

            const ctx = window.__warmingAudioContext || new AudioContext();
            window.__warmingAudioContext = ctx;
            ctx.resume?.();
            const gain = ctx.createGain();
            gain.gain.value = 0.08;
            gain.connect(ctx.destination);

            const startedAtMs = nowMs();
            const oscillators = [];
            const midiToHz = (pitch) => 440 * Math.pow(2, (pitch - 69) / 12);

            for (const note of notes) {{
                const noteEnd = note.start_beats + note.duration_beats;
                if (noteEnd < startBeat) continue;
                const startDelay = Math.max(0, (note.start_beats - startBeat) * secondsPerBeat);
                const duration = Math.max(0.04, note.duration_beats * secondsPerBeat);
                const osc = ctx.createOscillator();
                const env = ctx.createGain();
                osc.type = "triangle";
                osc.frequency.value = midiToHz(note.pitch);
                env.gain.setValueAtTime(0.0001, ctx.currentTime + startDelay);
                env.gain.exponentialRampToValueAtTime(0.9, ctx.currentTime + startDelay + 0.015);
                env.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + startDelay + duration);
                osc.connect(env);
                env.connect(gain);
                osc.start(ctx.currentTime + startDelay);
                osc.stop(ctx.currentTime + startDelay + duration + 0.03);
                oscillators.push(osc);
            }}

            const sendTick = () => {{
                const beat = Math.min(totalBeats, startBeat + ((nowMs() - startedAtMs) / 1000) / secondsPerBeat);
                window.dispatchEvent(new CustomEvent("warming-practice-tick", {{ detail: {{ beat, ended: beat >= totalBeats }} }}));
                if (beat >= totalBeats) window.__warmingAudioStop?.();
            }};

            const timer = setInterval(sendTick, 33);
            sendTick();
            window.__warmingAudioStop = () => {{
                clearInterval(timer);
                for (const osc of oscillators) {{
                    try {{ osc.stop(); }} catch (_) {{}}
                }}
            }};
        }})();
        "#,
        notes_json = notes_json,
        bpm = preview.bpm.max(1.0),
        total_beats = preview.total_beats.max(1.0),
        start_beat = start_beat.max(0.0),
        speed = speed.max(0.1),
    );
    document::eval(&script);
}

fn stop_audio() {
    document::eval("window.__warmingAudioStop?.();");
}
