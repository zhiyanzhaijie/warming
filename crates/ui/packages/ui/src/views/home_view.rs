use dioxus::prelude::*;

#[component]
pub fn HomeView() -> Element {
    let mut selected_piece_id = use_signal(String::new);
    let mut action_message = use_signal(String::new);

    let mut pieces = use_resource(move || async move {
        let _ = api::music::ensure_demo_piece().await;
        api::music::list_pieces().await
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

                section { class: "grid flex-1 gap-5 lg:grid-cols-[360px_minmax(0,1fr)]",
                    LibraryPane {
                        pieces_result: piece_result.cloned(),
                        selected_piece_id: selected_piece_id(),
                        on_select: move |id: String| {
                            selected_piece_id.set(id);
                            arrangements.restart();
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
                                                match api::learning::start_demo_session(piece.id, arrangement.id).await {
                                                    Ok(session) => action_message.set(format!("Started {}", session.id)),
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
                    }
                }
            }
        }
    }
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
