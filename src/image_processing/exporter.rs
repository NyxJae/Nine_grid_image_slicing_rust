use anyhow::{Context, Result};
use std::path::PathBuf;
use crate::state::image_item::ImageItem;
use crate::image_processing::slicer;

/// 导出选项
pub enum ExportMode {
    /// 覆盖原图
    Overwrite,
    /// 添加后缀
    WithSuffix(String),
}

/// 导出单个图片
pub fn export_image(item: &ImageItem, mode: &ExportMode) -> Result<PathBuf> {
    // 生成处理后的图片
    let processed = slicer::slice_and_stitch(item)?;
    
    // 确定输出路径
    let output_path = match mode {
        ExportMode::Overwrite => item.file_path.clone(),
        ExportMode::WithSuffix(suffix) => {
            let parent = item.file_path.parent().unwrap_or(std::path::Path::new(""));
            let stem = item.file_path.file_stem().unwrap_or_default();
            let ext = item.file_path.extension().unwrap_or_default();
            
            parent.join(format!(
                "{}{}{}",
                stem.to_string_lossy(),
                suffix,
                if ext.is_empty() {
                    "".to_string()
                } else {
                    format!(".{}", ext.to_string_lossy())
                }
            ))
        }
    };
    
    // 保存图片
    processed
        .save(&output_path)
        .with_context(|| format!("无法保存图片到: {:?}", output_path))?;
    
    Ok(output_path)
}

/// 批量导出图片
pub fn export_images(
    items: &[ImageItem],
    mode: &ExportMode,
    progress_callback: Option<&dyn Fn(usize, usize)>,
) -> Result<Vec<(PathBuf, Result<PathBuf>)>> {
    let mut results = Vec::new();
    
    for (index, item) in items.iter().enumerate() {
        // 更新进度
        if let Some(callback) = progress_callback {
            callback(index + 1, items.len());
        }
        
        // 导出图片
        let result = export_image(item, mode);
        results.push((item.file_path.clone(), result));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use std::fs;
    use std::env;
    
    #[test]
    fn test_export_with_suffix() {
        // 创建测试图片
        let temp_dir = env::temp_dir();
        let test_path = temp_dir.join("test_export.png");
        
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            100,
            100,
            image::Rgba([255, 0, 0, 255]),
        ));
        img.save(&test_path).unwrap();
        
        let item = ImageItem::new(test_path.clone(), img);
        
        // 导出
        let result = export_image(&item, &ExportMode::WithSuffix("_cut".to_string()));
        assert!(result.is_ok());
        
        // 清理
        fs::remove_file(&test_path).ok();
        fs::remove_file(temp_dir.join("test_export_cut.png")).ok();
    }
}
