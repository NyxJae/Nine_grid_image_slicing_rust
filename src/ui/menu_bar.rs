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
                
                if ui.button("ℹ️ 关于...").clicked() {
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

/// 覆盖原图导出
fn export_overwrite(state: &mut AppState) {
    if !state.has_images() {
        return;
    }
    
    // TODO: 显示确认对话框
    // 目前直接执行导出
    do_export(state, crate::image_processing::exporter::ExportMode::Overwrite);
}

/// 添加后缀导出
fn export_with_suffix(state: &mut AppState) {
    if !state.has_images() {
        return;
    }
    
    do_export(
        state,
        crate::image_processing::exporter::ExportMode::WithSuffix("_cut".to_string()),
    );
}

/// 执行导出
fn do_export(state: &mut AppState, mode: crate::image_processing::exporter::ExportMode) {
    use crate::image_processing::exporter;
    
    // 设置导出状态
    state.is_exporting = true;
    state.export_progress = 0.0;
    state.export_message = "正在导出...".to_string();
    
    // 直接导出图片(不使用回调,避免闭包修改问题)
    let mut results = Vec::new();
    let total = state.images.len();
    
    for (index, item) in state.images.iter().enumerate() {
        // 更新进度
        state.export_progress = (index + 1) as f32 / total as f32;
        state.export_message = format!("正在导出 {}/{}", index + 1, total);
        
        // 导出图片
        let result = exporter::export_image(item, &mode);
        results.push((item.file_path.clone(), result));
    }
    
    // 处理结果
    let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
    let fail_count = results.len() - success_count;
    
    if fail_count == 0 {
        state.export_message = format!("成功导出 {} 张图片", success_count);
    } else {
        state.export_message = format!(
            "导出完成: 成功 {} 张, 失败 {} 张",
            success_count, fail_count
        );
        
        // 收集错误信息
        let errors: Vec<String> = results
            .iter()
            .filter_map(|(path, r)| {
                r.as_ref().err().map(|e| {
                    format!("{}: {}", path.file_name().unwrap_or_default().to_string_lossy(), e)
                })
            })
            .collect();
        
        if !errors.is_empty() {
            eprintln!("导出错误:\n{}", errors.join("\n"));
        }
    }
    
    state.export_progress = 1.0;
    
    // 延迟一下让用户看到完成消息
    std::thread::sleep(std::time::Duration::from_secs(1));
    state.is_exporting = false;
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
