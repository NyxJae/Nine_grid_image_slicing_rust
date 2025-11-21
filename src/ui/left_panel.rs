use crate::state::app_state::AppState;
use image::DynamicImage;

/// 渲染左侧操作区
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // 检测文件拖放
    ui.ctx().input(|i| {
        // 处理已拖放的文件
        for file in &i.raw.dropped_files {
            if let Some(path) = &file.path {
                // 验证是PNG文件
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "png" {
                        // 加载图片
                        match crate::image_processing::loader::load_image(path.clone()) {
                            Ok(image_item) => {
                                state.add_image(image_item);
                            }
                            Err(e) => {
                                eprintln!("加载图片失败: {}", e);
                                state.set_error(format!("加载图片失败: {}", e));
                            }
                        }
                    } else {
                        state.set_error(format!("不支持的文件格式: {:?}\n只支持PNG格式", ext));
                    }
                } else {
                    state.set_error("文件没有扩展名,只支持PNG格式".to_string());
                }
            }
        }
    });
    
    // 检测是否有文件正在悬停
    let is_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    
    // 如果有文件悬停,显示高亮边框
    if is_hovering {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 200, 255)),
        );
    }
    
    egui::ScrollArea::vertical()
        .id_source("left_panel_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if state.images.is_empty() {
                // 显示空状态提示
                render_empty_state(ui, state, is_hovering);
            } else {
                // 显示图片列表
                render_image_list(ui, state);
            }
        });
}

/// 渲染空状态提示
fn render_empty_state(ui: &mut egui::Ui, _state: &mut AppState, is_hovering: bool) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        
        if is_hovering {
            ui.heading("📥 释放以导入图片");
            ui.add_space(5.0);
            ui.label(egui::RichText::new("拖放PNG文件到此处").color(egui::Color32::from_rgb(100, 200, 255)));
        } else {
            ui.heading("📁 拖放PNG图片到此处");
            ui.add_space(10.0);
            ui.label("或点击 \"文件 → 导入图片\"");
        }
        
        ui.add_space(20.0);
        
        // 支持的格式
        ui.group(|ui| {
            ui.label("支持的格式:");
            ui.label("• PNG (.png)");
        });
    });
}

/// 渲染图片列表
fn render_image_list(ui: &mut egui::Ui, state: &mut AppState) {
    let num_images = state.images.len();
    let mut image_to_remove: Option<usize> = None;
    
    for i in 0..num_images {
        let is_selected = state.selected_index == Some(i);
        
        // 图片处理单元
        let should_remove = egui::Frame::none()
            .fill(if is_selected {
                ui.style().visuals.extreme_bg_color
            } else {
                ui.style().visuals.window_fill
            })
            .inner_margin(egui::Margin::same(10.0))
            .outer_margin(egui::Margin::symmetric(5.0, 5.0))
            .rounding(5.0)
            .stroke(if is_selected {
                egui::Stroke::new(2.0, ui.style().visuals.selection.bg_fill)
            } else {
                egui::Stroke::NONE
            })
            .show(ui, |ui| {
                render_image_item(ui, state, i)
            })
            .inner;
        
        // 记录要删除的图片索引
        if should_remove {
            image_to_remove = Some(i);
        }
    }
    
    // 在循环外删除图片,避免索引越界
    if let Some(index) = image_to_remove {
        state.remove_image(index);
    }
    
    // 拖放提示区域
    ui.add_space(20.0);
    render_drop_hint(ui);
}

/// 渲染单个图片处理单元
/// 返回是否应该删除这个图片
fn render_image_item(ui: &mut egui::Ui, state: &mut AppState, index: usize) -> bool {
    // 点击选中
    if ui
        .interact(
            ui.available_rect_before_wrap(),
            ui.id().with(("image_item", index)),
            egui::Sense::click(),
        )
        .clicked()
    {
        state.select_image(index);
    }
    
    // 图片信息栏(带关闭按钮)
    let mut should_remove = false;
    ui.horizontal(|ui| {
        ui.label(format!(
            "📄 {} ({}×{})",
            state.images[index].file_name,
            state.images[index].width,
            state.images[index].height
        ));
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("❌ 关闭").clicked() {
                should_remove = true;
            }
        });
    });
    
    // 如果点击了关闭按钮,直接返回,不再渲染其他内容
    if should_remove {
        return true;
    }
    
    ui.separator();
    
    // 图片操作区(显示图片和切割线)
    render_image_with_guides(ui, state, index);
    
    ui.separator();
    
    // 参数控制区
    render_params_controls(ui, state, index);
    
    // 生成预览图片(如果参数改变)
    generate_preview(state, index);
    
    false
}

/// 渲染图片和切割线
fn render_image_with_guides(ui: &mut egui::Ui, state: &mut AppState, index: usize) {
    // 获取图片尺寸(复制值,避免持有引用)
    let (width, height) = {
        let img = &state.images[index];
        (img.width, img.height)
    };
    
    // 计算显示尺寸(保持宽高比,适应UI)
    let available_width = ui.available_width().max(100.0);
    let max_height = 250.0;
    
    // 安全检查:确保image尺寸有效
    if width == 0 || height == 0 {
        ui.label("图片尺寸无效");
        return;
    }
    
    let aspect_ratio = width as f32 / height as f32;
    
    // 安全检查:确保aspect_ratio有效
    if !aspect_ratio.is_finite() || aspect_ratio <= 0.0 {
        ui.label("图片宽高比无效");
        return;
    }
    
    let (display_width, display_height) = if available_width / max_height > aspect_ratio {
        let h = max_height;
        let w = h * aspect_ratio;
        (w, h)
    } else {
        let w = available_width;
        let h = w / aspect_ratio;
        (w.max(1.0), h.max(1.0))
    };
    
    // 计算缩放比例
    let scale_x = display_width / width as f32;
    let scale_y = display_height / height as f32;
    
    // 预留绘制区域
    let (response, painter) = ui.allocate_painter(
        egui::vec2(display_width, display_height),
        egui::Sense::hover(),
    );
    
    let rect = response.rect;
    
    // 绘制图片背景(棋盘格,用于透明图片)
    draw_checkerboard(&painter, rect);
    
    // 绘制实际图片
    // 如果没有缩略图纹理,生成一个
    if state.images[index].thumbnail_texture.is_none() {
        let img = &state.images[index].original_image;
        
        // 缩放到合适大小作为缩略图(最大边长500像素,保证清晰度)
        let max_size = 500;
        let (new_w, new_h) = if img.width() > max_size || img.height() > max_size {
            let ratio = img.width() as f32 / img.height() as f32;
            if ratio > 1.0 {
                (max_size, (max_size as f32 / ratio) as u32)
            } else {
                ((max_size as f32 * ratio) as u32, max_size)
            }
        } else {
            (img.width(), img.height())
        };
        
        let thumbnail = img.resize(
            new_w, 
            new_h, 
            image::imageops::FilterType::Lanczos3
        );
        
        let color_image = dynamic_image_to_color_image(&thumbnail);
        
        let texture = ui.ctx().load_texture(
            format!("thumb_{}", state.images[index].file_name),
            color_image,
            egui::TextureOptions::default(),
        );
        
        state.images[index].thumbnail_texture = Some(texture);
    }
    
    // 绘制纹理
    if let Some(texture) = &state.images[index].thumbnail_texture {
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        // 降级显示
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(100));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "加载中...",
            egui::FontId::default(),
            egui::Color32::WHITE,
        );
    }
    
    // 获取当前的切割参数(复制值)
    let (top, bottom, left, right) = {
        let img = &state.images[index];
        (img.top, img.bottom, img.left, img.right)
    };
    
    // 计算切割线位置
    let top_y = rect.top() + top as f32 * scale_y;
    let bottom_y = rect.bottom() - bottom as f32 * scale_y;
    let left_x = rect.left() + left as f32 * scale_x;
    let right_x = rect.right() - right as f32 * scale_x;
    
    let line_color = egui::Color32::from_rgb(255, 100, 100);
    let line_width = 2.0;
    let handle_radius = 8.0;
    
    // 绘制切割线
    painter.line_segment(
        [egui::pos2(rect.left(), top_y), egui::pos2(rect.right(), top_y)],
        egui::Stroke::new(line_width, line_color),
    );
    painter.line_segment(
        [egui::pos2(rect.left(), bottom_y), egui::pos2(rect.right(), bottom_y)],
        egui::Stroke::new(line_width, line_color),
    );
    painter.line_segment(
        [egui::pos2(left_x, rect.top()), egui::pos2(left_x, rect.bottom())],
        egui::Stroke::new(line_width, line_color),
    );
    painter.line_segment(
        [egui::pos2(right_x, rect.top()), egui::pos2(right_x, rect.bottom())],
        egui::Stroke::new(line_width, line_color),
    );
    
    // 绘制并处理拖动手柄
    let mut needs_update = false;
    
    // 上手柄
    let top_handle_pos = egui::pos2(rect.center().x, top_y);
    let top_handle_id = ui.id().with((index, "top_handle"));
    let top_handle_rect = egui::Rect::from_center_size(top_handle_pos, egui::vec2(handle_radius * 2.0, handle_radius * 2.0));
    let top_handle_response = ui.interact(top_handle_rect, top_handle_id, egui::Sense::drag());
    
    if top_handle_response.dragged() {
        let delta_y = top_handle_response.drag_delta().y;
        
        // 使用累加器处理亚像素移动
        let acc_id = ui.id().with((index, "top_acc"));
        let mut acc = ui.data(|d| d.get_temp::<f32>(acc_id).unwrap_or(0.0));
        
        acc += delta_y / scale_y;
        
        let param_delta = acc.trunc() as i32;
        if param_delta != 0 {
            let new_top = (state.images[index].top as i32 + param_delta).max(0) as u32;
            state.images[index].top = new_top;
            acc -= param_delta as f32;
            needs_update = true;
        }
        
        ui.data_mut(|d| d.insert_temp(acc_id, acc));
    }
    
    let top_color = if top_handle_response.hovered() {
        egui::Color32::from_rgb(255, 200, 200)
    } else {
        egui::Color32::WHITE
    };
    painter.circle_filled(top_handle_pos, handle_radius, top_color);
    painter.circle_stroke(top_handle_pos, handle_radius, egui::Stroke::new(2.0, line_color));
    
    // 下手柄
    let bottom_handle_pos = egui::pos2(rect.center().x, bottom_y);
    let bottom_handle_id = ui.id().with((index, "bottom_handle"));
    let bottom_handle_rect = egui::Rect::from_center_size(bottom_handle_pos, egui::vec2(handle_radius * 2.0, handle_radius * 2.0));
    let bottom_handle_response = ui.interact(bottom_handle_rect, bottom_handle_id, egui::Sense::drag());
    
    if bottom_handle_response.dragged() {
        let delta_y = bottom_handle_response.drag_delta().y;
        
        // 使用累加器处理亚像素移动
        let acc_id = ui.id().with((index, "bottom_acc"));
        let mut acc = ui.data(|d| d.get_temp::<f32>(acc_id).unwrap_or(0.0));
        
        // 注意: 向下拖动增加bottom值(因为bottom是距离底部的距离,向下拖意味着bottom变小? 不,bottom是距离底部的像素数)
        // 让我们重新理一下:
        // rect.bottom() - bottom * scale
        // 向下拖动 -> delta_y > 0 -> bottom_y 变大 -> bottom 应该变小
        // 所以 delta_y > 0 对应 bottom 减小
        acc += -delta_y / scale_y;
        
        let param_delta = acc.trunc() as i32;
        if param_delta != 0 {
            let new_bottom = (state.images[index].bottom as i32 + param_delta).max(0) as u32;
            state.images[index].bottom = new_bottom;
            acc -= param_delta as f32;
            needs_update = true;
        }
        
        ui.data_mut(|d| d.insert_temp(acc_id, acc));
    }
    
    let bottom_color = if bottom_handle_response.hovered() {
        egui::Color32::from_rgb(255, 200, 200)
    } else {
        egui::Color32::WHITE
    };
    painter.circle_filled(bottom_handle_pos, handle_radius, bottom_color);
    painter.circle_stroke(bottom_handle_pos, handle_radius, egui::Stroke::new(2.0, line_color));
    
    // 左手柄
    let left_handle_pos = egui::pos2(left_x, rect.center().y);
    let left_handle_id = ui.id().with((index, "left_handle"));
    let left_handle_rect = egui::Rect::from_center_size(left_handle_pos, egui::vec2(handle_radius * 2.0, handle_radius * 2.0));
    let left_handle_response = ui.interact(left_handle_rect, left_handle_id, egui::Sense::drag());
    
    if left_handle_response.dragged() {
        let delta_x = left_handle_response.drag_delta().x;
        
        // 使用累加器处理亚像素移动
        let acc_id = ui.id().with((index, "left_acc"));
        let mut acc = ui.data(|d| d.get_temp::<f32>(acc_id).unwrap_or(0.0));
        
        acc += delta_x / scale_x;
        
        let param_delta = acc.trunc() as i32;
        if param_delta != 0 {
            let new_left = (state.images[index].left as i32 + param_delta).max(0) as u32;
            state.images[index].left = new_left;
            acc -= param_delta as f32;
            needs_update = true;
        }
        
        ui.data_mut(|d| d.insert_temp(acc_id, acc));
    }
    
    let left_color = if left_handle_response.hovered() {
        egui::Color32::from_rgb(255, 200, 200)
    } else {
        egui::Color32::WHITE
    };
    painter.circle_filled(left_handle_pos, handle_radius, left_color);
    painter.circle_stroke(left_handle_pos, handle_radius, egui::Stroke::new(2.0, line_color));
    
    // 右手柄
    let right_handle_pos = egui::pos2(right_x, rect.center().y);
    let right_handle_id = ui.id().with((index, "right_handle"));
    let right_handle_rect = egui::Rect::from_center_size(right_handle_pos, egui::vec2(handle_radius * 2.0, handle_radius * 2.0));
    let right_handle_response = ui.interact(right_handle_rect, right_handle_id, egui::Sense::drag());
    
    if right_handle_response.dragged() {
        let delta_x = right_handle_response.drag_delta().x;
        
        // 使用累加器处理亚像素移动
        let acc_id = ui.id().with((index, "right_acc"));
        let mut acc = ui.data(|d| d.get_temp::<f32>(acc_id).unwrap_or(0.0));
        
        // 向右拖动 -> delta_x > 0 -> right_x 变大 -> right 应该变小
        acc += -delta_x / scale_x;
        
        let param_delta = acc.trunc() as i32;
        if param_delta != 0 {
            let new_right = (state.images[index].right as i32 + param_delta).max(0) as u32;
            state.images[index].right = new_right;
            acc -= param_delta as f32;
            needs_update = true;
        }
        
        ui.data_mut(|d| d.insert_temp(acc_id, acc));
    }
    
    let right_color = if right_handle_response.hovered() {
        egui::Color32::from_rgb(255, 200, 200)
    } else {
        egui::Color32::WHITE
    };
    painter.circle_filled(right_handle_pos, handle_radius, right_color);
    painter.circle_stroke(right_handle_pos, handle_radius, egui::Stroke::new(2.0, line_color));
    
    // 如果参数改变了,验证并更新
    if needs_update {
        state.images[index].validate_params();
        // 注意:不在这里生成预览,避免拖动时卡顿
        // 预览会在render_image_item末尾统一生成
    }
}

/// 绘制棋盘格背景
fn draw_checkerboard(painter: &egui::Painter, rect: egui::Rect) {
    let square_size = 10.0;
    let color1 = egui::Color32::from_gray(200);
    let color2 = egui::Color32::from_gray(220);
    
    let mut y = rect.top();
    let mut row = 0;
    
    while y < rect.bottom() {
        let mut x = rect.left();
        let mut col = 0;
        
        while x < rect.right() {
            let color = if (row + col) % 2 == 0 { color1 } else { color2 };
            
            let square_rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(square_size, square_size),
            );
            
            painter.rect_filled(square_rect.intersect(rect), 0.0, color);
            
            x += square_size;
            col += 1;
        }
        
        y += square_size;
        row += 1;
    }
}

/// 绘制切割线
fn draw_cutting_guides(
    painter: &egui::Painter,
    rect: egui::Rect,
    image: &crate::state::image_item::ImageItem,
    display_width: f32,
    display_height: f32,
) {
    let scale_x = display_width / image.width as f32;
    let scale_y = display_height / image.height as f32;
    
    // 计算切割线位置
    let top_y = rect.top() + image.top as f32 * scale_y;
    let bottom_y = rect.bottom() - image.bottom as f32 * scale_y;
    let left_x = rect.left() + image.left as f32 * scale_x;
    let right_x = rect.right() - image.right as f32 * scale_x;
    
    let line_color = egui::Color32::from_rgb(255, 100, 100);
    let line_width = 2.0;
    
    // 上切割线
    painter.line_segment(
        [egui::pos2(rect.left(), top_y), egui::pos2(rect.right(), top_y)],
        egui::Stroke::new(line_width, line_color),
    );
    
    // 下切割线
    painter.line_segment(
        [
            egui::pos2(rect.left(), bottom_y),
            egui::pos2(rect.right(), bottom_y),
        ],
        egui::Stroke::new(line_width, line_color),
    );
    
    // 左切割线
    painter.line_segment(
        [egui::pos2(left_x, rect.top()), egui::pos2(left_x, rect.bottom())],
        egui::Stroke::new(line_width, line_color),
    );
    
    // 右切割线
    painter.line_segment(
        [
            egui::pos2(right_x, rect.top()),
            egui::pos2(right_x, rect.bottom()),
        ],
        egui::Stroke::new(line_width, line_color),
    );
    
    // TODO: 绘制可拖动的手柄
}

/// 渲染参数控制
fn render_params_controls(ui: &mut egui::Ui, state: &mut AppState, index: usize) {
    ui.horizontal(|ui| {
        ui.label("上:");
        let mut top = state.images[index].top as i32;
        if ui.add(egui::DragValue::new(&mut top).range(0..=1000)).changed() {
            state.images[index].top = top.max(0) as u32;
            state.images[index].validate_params();
        }
        
        ui.label("下:");
        let mut bottom = state.images[index].bottom as i32;
        if ui.add(egui::DragValue::new(&mut bottom).range(0..=1000)).changed() {
            state.images[index].bottom = bottom.max(0) as u32;
            state.images[index].validate_params();
        }
        
        ui.label("左:");
        let mut left = state.images[index].left as i32;
        if ui.add(egui::DragValue::new(&mut left).range(0..=1000)).changed() {
            state.images[index].left = left.max(0) as u32;
            state.images[index].validate_params();
        }
        
        ui.label("右:");
        let mut right = state.images[index].right as i32;
        if ui.add(egui::DragValue::new(&mut right).range(0..=1000)).changed() {
            state.images[index].right = right.max(0) as u32;
            state.images[index].validate_params();
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("缩放:");
        let mut scale = state.images[index].scale;
        if ui
            .add(egui::DragValue::new(&mut scale).range(1.0..=200.0).suffix("%"))
            .changed()
        {
            state.images[index].scale = scale.clamp(1.0, 200.0);
            state.images[index].validate_params();
        }
        
        ui.add_space(10.0);
        
        // "复制上图参数"按钮
        let can_copy = index > 0;
        if ui
            .add_enabled(can_copy, egui::Button::new("📋 复制上图参数"))
            .clicked()
        {
            if index > 0 {
                let prev_params = state.images[index - 1].clone();
                state.images[index].copy_params_from(&prev_params);
            }
        }
    });
}

/// 生成预览图片
fn generate_preview(state: &mut AppState, index: usize) {
    use crate::image_processing::slicer;
    
    // 生成预览图片
    if let Ok(preview) = slicer::slice_and_stitch(&state.images[index]) {
        state.images[index].preview_image = Some(preview);
    }
}

/// 渲染拖放提示
fn render_drop_hint(ui: &mut egui::Ui) {
    egui::Frame::none()
        .fill(ui.style().visuals.faint_bg_color)
        .inner_margin(egui::Margin::same(15.0))
        .outer_margin(egui::Margin::symmetric(5.0, 10.0))
        .rounding(5.0)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.label(egui::RichText::new("💡 可继续拖入更多图片")
                    .color(ui.style().visuals.weak_text_color()));
                ui.label(egui::RichText::new("支持同时拖入多个PNG文件")
                    .size(11.0)
                    .color(ui.style().visuals.weak_text_color()));
                ui.add_space(5.0);
            });
        });
}

/// 将DynamicImage转换为egui的ColorImage
fn dynamic_image_to_color_image(img: &DynamicImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    let rgba = img.to_rgba8();
    let pixels: Vec<egui::Color32> = rgba
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    
    egui::ColorImage {
        size,
        pixels,
    }
}

