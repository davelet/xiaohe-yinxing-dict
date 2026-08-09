use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;

use crate::update::{UpdateInfo, UpdateProgress, UpdateState};

/// 自动更新相关的全部 UI 状态与渲染逻辑（从 DictApp 抽取）。
///
/// 内聚了更新检查轮询、更新弹窗、跨视图进度 toast、下载线程，
/// 删除原 DictApp 上的两个死字段（update_toast_dismissed / update_toast_timer）。
pub struct UpdateUiState {
    pub info: Arc<Mutex<Option<UpdateInfo>>>,
    pub show_dialog: bool,
    pub info_for_dialog: Option<UpdateInfo>,
    pub state: Arc<Mutex<UpdateState>>,
    pub progress: Arc<Mutex<UpdateProgress>>,
    pub started_at: Option<Instant>,
    pub retry_info: Option<UpdateInfo>,
    pub cancelled: Arc<AtomicBool>,
    pub toast_shown_done_or_failed: bool,
    pub toast_background: bool,
    pub toast_background_auto: bool,
    pub show_md_preview: bool,
}

impl Default for UpdateUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateUiState {
    pub fn new() -> Self {
        Self {
            info: Arc::new(Mutex::new(None)),
            show_dialog: false,
            info_for_dialog: None,
            state: Arc::new(Mutex::new(UpdateState::Idle)),
            progress: Arc::new(Mutex::new(UpdateProgress {
                message: String::new(),
                bytes_downloaded: 0,
                bytes_total: 0,
                sha256_ok: None,
            })),
            started_at: None,
            retry_info: None,
            cancelled: Arc::new(AtomicBool::new(false)),
            toast_shown_done_or_failed: false,
            toast_background: false,
            toast_background_auto: false,
            show_md_preview: true,
        }
    }

    /// 轮询后台线程写入的更新信息：发现新版本则弹出更新弹窗并自动后台下载。
    pub fn poll_update_info(&mut self, ctx: &egui::Context) {
        if self.show_dialog {
            return;
        }
        let info_to_show = self.info.lock().ok().and_then(|guard| guard.clone());
        if let Some(info) = info_to_show {
            let is_idle = {
                let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                *state == UpdateState::Idle
            };
            if is_idle {
                self.show_dialog = true;
                self.info_for_dialog = Some(info.clone());
                // 发现新版本后立即后台下载+安装，toast 先隐藏
                self.toast_background = true;
                self.toast_background_auto = true;
                self.start_update_thread(info, ctx.clone());
            }
        }
    }

    /// 渲染"发现新版本"更新弹窗。
    pub fn render_dialog(&mut self, ctx: &egui::Context) {
        let mut close_dialog = false;
        if self.show_dialog
            && let Some(info) = &self.info_for_dialog
        {
            let info_clone = info.clone();
            let update_state = self.state.clone();
            let update_state_clone = update_state.clone();
            egui::Window::new("发现新版本")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let mut md_cache = egui_commonmark::CommonMarkCache::default();
                    ui.label(format!("发现新版本: v{}", info_clone.latest_version));
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("更新内容:");
                        ui.add_space(4.0);
                        let md = self.show_md_preview;
                        if ui.add(egui::Button::new("纯文本").selected(!md)).clicked() {
                            self.show_md_preview = false;
                        }
                        if ui.add(egui::Button::new("Markdown").selected(md)).clicked() {
                            self.show_md_preview = true;
                        }
                    });
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            if self.show_md_preview {
                                egui_commonmark::CommonMarkViewer::new().show(
                                    ui,
                                    &mut md_cache,
                                    &info_clone.release_notes,
                                );
                            } else {
                                ui.label(&info_clone.release_notes);
                            }
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("立即更新").clicked() {
                            self.toast_background_auto = false;
                            let current_state = {
                                let guard =
                                    update_state_clone.lock().unwrap_or_else(|e| e.into_inner());
                                guard.clone()
                            };
                            self.toast_background = false;
                            match current_state {
                                UpdateState::Failed(_) | UpdateState::Idle => {
                                    let ctx = ui.ctx().clone();
                                    self.retry_info = Some(info_clone.clone());
                                    self.start_update_thread(info_clone.clone(), ctx);
                                }
                                _ => {}
                            }
                            close_dialog = true;
                        }
                        if ui.button("稍后再说").clicked() {
                            self.cancelled.store(true, Ordering::Relaxed);
                            *self.state.lock().unwrap_or_else(|e| e.into_inner()) =
                                UpdateState::Idle;
                            close_dialog = true;
                        }
                    });
                });
        }
        if close_dialog {
            self.show_dialog = false;
            self.info_for_dialog = None;
            if let Ok(mut guard) = self.info.lock() {
                *guard = None;
            }
        }
    }

    /// 渲染跨视图更新进度 toast（从 update_progress 读取详情）。
    pub fn render_toast(&mut self, ui: &mut egui::Ui) {
        // 记录下载开始时间（用于计算已用时间）
        if let Ok(guard) = self.state.lock() {
            if *guard == UpdateState::Downloading || *guard == UpdateState::Installing {
                if self.started_at.is_none() {
                    self.started_at = Some(Instant::now());
                }
            } else {
                self.started_at = None;
            }
        }

        // 从 update_state 同步 toast 开关
        let mut in_progress = false;
        let mut is_done = false;
        let mut is_failed = false;
        if let Ok(guard) = self.state.lock() {
            match &*guard {
                UpdateState::Downloading | UpdateState::Installing => {
                    in_progress = true;
                }
                UpdateState::Done(_) | UpdateState::Failed(_) => {
                    if !self.toast_shown_done_or_failed {
                        self.toast_shown_done_or_failed = true;
                    }
                    if matches!(&*guard, UpdateState::Done(_)) {
                        is_done = true;
                    } else {
                        is_failed = true;
                    }
                }
                UpdateState::Idle => {}
            }
        }

        // 后台静默下载
        let show_toast = if self.toast_background_auto {
            // 弹窗首次自动下载：彻底静默，不显示任何 toast
            false
        } else if self.toast_background {
            // 用户主动转入后台：隐藏下载中，完成/失败时显示
            if in_progress {
                false
            } else {
                if is_done || is_failed {
                    self.toast_background = false;
                }
                is_done || is_failed
            }
        } else {
            in_progress || is_done || is_failed
        };

        // 渲染富文本 toast
        if show_toast {
            // 从 progress / state 读取详细信息
            let (prog_msg, prog_down, prog_total, sha256_ok) = if let Ok(p) = self.progress.lock() {
                (
                    p.message.clone(),
                    p.bytes_downloaded,
                    p.bytes_total,
                    p.sha256_ok,
                )
            } else {
                (String::new(), 0, 0, None)
            };

            let (state_label, state_color) = if in_progress {
                if let Ok(guard) = self.state.lock() {
                    match &*guard {
                        UpdateState::Downloading => ("下载中", egui::Color32::WHITE),
                        UpdateState::Installing => ("安装中", egui::Color32::WHITE),
                        _ => ("进行中", egui::Color32::WHITE),
                    }
                } else {
                    ("进行中", egui::Color32::WHITE)
                }
            } else if is_done {
                ("完成", egui::Color32::from_rgb(100, 220, 100))
            } else {
                ("失败", egui::Color32::from_rgb(255, 80, 80))
            };

            // 计算已用时间
            let elapsed_str = self.started_at.map(|start| {
                let secs = start.elapsed().as_secs_f64();
                if secs < 60.0 {
                    format!("{:.0}秒", secs)
                } else if secs < 3600.0 {
                    format!("{:.0}分{:.0}秒", secs / 60.0, secs % 60.0)
                } else {
                    format!("{:.1}小时", secs / 3600.0)
                }
            });

            // 进度百分比
            let progress_ratio = if prog_total > 0 {
                (prog_down as f64 / prog_total as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let bg = if in_progress {
                egui::Color32::from_rgba_premultiplied(30, 30, 40, 230)
            } else {
                egui::Color32::from_rgba_premultiplied(40, 40, 50, 220)
            };
            let text_color = egui::Color32::from_rgb(220, 220, 220);

            // 完成/失败时读取 exe_path 和错误详情
            let (done_exe_path, fail_error) = if !in_progress {
                if let Ok(guard) = self.state.lock() {
                    match &*guard {
                        UpdateState::Done(path) => (Some(path.clone()), None),
                        UpdateState::Failed(e) => (None, Some(e.clone())),
                        _ => (None, None),
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            let retry_info = self.retry_info.clone();

            egui::Area::new("update_progress_toast".into())
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -36.0])
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    let frame = egui::Frame::NONE
                        .fill(bg)
                        .corner_radius(8.0)
                        .inner_margin(egui::Margin::symmetric(14, 10));
                    frame.show(ui, |ui| {
                        ui.set_max_width(420.0);

                        // 第一行：状态标签 + 已用时间
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(state_label)
                                    .color(state_color)
                                    .size(12.0)
                                    .strong(),
                            );
                            if let Some(ref elap) = elapsed_str {
                                ui.label(
                                    egui::RichText::new(format!("  ⏱ {}", elap))
                                        .color(egui::Color32::from_rgb(160, 180, 200))
                                        .size(11.0),
                                );
                            }
                        });

                        ui.add_space(4.0);

                        // 第二行：下载进度 + 大小（仅在下载中显示）
                        if in_progress && prog_total > 0 {
                            // 进度条
                            let bar_width = 390.0;
                            let bar_height = 6.0;
                            let (bar_rect, _) = ui.allocate_exact_size(
                                egui::vec2(bar_width, bar_height),
                                egui::Sense::hover(),
                            );
                            if ui.is_rect_visible(bar_rect) {
                                ui.painter().rect_filled(
                                    bar_rect,
                                    egui::CornerRadius::same(3),
                                    egui::Color32::from_rgba_premultiplied(255, 255, 255, 30),
                                );
                                let filled_w = (bar_rect.width() as f64 * progress_ratio) as f32;
                                if filled_w > 0.0 {
                                    let filled_rect = egui::Rect::from_min_size(
                                        bar_rect.min,
                                        egui::vec2(filled_w, bar_height),
                                    );
                                    ui.painter().rect_filled(
                                        filled_rect,
                                        egui::CornerRadius::same(3),
                                        egui::Color32::from_rgb(80, 180, 255),
                                    );
                                }
                            }

                            ui.add_space(2.0);

                            // 大小文本
                            let down_mb = prog_down as f64 / 1_048_576.0;
                            let total_mb = prog_total as f64 / 1_048_576.0;
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} MB / {:.1} MB  ({:.0}%)",
                                    down_mb,
                                    total_mb,
                                    progress_ratio * 100.0
                                ))
                                .color(egui::Color32::from_rgb(180, 200, 220))
                                .size(11.0),
                            );
                        } else if in_progress && prog_total == 0 {
                            ui.label(egui::RichText::new(&prog_msg).color(text_color).size(12.0));
                        }

                        // 第三行：SHA256 校验 + 状态消息
                        // 当 prog_total == 0 时，prog_msg 已在第二行显示，跳过重复
                        if !(prog_msg.is_empty() || (in_progress && prog_total == 0)) {
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                if let Some(ok) = sha256_ok {
                                    let (icon, hash_color) = if ok {
                                        ("✓", egui::Color32::from_rgb(100, 220, 100))
                                    } else {
                                        ("✗", egui::Color32::from_rgb(255, 80, 80))
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("SHA256 {}", icon))
                                            .color(hash_color)
                                            .size(11.0),
                                    );
                                    ui.add_space(4.0);
                                }
                                ui.label(
                                    egui::RichText::new(&prog_msg).color(text_color).size(11.0),
                                );
                            });
                        }

                        // 第四行：失败详情
                        if let Some(ref err) = fail_error {
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(err)
                                    .color(egui::Color32::from_rgb(255, 140, 140))
                                    .size(11.0),
                            );
                        }

                        // 第五行：操作按钮（Done / Failed / 下载中取消）
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if in_progress {
                                // 下载/安装中：后台 + 取消按钮
                                if ui
                                    .add(
                                        egui::Button::new(" 后台下载 ")
                                            .min_size(egui::vec2(80.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    self.toast_background = true;
                                }
                                if ui
                                    .add(
                                        egui::Button::new("x 取消")
                                            .min_size(egui::vec2(60.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    self.cancelled.store(true, Ordering::Relaxed);
                                    *self.state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        UpdateState::Idle;
                                }
                            } else if is_done {
                                // 完成：重启 + 稍后
                                if ui
                                    .add(
                                        egui::Button::new(" 🔄 立即重启 ")
                                            .min_size(egui::vec2(90.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    #[cfg(target_os = "macos")]
                                    {
                                        // macOS 没有独立 updater 进程，需由 app 自己启动新版本
                                        if let Some(p) = done_exe_path.clone() {
                                            let _ =
                                                std::process::Command::new("open").arg(p).spawn();
                                        }
                                    }
                                    // Windows 下不要在此处启动程序：updater.exe 已在 apply_update
                                    // 时被 spawn，它会在旧进程（通过 PID）退出后完成文件替换并负责
                                    // 启动新版本。若这里先 spawn current_exe（仍是旧版），其进程名与
                                    // 旧进程相同，会导致 updater 按进程名等待时永远等不到退出，从而
                                    // 不执行替换 —— 表现为「重启后是旧版，关闭后新版才起」。
                                    // 这里只写 relaunch 标记：updater 检查到标记才启动新版本；
                                    // 不写则仅替换、不自动重启（解决「关闭后仍自动重启」）。
                                    if let Ok(exe) = std::env::current_exe() {
                                        let _ = fs::write(exe.with_extension("exe.relaunch"), b"");
                                    }
                                    std::process::exit(0);
                                }
                                if ui
                                    .add(
                                        egui::Button::new(" 稍后 ")
                                            .min_size(egui::vec2(60.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    *self.state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        UpdateState::Idle;
                                }
                            } else if is_failed {
                                // 失败：重试 + 关闭
                                if retry_info.is_some()
                                    && ui
                                        .add(
                                            egui::Button::new(" 🔄 重试 ")
                                                .min_size(egui::vec2(70.0, 24.0)),
                                        )
                                        .clicked()
                                    && let Some(ref info) = retry_info
                                {
                                    // 重置状态并重新开始下载
                                    *self.state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        UpdateState::Idle;
                                    self.start_update_thread(info.clone(), ui.ctx().clone());
                                }
                                if ui
                                    .add(
                                        egui::Button::new("x 关闭")
                                            .min_size(egui::vec2(60.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    *self.state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        UpdateState::Idle;
                                }
                            }
                        });
                    });

                    if in_progress {
                        ui.ctx().request_repaint();
                    }
                });
        }
    }

    /// 启动更新下载和安装线程
    fn start_update_thread(&mut self, info: UpdateInfo, ctx: egui::Context) {
        let state = self.state.clone();
        let progress = self.progress.clone();
        let cancelled = self.cancelled.clone();
        cancelled.store(false, Ordering::Relaxed);
        {
            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
            *s = UpdateState::Downloading;
        }
        {
            let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
            p.message = "正在连接服务器...".to_string();
            p.bytes_downloaded = 0;
            p.bytes_total = 0;
            p.sha256_ok = None;
        }
        self.toast_shown_done_or_failed = false;
        ctx.request_repaint();
        std::thread::spawn(move || {
            let result = (|| -> Result<std::path::PathBuf, String> {
                let zip_path = crate::update::download_update(&info, &progress, &cancelled)?;
                // 如果中途被取消，直接返回
                if cancelled.load(Ordering::Relaxed) {
                    let _ = std::fs::remove_file(&zip_path);
                    return Err("已取消".to_string());
                }
                *state.lock().unwrap_or_else(|e| e.into_inner()) = UpdateState::Installing;
                {
                    let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
                    p.message = "正在安装...".to_string();
                }
                ctx.request_repaint();
                // 安装前再次检查取消
                if cancelled.load(Ordering::Relaxed) {
                    return Err("已取消".to_string());
                }
                let new_exe = crate::update::apply_update(&zip_path)?;
                Ok(new_exe)
            })();
            // 如果已取消，回退到 Idle（防止 Installing 被设置后取消导致状态卡住）
            if cancelled.load(Ordering::Relaxed) {
                *state.lock().unwrap_or_else(|e| e.into_inner()) = UpdateState::Idle;
                return;
            }
            match result {
                Ok(new_exe) => {
                    *state.lock().unwrap_or_else(|e| e.into_inner()) = UpdateState::Done(new_exe);
                    if let Ok(mut p) = progress.lock() {
                        p.message = "安装完成".to_string();
                    }
                }
                Err(e) => {
                    *state.lock().unwrap_or_else(|e| e.into_inner()) = UpdateState::Failed(e);
                }
            }
            ctx.request_repaint();
        });
    }
}
