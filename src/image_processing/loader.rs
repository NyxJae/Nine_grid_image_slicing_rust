use anyhow::{Context, Result};
use std::path::PathBuf;
use crate::state::image_item::ImageItem;

/// 加载单个图片文件
pub fn load_image(path: PathBuf) -> Result<ImageItem> {
    // 验证文件扩展名
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if ext_str != "png" {
            anyhow::bail!("不支持的文件格式,请选择PNG图片");
        }
    } else {
        anyhow::bail!("文件没有扩展名");
    }
    
    // 加载图片
    let img = image::open(&path)
        .with_context(|| format!("无法打开图片文件: {:?}", path))?;
    
    // 验证是否为PNG格式
    if img.color().channel_count() < 3 {
        anyhow::bail!("图片格式不正确");
    }
    
    // 检查图片尺寸
    let (width, height) = (img.width(), img.height());
    
    if width == 0 || height == 0 {
        anyhow::bail!("图片尺寸无效");
    }
    
    if width > 8192 || height > 8192 {
        eprintln!(
            "警告: 图片尺寸过大({}×{}),可能影响性能",
            width, height
        );
    }
    
    if width < 32 || height < 32 {
        eprintln!(
            "警告: 图片尺寸过小({}×{}),切割效果可能不理想",
            width, height
        );
    }
    
    Ok(ImageItem::new(path, img))
}

/// 批量加载图片文件
pub fn load_images(paths: Vec<PathBuf>) -> Result<Vec<ImageItem>> {
    let mut images = Vec::new();
    let mut errors = Vec::new();
    
    for path in paths {
        match load_image(path.clone()) {
            Ok(image) => images.push(image),
            Err(e) => {
                errors.push(format!("{:?}: {}", path.file_name().unwrap_or_default(), e));
            }
        }
    }
    
    // 如果有错误,报告它们
    if !errors.is_empty() {
        let error_msg = errors.join("\n");
        if images.is_empty() {
            // 所有文件都失败了
            anyhow::bail!("所有图片加载失败:\n{}", error_msg);
        } else {
            // 部分文件失败
            eprintln!("部分图片加载失败:\n{}", error_msg);
        }
    }
    
    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_extension() {
        let path = PathBuf::from("test.jpg");
        let result = load_image(path);
        assert!(result.is_err());
    }
}
