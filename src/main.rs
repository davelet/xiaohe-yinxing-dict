#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use eframe::egui;

pub mod app;
pub mod config;
pub mod dict;
pub mod help;
pub mod icon;
pub mod rime_loader;
pub mod search;
mod trie;
mod ui;
pub mod update;

use dict::Category;
use help::HelpManager;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Text,
    Code,
}
use dict::DictEntry;
use search::SearchEngine;

mod dict_data;
use dict_data::DICT_ENTRIES;

impl DictApp {
    fn new(engine: SearchEngine<DictEntry>, ctx: &egui::Context) -> Self {
        update::cleanup_old_files_keep_previous();

        let categories = dict::DictEntry::all_categories();
        let help_image = Self::load_help_image(ctx);
        let help_manager = HelpManager::new();
        let update_info = Arc::new(Mutex::new(None));
        let update_info_clone = update_info.clone();
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            Self::check_update_background(update_info_clone, ctx_clone);
        });

        Self {
            engine,
            categories,
            query: String::new(),
            last_query: String::new(),
            selected_category: None,
            last_category: None,
            search_results: Vec::new(),
            total_results: 0,
            copied_feedback: None,
            feedback_timer: 0.0,
            help_image,
            show_help_image: false,
            help_manager,
            show_help_panel: false,
            selected_help_chapter: None,
            help_search_query: String::new(),
            help_search_auto_navigated: false,
            update_info,
            show_update_dialog: false,
            update_info_for_dialog: None,
            update_state: Arc::new(Mutex::new(update::UpdateState::Idle)),
            update_progress: Arc::new(Mutex::new(update::UpdateProgress {
                message: String::new(),
                bytes_downloaded: 0,
                bytes_total: 0,
                sha256_ok: None,
            })),
            update_started_at: None,
            // 视图切换
            current_view: app::ViewMode::Dict,
            manager: app::ManagerState::new(),
            search_auto_focus: true,
            // 跨视图更新进度 toast
            update_toast_timer: 0.0,
            update_toast_shown_done_or_failed: false,
            update_toast_dismissed: true,
            update_retry_info: None,
            update_cancelled: Arc::new(AtomicBool::new(false)),
            update_toast_background: false,
            show_md_preview: std::cell::Cell::new(true),
        }
    }

    fn check_update_background(
        update_info: Arc<Mutex<Option<update::UpdateInfo>>>,
        ctx: egui::Context,
    ) {
        let current_version = env!("CARGO_PKG_VERSION");
        if let Some(info) = update::check_for_update(current_version) {
            if let Ok(mut guard) = update_info.lock() {
                *guard = Some(info);
            }
            ctx.request_repaint();
        }
    }

    fn load_help_image(ctx: &egui::Context) -> Option<egui::TextureHandle> {
        let image_data = include_bytes!("../assets/xhzg.webp");
        if let Ok(img) = image::load_from_memory(image_data) {
            let size = [img.width() as _, img.height() as _];
            let pixels = img.to_rgba8().into_flat_samples().samples;
            let image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            Some(ctx.load_texture("help_image", image, egui::TextureOptions::default()))
        } else {
            None
        }
    }
}

struct DictApp {
    engine: SearchEngine<DictEntry>,
    categories: Vec<Category>,
    query: String,
    last_query: String,
    selected_category: Option<Category>,
    last_category: Option<Category>,
    search_results: Vec<(usize, search::MatchKind)>,
    total_results: usize,
    copied_feedback: Option<(usize, CopyKind)>,
    feedback_timer: f32,
    help_image: Option<egui::TextureHandle>,
    show_help_image: bool,
    help_manager: HelpManager,
    show_help_panel: bool,
    selected_help_chapter: Option<String>,
    help_search_query: String,
    help_search_auto_navigated: bool,
    update_info: Arc<Mutex<Option<update::UpdateInfo>>>,
    show_update_dialog: bool,
    update_info_for_dialog: Option<update::UpdateInfo>,
    update_state: Arc<Mutex<update::UpdateState>>,
    /// 详细更新进度（跨线程共享给 toast 渲染）
    update_progress: Arc<Mutex<update::UpdateProgress>>,
    /// 更新开始时间（用于计算已用时间）
    update_started_at: Option<std::time::Instant>,
    // 视图切换
    current_view: app::ViewMode,
    /// 管理视图状态（与词典视图完全独立）
    manager: app::ManagerState,
    /// 是否需要在进入视图时自动聚焦搜索框（仅首帧）
    search_auto_focus: bool,
    /// 跨视图更新进度 toast 倒计时（<= 0 时清除，0.0 表示常驻）
    update_toast_timer: f32,
    /// 用于失败后重试的更新信息
    update_retry_info: Option<update::UpdateInfo>,
    /// 取消更新下载的标志
    update_cancelled: Arc<AtomicBool>,
    /// 是否已为当前 Done/Failed 状态显示过 toast（状态回到 Idle 时重置）
    update_toast_shown_done_or_failed: bool,
    /// 标记Idle是否出现过，用来重置 shown_done_or_failed
    update_toast_dismissed: bool,
    /// 后台更新模式：隐藏toast但继续下载
    update_toast_background: bool,
    /// 更新弹窗中 release notes 是否以 Markdown 渲染
    show_md_preview: std::cell::Cell<bool>,
}

fn create_app_icon() -> egui::IconData {
    let size = 64u32;
    let rgba = icon::generate_icon_rgba(size);
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn main() -> eframe::Result {
    let icon = create_app_icon();
    let options = eframe::NativeOptions {
        run_and_return: false,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([950.0, 650.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "小鹤音形词典",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            let engine = SearchEngine::build(&DICT_ENTRIES);
            Ok(Box::new(DictApp::new(engine, &cc.egui_ctx)))
        }),
    )
}

fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!("../fonts/NotoSansSC-Regular.ttf")).into(),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    ctx.set_fonts(fonts);
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}
