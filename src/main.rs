// Windows GUI应用配置 - 消除控制台黑框
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod image_processing;
mod state;
mod ui;
mod utils;

use app::NineGridSlicerApp;

fn main() -> eframe::Result<()> {
    // 配置日志(开发时有用)
    env_logger::init();

    // 配置窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])  // 初始尺寸
            .with_min_inner_size([800.0, 600.0])  // 最小尺寸
            .with_icon(load_icon()),  // 应用图标
        ..Default::default()
    };

    // 启动应用
    eframe::run_native(
        "九宫格切图工具",  // 窗口标题
        options,
        Box::new(|cc| {
            // 配置深色主题
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            // 配置字体(支持中文)
            setup_fonts(&cc.egui_ctx);
            
            Ok(Box::new(NineGridSlicerApp::new(cc)))
        }),
    )
}

/// 加载应用图标
/// 注意：Windows exe的图标是通过build.rs配置的，这里是窗口标题栏图标
/// 使用 include_bytes! 将图标嵌入到二进制文件中，确保release构建后也能正常显示
fn load_icon() -> egui::IconData {
    // 将图标文件嵌入到二进制中，避免路径问题
    match load_icon_from_embedded() {
        Ok(icon) => icon,
        Err(e) => {
            // 如果嵌入图标加载失败，尝试从文件系统加载
            eprintln!("嵌入图标加载失败: {}，尝试从文件系统加载", e);
            match load_icon_from_file("assets/icon.png") {
                Ok(icon) => icon,
                Err(_) => {
                    eprintln!("未找到图标文件 assets/icon.png，使用默认图标");
                    egui::IconData::default()
                }
            }
        }
    }
}

/// 从嵌入的字节数据加载图标
fn load_icon_from_embedded() -> Result<egui::IconData, Box<dyn std::error::Error>> {
    use image::GenericImageView;
    
    // 编译时将图标文件嵌入到二进制中
    let icon_bytes = include_bytes!("../assets/icon.png");
    
    let img = image::load_from_memory(icon_bytes)?;
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    
    Ok(egui::IconData {
        rgba,
        width,
        height,
    })
}

/// 从PNG文件加载图标（作为备用方案）
fn load_icon_from_file(path: &str) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    use image::GenericImageView;
    
    let img = image::open(path)?;
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    
    Ok(egui::IconData {
        rgba,
        width,
        height,
    })
}

/// 配置字体以支持中文
fn setup_fonts(ctx: &egui::Context) {
    use egui::{FontFamily, FontId, TextStyle};

    let mut fonts = egui::FontDefinitions::default();
    
    // 加载Windows系统自带的中文字体
    // 检查是否可以加载本地字体文件
    #[cfg(target_os = "windows")]
    {
        // Windows系统中文字体路径
        let font_paths = vec![
            "C:\\Windows\\Fonts\\msyh.ttc",      // 微软雅黑
            "C:\\Windows\\Fonts\\simhei.ttf",    // 黑体
            "C:\\Windows\\Fonts\\simsun.ttc",    // 宋体
        ];
        
        for font_path in font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "chinese".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                
                // 将中文字体添加到字体家族列表的最前面
                fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
                
                break; // 成功加载一个字体就退出
            }
        }
    }

    ctx.set_fonts(fonts);
    
    // 配置文本样式
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(10.0, FontFamily::Proportional)),
    ].into();

    ctx.set_style(style);
}
