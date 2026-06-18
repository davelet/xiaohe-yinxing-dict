use crate::DictApp;
use eframe::egui;

pub(crate) fn render_help_fullscreen(app: &mut DictApp, ui: &mut egui::Ui) {
    // Top bar: title + back button
    ui.horizontal(|ui| {
        if ui.button("← 返回词典").clicked() {
            app.show_help_panel = false;
        }
        ui.heading("📖 帮助文档");
    });
    ui.separator();

    // Two-column layout
    let chapters = app.help_manager.chapters().to_vec();
    let current_chapter = app.selected_help_chapter.clone();

    egui::Panel::left("help_sidebar")
        .resizable(false)
        .min_size(180.0)
        .max_size(250.0)
        .default_size(200.0)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("help_chapter_list")
                .show(ui, |ui| {
                    for chapter in &chapters {
                        let is_selected = current_chapter.as_deref() == Some(chapter.id);
                        let button = ui.selectable_label(is_selected, chapter.title);
                        if button.clicked() {
                            app.selected_help_chapter = Some(chapter.id.to_string());
                        }
                    }
                });
        });

    // Content fills the remaining central area
    egui::CentralPanel::default().show_inside(ui, |ui| {
        let chapter_id = app.selected_help_chapter.clone();
        let mut nav = crate::help::HelpNav { goto: None };
        if let Some(id) = &chapter_id {
            egui::ScrollArea::vertical()
                .id_salt("help_content")
                .show(ui, |ui| {
                    if let Some(chapter) = app.help_manager.get_chapter(id) {
                        crate::help::reset_table_counter();
                        (chapter.render)(ui, &mut nav);
                    }
                });
        } else {
            ui.label("请从左侧选择帮助章节");
        }
        if let Some(target) = nav.goto {
            app.selected_help_chapter = Some(target.to_string());
        }
    });
}
