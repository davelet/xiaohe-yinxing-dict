use eframe::egui;

/// 词典视图样式（浅色、简洁）
pub struct DictViewStyle;

impl DictViewStyle {
    /// 应用词典视图样式
    pub fn apply(style: &mut egui::Style) {
        style.visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
        style.visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    }
}

/// 输入法数据视图样式（浅色、简洁，布局风格不同于默认数据视图）
pub struct ManagerViewStyle;

impl ManagerViewStyle {
    /// 获取卡片背景色
    pub fn card_background() -> egui::Color32 {
        egui::Color32::from_rgb(240, 240, 245)
    }

    /// 获取成功色
    pub fn success_color() -> egui::Color32 {
        egui::Color32::from_rgb(34, 150, 80)
    }

    /// 应用管理视图样式
    pub fn apply(style: &mut egui::Style) {
        style.visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
        style.visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.window_margin.bottom = 0;
    }
}
