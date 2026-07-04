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
        main { class: "h-screen w-screen bg-background text-foreground font-sans flex flex-col overflow-hidden select-none",
            // 顶栏 - 固定高度
            header { class: "flex-none h-16 flex items-center justify-between border-b border-border px-6 md:px-8",
                div { class: "space-y-0.5",
                    p { class: "text-[10px] font-bold uppercase tracking-[1.5px] text-primary", "Local training workspace" }
                    h1 { class: "text-xl font-bold tracking-tight", "Warming" }
                }
                div { class: "flex items-center gap-3",
                    button {
                        class: "p-2 rounded-full bg-secondary hover:bg-accent text-white transition-all cursor-pointer hover:scale-105",
                        title: "设置 MIDI 目录",
                        onclick: move |_| {
                            nav.push("/settings");
                        },
                        svg {
                            class: "h-4.5 w-4.5",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                            }
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                            }
                        }
                    }
                    
                    div { class: "flex items-center gap-2 bg-card px-3 py-1.5 rounded-full text-[11px] font-bold text-muted-foreground shadow-sm",
                        span { class: "h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_8px_rgba(30,215,96,0.7)]" }
                        span { "SQLite local" }
                    }
                }
            }

            // 主体内容区域 - 高度填充，不产生 y 轴溢出
            div { class: "flex-1 min-h-0 flex gap-4 p-4 md:p-6",
                // 左侧曲库（固定宽度，独立滚动）
                div { class: "w-[320px] flex-none flex flex-col bg-background rounded-lg border border-border overflow-hidden",
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
                }

                // 右侧大面板（自动撑满，独立滚动）
                div { class: "flex-1 min-w-0 bg-card rounded-lg border border-border/50 shadow-medium flex flex-col overflow-hidden",
                    // 详情面板头部 - 固定
                    div { class: "flex-none flex flex-col gap-4 border-b border-border p-5 md:flex-row md:items-center md:justify-between bg-gradient-to-b from-[#1c1c1c] to-card",
                        div { class: "space-y-1 min-w-0",
                            p { class: "text-[10px] font-bold uppercase tracking-[1.5px] text-primary", "Selected score" }
                            h2 { class: "text-xl md:text-2xl font-bold tracking-tight text-white truncate",
                                {selected_piece.as_ref().map(|piece| piece.title.as_str()).unwrap_or("No score selected")}
                            }
                        }
                        button {
                            class: "flex-none h-11 bg-primary hover:bg-spotify-green-hover hover:scale-104 transition-all duration-150 active:scale-95 px-6 rounded-full text-xs font-bold text-primary-foreground uppercase tracking-[1.5px] disabled:cursor-not-allowed disabled:opacity-40 shadow-lg shadow-primary/20 cursor-pointer",
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

                    // 右侧下半部可滚动的区域
                    div { class: "flex-1 min-h-0 overflow-y-auto p-5 space-y-5 scrollbar-thin",
                        if !action_message().is_empty() {
                            div { class: "bg-chart-4/10 border border-chart-4/30 rounded px-4 py-2 text-xs text-chart-4 font-normal",
                                "{action_message()}"
                            }
                        }

                        // Metrics 统计
                        div { class: "grid gap-3 grid-cols-3",
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

                        // 编配与历史记录
                        div { class: "grid gap-4 md:grid-cols-2",
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
        // 曲库控制头部
        div { class: "flex-none flex items-center justify-between gap-2 border-b border-border p-4 bg-[#181818]",
            div { class: "space-y-0.5",
                h2 { class: "text-sm font-bold text-white", "曲库" }
                p { class: "text-[10px] font-normal text-muted-foreground", "SQLite 驱动" }
            }
            button {
                class: "h-8 border border-silver/30 hover:border-white transition-all bg-transparent hover:scale-104 px-3 rounded-full text-[10px] font-bold text-white uppercase tracking-[1px] cursor-pointer",
                onclick: move |event| on_refresh.call(event),
                "Refresh"
            }
        }

        // 独立 Y 轴滚动列表
        div { class: "flex-1 min-h-0 overflow-y-auto p-2 space-y-1.5 scrollbar-thin bg-background",
            match pieces_result {
                Some(Ok(items)) if items.is_empty() => rsx! {
                    div { class: "p-4 text-xs font-normal text-muted-foreground text-center", "暂无曲目" }
                },
                Some(Ok(items)) => rsx! {
                    for piece in items {
                        button {
                            class: if selected_piece_id == piece.id {
                                "w-full grid gap-1 bg-accent p-3 text-left rounded-md border-l-4 border-primary cursor-pointer transition-all"
                            } else {
                                "w-full grid gap-1 bg-transparent hover:bg-card p-3 text-left rounded-md border-l-4 border-transparent cursor-pointer transition-all"
                            },
                            onclick: {
                                let id = piece.id.clone();
                                move |_| on_select.call(id.clone())
                            },
                            span { class: "font-bold text-white text-xs md:text-sm truncate", "{piece.title}" }
                            span { class: "text-[10px] font-normal text-muted-foreground",
                                "{piece.arrangement_count} arrangements · {piece.note_count} notes"
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    div { class: "p-4 text-xs font-bold text-destructive text-center", "{err}" }
                },
                None => rsx! {
                    div { class: "p-4 text-xs font-normal text-muted-foreground text-center", "Loading local library" }
                },
            }
        }
    }
}

#[component]
fn Metric(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "bg-secondary rounded-lg p-3 border border-border/40 shadow-sm",
            span { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-[1px]", "{label}" }
            strong { class: "mt-1 block truncate text-base font-bold text-white", "{value}" }
        }
    }
}

#[component]
fn ArrangementPanel(arrangements: Option<api::ApiResult<Vec<api::music::ArrangementDTO>>>) -> Element {
    rsx! {
        div { class: "bg-secondary rounded-lg p-4 border border-border/40 flex flex-col h-fit",
            h3 { class: "mb-3 text-[11px] font-bold text-white uppercase tracking-[1.5px]", "编配" }
            match arrangements {
                Some(Ok(items)) if items.is_empty() => rsx! {
                    p { class: "text-xs font-normal text-muted-foreground", "当前曲目没有编配" }
                },
                Some(Ok(items)) => rsx! {
                    div { class: "grid gap-2.5",
                        for item in items {
                            div { class: "flex items-center justify-between gap-3 bg-card rounded-md p-3.5 shadow-sm border border-border/10",
                                div { class: "grid gap-0.5 min-w-0",
                                    strong { class: "font-bold text-white text-xs truncate", "{item.title}" }
                                    span { class: "text-[10px] font-normal text-muted-foreground", "{item.part_count} parts · {item.note_count} notes" }
                                }
                                span { class: "flex-none text-[9px] font-bold text-chart-3 uppercase tracking-[0.5px] bg-chart-3/10 px-2 py-0.5 rounded",
                                    "{item.tempo_label}"
                                }
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    p { class: "text-xs font-bold text-destructive", "{err}" }
                },
                None => rsx! {
                    p { class: "text-xs font-normal text-muted-foreground", "Loading arrangements" }
                },
            }
        }
    }
}

#[component]
fn SessionPanel(sessions: Option<api::ApiResult<Vec<api::learning::PracticeSessionDTO>>>) -> Element {
    rsx! {
        div { class: "bg-secondary rounded-lg p-4 border border-border/40 flex flex-col h-fit",
            h3 { class: "mb-3 text-[11px] font-bold text-white uppercase tracking-[1.5px]", "练习记录" }
            match sessions {
                Some(Ok(items)) if items.is_empty() => rsx! {
                    p { class: "text-xs font-normal text-muted-foreground", "还没有练习会话" }
                },
                Some(Ok(items)) => rsx! {
                    div { class: "grid gap-2.5",
                        for item in items {
                            div { class: "grid gap-0.5 bg-card rounded-md p-3 shadow-sm border border-border/25 min-w-0",
                                strong { class: "font-bold text-white text-xs truncate", "{item.status}" }
                                span { class: "truncate text-[10px] font-normal text-muted-foreground font-mono", "{item.id}" }
                                small { class: "text-[10px] font-normal text-muted-foreground", "{item.attempt_count} attempts · speed {item.target_speed}" }
                            }
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    p { class: "text-xs font-bold text-destructive", "{err}" }
                },
                None => rsx! {
                    p { class: "text-xs font-normal text-muted-foreground", "Loading sessions" }
                },
            }
        }
    }
}
