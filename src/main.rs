#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use eframe::egui;

pub mod ai;
pub mod config;
pub mod dict;
mod external_dict_cache;
pub mod help;
pub mod icon;
pub mod manager;
pub mod menu;
pub mod rime_loader;
pub mod search;
mod trie;
pub mod types;
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
use ui::UpdateUiState;
use ui::chat_state::ChatUiState;

mod dict_data;
use dict_data::DICT_ENTRIES;

impl DictApp {
    fn new(engine: SearchEngine<DictEntry>, ctx: &egui::Context) -> Self {
        update::cleanup_old_update_files();

        let categories = dict::DictEntry::all_categories();
        let help_image = Self::load_help_image(ctx);
        let help_manager = Arc::new(HelpManager::new());
        let update_ui = UpdateUiState::new();
        let update_info_clone = update_ui.info.clone();
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            Self::check_update_background(update_info_clone, ctx_clone);
        });

        // 菜单延迟到第一个 UI 帧初始化，避免在 app_did_finish_launching 回调栈内调用引发崩溃

        Self {
            engine: Arc::new(engine),
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
            help_image_shown_at: None,
            help_manager,
            show_help_panel: false,
            selected_help_chapter: None,
            help_search_query: String::new(),
            help_search_auto_navigated: false,
            update_ui,
            // 视图切换
            current_view: types::ViewMode::Dict,
            manager: manager::ManagerState::new(),
            search_auto_focus: true,
            show_about_dialog: false,
            search_dirty: true,
            cached_category_count: None,
            chat: {
                let mut chat = ChatUiState::new();
                chat.tokio_runtime = Some(
                    tokio::runtime::Runtime::new()
                        .expect("无法初始化 Tokio 运行时，AI 对话功能将不可用"),
                );
                chat
            },
            menu_initialized: false,
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
    engine: Arc<SearchEngine<DictEntry>>,
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
    /// 是否显示键位图（点击“键位图”按钮切换）
    show_help_image: bool,
    help_image_shown_at: Option<std::time::Instant>,
    help_manager: Arc<HelpManager>,
    show_help_panel: bool,
    selected_help_chapter: Option<String>,
    help_search_query: String,
    help_search_auto_navigated: bool,
    /// 自动更新 UI 状态（弹窗 + 跨视图 toast + 下载线程）
    update_ui: UpdateUiState,
    // 视图切换
    current_view: types::ViewMode,
    /// 管理视图状态（与词典视图完全独立）
    manager: manager::ManagerState,
    /// 是否需要在进入视图时自动聚焦搜索框（仅首帧）
    search_auto_focus: bool,
    /// 关于本软件的介绍弹窗
    show_about_dialog: bool,
    /// 标记搜索是否需要重新执行（避免每帧重复搜索）
    search_dirty: bool,
    /// 分类条目数缓存：(分类, 数量)；仅在 selected_category 变化时失效
    cached_category_count: Option<(Option<crate::dict::Category>, usize)>,
    /// AI 对话全部状态（输入、对话历史、流式响应、模型编辑草稿等）
    chat: ChatUiState,
    /// 原生菜单是否已初始化。
    ///
    /// 菜单不能放在 `DictApp::new()` 中初始化，因为 eframe 的 init 闭包运行在
    /// NSApplicationDelegate 的 app_did_finish_launching 回调栈内，此时调用
    /// [NSApp setMainMenu:] 会触发 ObjC 回调栈冲突导致 panic → abort。
    /// 因此延迟到第一帧 `App::ui()` 中执行。
    ///
    /// 详见 src/ui/mod.rs 和 src/menu.rs 中的相关笔记。
    menu_initialized: bool,
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
