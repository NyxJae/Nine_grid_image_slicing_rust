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
            
            // 关于菜单
            if ui.button("ℹ 关于...").clicked() {
                show_about(state);
            }
        });
    });
    
    // 渲染关于对话框
    render_about_dialog(ctx, state);
    
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

/// 显示关于对话框
fn show_about(state: &mut AppState) {
    state.show_about_dialog = true;
}

/// 渲染关于对话框
fn render_about_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_about_dialog {
        return;
    }
    
    egui::Window::new("ℹ 关于")
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("将可九宫格的大图进行快速切图的便捷工具");
                ui.add_space(10.0);

                ui.label("版本：1.0.0");
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label("项目地址：");
                    ui.hyperlink_to(
                        "GitHub",
                        "https://github.com/NyxJae/Nine_grid_image_slicing_rust"
                    );
                });
                ui.add_space(10.0);
                
                ui.label("使用 Rust + egui 构建");
                ui.add_space(5.0);
                
                ui.label("© 2025 NyxJae HJ");
            });
            
            ui.add_space(10.0);
            ui.separator();
            
            ui.horizontal(|ui| {
                if ui.button("关闭").clicked() {
                    state.show_about_dialog = false;
                }
            });
        });
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
