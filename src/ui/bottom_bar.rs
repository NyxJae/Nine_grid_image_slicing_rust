use crate::state::app_state::AppState;
use crate::utils::export_utils;
use crate::image_processing::exporter::ExportMode;

/// 渲染底部按钮区
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            let has_images = state.has_images();
            let is_exporting = state.is_exporting;
            
            // 覆盖原图按钮(警告色)
            if ui
                .add_enabled(
                    has_images && !is_exporting,
                    egui::Button::new("⚠ 覆盖原图")
                        .min_size(egui::vec2(150.0, 40.0))
                        .fill(egui::Color32::from_rgb(200, 100, 50)),
                )
                .clicked()
            {
                export_overwrite(state);
            }
            
            ui.add_space(20.0);
            
            // 导出(添加后缀)按钮(主色调)
            if ui
                .add_enabled(
                    has_images && !is_exporting,
                    egui::Button::new("💾 导出(添加后缀)")
                        .min_size(egui::vec2(150.0, 40.0))
                        .fill(egui::Color32::from_rgb(50, 150, 100)),
                )
                .clicked()
            {
                export_with_suffix(state);
            }
        });
        
        ui.add_space(5.0);
        
        // 显示导出进度
        if state.is_exporting || !state.export_message.is_empty() {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if state.is_exporting {
                    ui.spinner();
                }
                ui.label(&state.export_message);
            });
            
            if state.is_exporting {
                ui.add(
                    egui::ProgressBar::new(state.export_progress)
                        .show_percentage()
                        .animate(true),
                );
            }
        }
    });

    // 显示覆盖确认对话框
    if state.show_overwrite_confirmation {
        egui::Window::new("确认覆盖")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label("⚠ 此操作将直接修改原始图片文件，且不可撤销！");
                ui.label("确定要继续吗？");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("确定覆盖").clicked() {
                        state.show_overwrite_confirmation = false;
                        export_utils::start_export(state, ExportMode::Overwrite);
                    }
                    
                    if ui.button("取消").clicked() {
                        state.show_overwrite_confirmation = false;
                    }
                });
            });
    }
}

/// 覆盖原图导出
fn export_overwrite(state: &mut AppState) {
    state.show_overwrite_confirmation = true;
}

/// 添加后缀导出
fn export_with_suffix(state: &mut AppState) {
    export_utils::start_export(state, ExportMode::WithSuffix("_sliced".to_string()));
}
