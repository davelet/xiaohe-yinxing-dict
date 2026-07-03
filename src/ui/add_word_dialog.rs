use crate::app::ManagerState;
use eframe::egui;

const DIALOG_WIDTH: f32 = 360.0;

pub fn show_add_word_dialog(state: &mut ManagerState, ctx: &egui::Context) {
    let mut close_dialog = false;

    egui::Window::new("添加新词")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size(egui::vec2(DIALOG_WIDTH, 0.0))
        .show(ctx, |ui| {
            if state.add_word_dialog_auto_focus {
                ui.memory_mut(|mem| mem.request_focus(egui::Id::new("dialog_add_word_text")));
                state.add_word_dialog_auto_focus = false;
            }
            ui.label("文字:");
            ui.add(
                egui::TextEdit::singleline(&mut state.new_word_text)
                    .id("dialog_add_word_text".into())
                    .hint_text("输入文字或词组")
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(8.0);

            ui.label("编码:");
            ui.add(
                egui::TextEdit::singleline(&mut state.new_word_code)
                    .id("dialog_add_word_code".into())
                    .hint_text("小鹤编码（小写字母）")
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(12.0);

            let text_focused = ui.memory(|mem| mem.has_focus(egui::Id::new("dialog_add_word_text")));
            let code_focused = ui.memory(|mem| mem.has_focus(egui::Id::new("dialog_add_word_code")));

            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

            if escape_pressed {
                close_dialog = true;
            }

            ui.horizontal(|ui| {
                if ui.button("添加").clicked()
                    || (enter_pressed && (text_focused || code_focused))
                {
                    state.add_new_word();
                    if state.add_word_feedback.as_ref().is_some_and(|(_, success)| *success) {
                        close_dialog = true;
                        state.show_add_word_dialog = false;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("取消").clicked() {
                        close_dialog = true;
                    }
                });
            });

            if let Some((msg, success)) = &state.add_word_feedback {
                ui.add_space(8.0);
                let color = if *success {
                    egui::Color32::from_rgb(30, 120, 50)
                } else {
                    egui::Color32::from_rgb(160, 20, 20)
                };
                ui.colored_label(color, msg);
            }
        });

    if close_dialog {
        state.show_add_word_dialog = false;
        state.new_word_text.clear();
        state.new_word_code.clear();
        state.add_word_feedback = None;
        state.add_word_timer = 0.0;
    }
}
