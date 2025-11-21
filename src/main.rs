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
fn load_icon() -> egui::IconData {
    // TODO: 添加自定义图标
    // 暂时返回默认图标
    egui::IconData::default()
}

/// 配置字体以支持中文
fn setup_fonts(ctx: &egui::Context) {
    use egui::{FontFamily, FontId, TextStyle};

    let mut fonts = egui::FontDefinitions::default();
    
    // 加载Windows系统自带的中文字体
    // 这里使用一个简单的方法:让egui尝试使用系统字体
    // egui默认字体列表中,"Hack"和"Ubuntu-Light"不包含中文
    // 我们需要确保字体回退顺序正确
    
    // 对于Windows,我们可以尝试读取系统字体
    // 但更简单的方法是使用egui内置的字体,并添加中文字体数据
    
    // 暂时使用这个方法:不修改字体家族,让它使用默认
    // 但是我们需要确保使用的是包含更多字符的字体
    
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
