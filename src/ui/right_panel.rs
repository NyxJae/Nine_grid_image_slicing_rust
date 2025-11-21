use crate::state::app_state::AppState;

/// 渲染右侧预览区
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    egui::ScrollArea::vertical()
        .id_source("right_panel_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if let Some(selected_image) = state.get_selected_image() {
                // 显示预览信息
                render_preview_info(ui, selected_image);
                
                ui.separator();
                
                // 显示预览图片
                render_preview_image(ui, selected_image);
            } else {
                // 显示空状态
                render_empty_state(ui);
            }
        });
}

/// 渲染预览信息
fn render_preview_info(ui: &mut egui::Ui, image: &crate::state::image_item::ImageItem) {
    ui.group(|ui| {
        ui.label(format!("📄 预览: {}", image.file_name));
        ui.label(format!("📐 原始: {}×{}", image.width, image.height));
        
        // 计算切割后尺寸，需要匹配 slicer.rs 中的逻辑
        // 如果某方向的两个参数都为0，则该方向保留完整尺寸
        let sliced_width = if image.left == 0 && image.right == 0 {
            image.width  // 水平方向保留全宽
        } else {
            image.left + image.right  // 左角宽度 + 右角宽度
        };
        
        let sliced_height = if image.top == 0 && image.bottom == 0 {
            image.height  // 垂直方向保留全高
        } else {
            image.top + image.bottom  // 上角高度 + 下角高度
        };
        
        ui.label(format!("✂ 切割后: {}×{}", sliced_width, sliced_height));
        
        // 计算最终尺寸（应用缩放后）
        let final_width = (sliced_width as f32 * image.scale / 100.0) as u32;
        let final_height = (sliced_height as f32 * image.scale / 100.0) as u32;
        ui.label(format!(
            "🔍 缩放: {}% → 最终: {}×{}",
            image.scale, final_width, final_height
        ));
    });
}

/// 渲染预览图片
fn render_preview_image(ui: &mut egui::Ui, image: &crate::state::image_item::ImageItem) {
    let available_size = ui.available_size();
    
    // 确保有足够的空间
    if available_size.x < 10.0 || available_size.y < 10.0 {
        return;
    }
    
    if let Some(preview_img) = &image.preview_image {
        // 计算显示尺寸(保持宽高比)
        let img_width = preview_img.width() as f32;
        let img_height = preview_img.height() as f32;
        
        // 安全检查
        if img_width <= 0.0 || img_height <= 0.0 {
            ui.label("预览图片尺寸无效");
            return;
        }
        
        let aspect_ratio = img_width / img_height;
        
        if !aspect_ratio.is_finite() || aspect_ratio <= 0.0 {
            ui.label("预览图片宽高比无效");
            return;
        }
        
        let (display_width, display_height) = if available_size.x / available_size.y > aspect_ratio {
            let h = (available_size.y - 40.0).max(10.0);
            let w = h * aspect_ratio;
            (w.max(10.0), h)
        } else {
            let w = (available_size.x - 40.0).max(10.0);
            let h = w / aspect_ratio;
            (w, h.max(10.0))
        };
        
        // 转换为egui图片
        let color_image = rgba_to_color_image(preview_img);
        let texture = ui.ctx().load_texture(
            format!("preview_{}", image.file_name),
            color_image,
            egui::TextureOptions::default(),
        );
        
        // 绘制棋盘格背景
        let (response, painter) = ui.allocate_painter(
            egui::vec2(display_width, display_height),
            egui::Sense::hover(),
        );
        let rect = response.rect;
        draw_checkerboard(&painter, rect);
        
        // 绘制图片
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        // 预览图片尚未生成
        ui.vertical_centered(|ui| {
            ui.add_space(available_size.y / 3.0);
            ui.spinner();
            ui.label("生成预览中...");
        });
    }
}

/// 将RgbaImage转换为egui的ColorImage
fn rgba_to_color_image(img: &image::RgbaImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    let pixels: Vec<egui::Color32> = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    
    egui::ColorImage {
        size,
        pixels,
    }
}

/// 渲染空状态
fn render_empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        ui.heading("暂无预览内容");
        ui.add_space(10.0);
        ui.label("请先导入图片");
    });
}

/// 绘制棋盘格背景
fn draw_checkerboard(painter: &egui::Painter, rect: egui::Rect) {
    let square_size = 16.0;
    let color1 = egui::Color32::from_gray(80);
    let color2 = egui::Color32::from_gray(100);
    
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
