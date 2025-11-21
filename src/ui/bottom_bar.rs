use crate::state::app_state::AppState;

/// 渲染底部按钮区
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            let has_images = state.has_images();
            
            // 覆盖原图按钮(警告色)
            if ui
                .add_enabled(
                    has_images,
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
                    has_images,
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
        if state.is_exporting {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(&state.export_message);
            });
            ui.add(
                egui::ProgressBar::new(state.export_progress)
                    .show_percentage()
                    .animate(true),
            );
        }
    });
}

/// 覆盖原图导出
fn export_overwrite(_state: &mut AppState) {
    // TODO: 显示确认对话框
    // TODO: 实现导出逻辑
    println!("覆盖原图导出");
}

/// 添加后缀导出
fn export_with_suffix(_state: &mut AppState) {
    // TODO: 实现导出逻辑
    println!("添加后缀导出");
}
