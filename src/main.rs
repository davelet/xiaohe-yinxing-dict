#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use eframe::egui;

pub mod dict;
pub mod help;
pub mod icon;
pub mod search;
mod trie;
mod ui;
pub mod update;
pub mod config;
pub mod rime_loader;
pub mod app;

use dict::Category;
use help::HelpManager;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Text,
    Code,
}
use search::SearchEngine;
use dict::DictEntry;

mod dict_data;
use dict_data::DICT_ENTRIES;

impl DictApp {
    fn new(engine: SearchEngine<DictEntry>, ctx: &egui::Context) -> Self {
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
            // 视图切换
            current_view: app::ViewMode::Dict,
            manager: app::ManagerState::new(),
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
    // 视图切换
    current_view: app::ViewMode,
    /// 管理视图状态（与词典视图完全独立）
    manager: app::ManagerState,
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
            .with_resizable(true)
            .with_maximize_button(true)
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
}
