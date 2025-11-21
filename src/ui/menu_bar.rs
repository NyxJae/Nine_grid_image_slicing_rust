use crate::state::app_state::AppState;

/// 渲染菜单栏
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // 文件菜单
            ui.menu_button("文件(F)", |ui| {
                if ui.button("📁 导入图片... (Ctrl+O)").clicked() {
                    import_images(state);
                    ui.close_menu();
                }
                
                ui.separator();
                
                let has_images = state.has_images();
                
                if ui
                    .add_enabled(has_images, egui::Button::new("💾 覆盖原图导出 (Ctrl+S)"))
                    .clicked()
                {
                    export_overwrite(state);
                    ui.close_menu();
                }
                
                if ui
                    .add_enabled(
                        has_images,
                        egui::Button::new("📤 添加后缀导出... (Ctrl+Shift+S)"),
                    )
                    .clicked()
                {
                    export_with_suffix(state);
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("❌ 退出 (Alt+F4)").clicked() {
                    std::process::exit(0);
                }
            });
            
            // 帮助菜单
            ui.menu_button("帮助(H)", |ui| {
                if ui.button("📖 使用说明...").clicked() {
                    show_help();
                    ui.close_menu();
                }
                
                if ui.button("ℹ 关于...").clicked() {
                    show_about();
                    ui.close_menu();
                }
            });
        });
    });
    
    // 处理快捷键
    handle_shortcuts(ctx, state);
}

/// 导入图片
fn import_images(state: &mut AppState) {
    use crate::image_processing::loader;
    
    // 打开文件选择对话框
    if let Some(paths) = rfd::FileDialog::new()
        .add_filter("PNG图片", &["png", "PNG"])
        .set_title("选择PNG图片")
        .pick_files()
    {
        // 加载图片
        match loader::load_images(paths) {
            Ok(images) => {
                state.add_images(images);
            }
            Err(e) => {
                state.set_error(format!("导入图片失败: {}", e));
            }
        }
    }
}

use crate::utils::export_utils;
use crate::image_processing::exporter::ExportMode;

/// 覆盖原图导出
fn export_overwrite(state: &mut AppState) {
    if !state.has_images() {
        return;
    }
    
    state.show_overwrite_confirmation = true;
}

/// 添加后缀导出
fn export_with_suffix(state: &mut AppState) {
    if !state.has_images() {
        return;
    }
    
    export_utils::start_export(state, ExportMode::WithSuffix("_sliced".to_string()));
}

/// 显示使用说明
fn show_help() {
    // TODO: 实现使用说明对话框
    println!("显示使用说明");
}

/// 显示关于对话框
fn show_about() {
    // TODO: 实现关于对话框
    println!("显示关于");
}

/// 处理快捷键
fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) {
    // Ctrl+O: 导入图片
    if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
        import_images(state);
    }
    
    // Ctrl+S: 覆盖原图导出
    if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && !i.modifiers.shift) {
        if state.has_images() {
            export_overwrite(state);
        }
    }
    
    // Ctrl+Shift+S: 添加后缀导出
    if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift) {
        if state.has_images() {
            export_with_suffix(state);
        }
    }
}
