use dioxus::prelude::*;

#[component]
pub fn SettingsView() -> Element {
    let nav = navigator();
    let mut action_message = use_signal(String::new);

    let mut watched_directories = use_resource(move || async move {
        api::watch::list_watch_directories().await
    });

    rsx! {
        main { class: "h-screen w-screen bg-background text-foreground font-sans flex flex-col overflow-hidden select-none",
            // 顶栏 - 固定高度
            header { class: "flex-none h-16 flex items-center justify-between border-b border-border px-6 md:px-8",
                div { class: "space-y-0.5",
                    p { class: "text-[10px] font-bold uppercase tracking-[1.5px] text-muted-foreground", "Workspace Settings" }
                    h1 { class: "text-lg font-bold tracking-tight", "设置" }
                }
                button {
                    class: "h-9 border border-[#7c7c7c] hover:border-white transition-all bg-transparent px-4 rounded-full text-xs font-bold text-white uppercase tracking-[1px] cursor-pointer hover:scale-104",
                    onclick: move |_| {
                        nav.push("/");
                    },
                    "返回主页"
                }
            }

            // 主要内容区域 - 撑满，禁止全局滚动，卡片内部滚动
            div { class: "flex-1 min-h-0 p-4 md:p-6 flex justify-center bg-background",
                div { class: "w-full max-w-3xl bg-card rounded-lg border border-border/50 shadow-heavy flex flex-col overflow-hidden",
                    // 卡片内固定头部
                    div { class: "flex-none p-5 border-b border-border bg-gradient-to-b from-[#1c1c1c] to-card",
                        h2 { class: "text-base font-bold text-white mb-1", "MIDI 监听目录" }
                        p { class: "text-xs font-normal text-muted-foreground", "Warming 将在这些本地文件夹中自动发现有效的 .mid/.midi 文件并同步到曲库。" }
                    }

                    // 卡片内部核心控制区与列表区 - 独立滚动
                    div { class: "flex-1 min-h-0 overflow-y-auto p-5 space-y-5 scrollbar-thin",
                        div { class: "flex flex-col gap-3 sm:flex-row",
                            button {
                                class: "h-10 bg-white hover:scale-104 transition-transform active:scale-95 duration-150 px-6 rounded-full text-xs font-bold text-background uppercase tracking-[1.5px] disabled:cursor-not-allowed disabled:opacity-40 cursor-pointer",
                                disabled: !desktop_folder_picker_enabled(),
                                onclick: move |_| {
                                    spawn(async move {
                                        match pick_watch_directories().await {
                                            Some(directories) if !directories.is_empty() => {
                                                match api::watch::add_watch_directories(directories).await {
                                                    Ok(report) => {
                                                        action_message.set(format!(
                                                            "成功添加！正在监听 {} 个目录，扫描到 {} 个 MIDI，登记 {} 个曲目。",
                                                            report.watched_directories.len(),
                                                            report.discovered_files,
                                                            report.registered_files
                                                        ));
                                                        watched_directories.restart();
                                                    }
                                                    Err(err) => action_message.set(err.to_string()),
                                                }
                                            }
                                            Some(_) => action_message.set("未选择任何目录".to_string()),
                                            None => action_message.set("此平台暂不支持目录选择".to_string()),
                                        }
                                    });
                                },
                                "添加监听目录"
                            }
                            button {
                                class: "h-10 border border-[#7c7c7c] hover:border-white hover:scale-104 transition-transform active:scale-95 duration-150 bg-transparent px-6 rounded-full text-xs font-bold text-white uppercase tracking-[1.5px] cursor-pointer",
                                onclick: move |_| {
                                    spawn(async move {
                                        match api::watch::refresh_watched_directories().await {
                                            Ok(report) => {
                                                action_message.set(format!(
                                                    "已成功刷新！当前正在监听 {} 个目录。",
                                                    report.watched_directories.len()
                                                ));
                                                watched_directories.restart();
                                            }
                                            Err(err) => action_message.set(err.to_string()),
                                        }
                                    });
                                },
                                "刷新扫描"
                            }
                        }

                        if !action_message().is_empty() {
                            div { class: "bg-primary/10 border border-primary/30 rounded p-4 text-xs font-normal text-primary",
                                "{action_message()}"
                            }
                        }

                        // 独立滚动的监听文件夹列表
                        div { class: "space-y-3",
                            h3 { class: "text-xs font-bold uppercase tracking-[1px] text-muted-foreground", "当前监听中的文件夹" }
                            
                            match watched_directories.read().as_ref() {
                                Some(Ok(items)) if items.is_empty() => rsx! {
                                    div { class: "p-4 bg-background rounded border border-border text-xs font-normal text-muted-foreground text-center", 
                                        "尚未配置任何监听目录，请点击上方按钮添加。" 
                                    }
                                },
                                Some(Ok(items)) => rsx! {
                                    div { class: "divide-y divide-border bg-background rounded-lg border border-border overflow-hidden",
                                        for directory in items {
                                            div { class: "p-4 flex items-center justify-between text-xs font-normal text-white hover:bg-mid-dark transition-colors min-w-0 gap-4",
                                                span { class: "truncate font-mono text-xs", "{directory}" }
                                                span { class: "flex-none text-[10px] bg-primary/10 text-primary px-2 py-0.5 rounded-full font-bold", "ACTIVE" }
                                            }
                                        }
                                    }
                                },
                                Some(Err(err)) => rsx! {
                                    div { class: "p-4 bg-destructive/10 rounded border border-destructive/30 text-xs font-bold text-destructive", 
                                        "{err}" 
                                    }
                                },
                                None => rsx! {
                                    div { class: "p-4 bg-background rounded border border-border text-xs font-normal text-muted-foreground text-center", 
                                        "正在加载监听文件夹列表..." 
                                    }
                                },
                            }
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
