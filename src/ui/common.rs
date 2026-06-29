use eframe::egui;

/// 表格单元格背景色
pub const ROW_NUM_BG: egui::Color32 = egui::Color32::from_rgb(235, 235, 235);
pub const TEXT_BG: egui::Color32 = egui::Color32::from_rgb(234, 237, 245);
pub const CODE_BG: egui::Color32 = egui::Color32::from_rgb(234, 245, 237);

/// 渲染固定宽度的表格单元格
pub fn render_cell<F>(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    align_center: bool,
    bg_color: Option<egui::Color32>,
    content: F,
) where
    F: FnOnce(&mut egui::Ui),
{
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    let layout = if align_center {
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
    } else {
        egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(false)
    };
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));

    if let Some(color) = bg_color {
        child_ui
            .painter()
            .rect_filled(child_ui.max_rect(), 0.0, color);
    }

    content(&mut child_ui);
}
