use anyhow::Result;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use crate::state::image_item::ImageItem;

/// 切割并拼接图片
/// 根据切割参数提取四个角,然后拼接成新图片
/// 如果所有切割参数都为0,则返回缩放后的完整图片(等比缩放模式)
pub fn slice_and_stitch(item: &ImageItem) -> Result<RgbaImage> {
    let img = &item.original_image;
    
    // 检查是否为等比缩放模式(所有切割参数都为0)
    if item.top == 0 && item.bottom == 0 && item.left == 0 && item.right == 0 {
        return scale_full_image(img, item.scale);
    }
    
    // 确定实际使用的切割参数
    // 如果某方向上的两个参数都为0,则视为该方向不切割(保留完整尺寸)
    let (use_left, use_right) = if item.left == 0 && item.right == 0 {
        (img.width(), 0) // 水平方向保留全宽
    } else {
        (item.left, item.right)
    };
    
    let (use_top, use_bottom) = if item.top == 0 && item.bottom == 0 {
        (img.height(), 0) // 垂直方向保留全高
    } else {
        (item.top, item.bottom)
    };
    
    // 提取四个角
    let top_left = extract_corner(img, 0, 0, use_left, use_top)?;
    let top_right = extract_corner(
        img,
        img.width().saturating_sub(use_right),
        0,
        use_right,
        use_top,
    )?;
    let bottom_left = extract_corner(
        img,
        0,
        img.height().saturating_sub(use_bottom),
        use_left,
        use_bottom,
    )?;
    let bottom_right = extract_corner(
        img,
        img.width().saturating_sub(use_right),
        img.height().saturating_sub(use_bottom),
        use_right,
        use_bottom,
    )?;
    
    // 拼接四个角
    let stitched = stitch_corners(
        &top_left,
        &top_right,
        &bottom_left,
        &bottom_right,
    )?;
    
    // 应用缩放
    let final_image = scale_image(&stitched, item.scale)?;
    
    Ok(final_image)
}

/// 提取图片的一个角
fn extract_corner(
    img: &DynamicImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<RgbaImage> {
    if width == 0 || height == 0 {
        // 如果宽度或高度为0,返回0x0的图片
        return Ok(ImageBuffer::new(0, 0));
    }
    
    // 确保不超出边界
    let x = x.min(img.width());
    let y = y.min(img.height());
    let width = width.min(img.width() - x);
    let height = height.min(img.height() - y);
    
    let cropped = img.crop_imm(x, y, width, height);
    Ok(cropped.to_rgba8())
}

/// 拼接四个角
fn stitch_corners(
    top_left: &RgbaImage,
    top_right: &RgbaImage,
    bottom_left: &RgbaImage,
    bottom_right: &RgbaImage,
) -> Result<RgbaImage> {
    // 计算拼接后的总尺寸
    // 宽度 = 左侧宽度 + 右侧宽度 (取上和下的最大值)
    // 高度 = 上侧高度 + 下侧高度 (取左和右的最大值)
    let left_width = top_left.width().max(bottom_left.width());
    let right_width = top_right.width().max(bottom_right.width());
    let top_height = top_left.height().max(top_right.height());
    let bottom_height = bottom_left.height().max(bottom_right.height());
    
    let width = left_width + right_width;
    let height = top_height + bottom_height;
    
    if width == 0 || height == 0 {
        return Ok(ImageBuffer::from_pixel(1, 1, Rgba([0, 0, 0, 0])));
    }
    
    let mut result = ImageBuffer::new(width, height);
    
    // 复制左上角 (0, 0)
    for (x, y, pixel) in top_left.enumerate_pixels() {
        if x < left_width && y < top_height {
            result.put_pixel(x, y, *pixel);
        }
    }
    
    // 复制右上角  (left_width, 0)
    for (x, y, pixel) in top_right.enumerate_pixels() {
        if x < right_width && y < top_height {
            result.put_pixel(x + left_width, y, *pixel);
        }
    }
    
    // 复制左下角 (0, top_height)
    for (x, y, pixel) in bottom_left.enumerate_pixels() {
        if x < left_width && y < bottom_height {
            result.put_pixel(x, y + top_height, *pixel);
        }
    }
    
    // 复制右下角 (left_width, top_height)
    for (x, y, pixel) in bottom_right.enumerate_pixels() {
        if x < right_width && y < bottom_height {
            result.put_pixel(x + left_width, y + top_height, *pixel);
        }
    }
    
    Ok(result)
}

/// 缩放图片
fn scale_image(img: &RgbaImage, scale_percent: f32) -> Result<RgbaImage> {
    if (scale_percent - 100.0).abs() < 0.01 {
        // 不需要缩放
        return Ok(img.clone());
    }
    
    let scale_factor = scale_percent / 100.0;
    let new_width = ((img.width() as f32 * scale_factor) as u32).max(1);
    let new_height = ((img.height() as f32 * scale_factor) as u32).max(1);
    
    let scaled = image::imageops::resize(
        img,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,  // 高质量缩放
    );
    
    Ok(scaled)
}

/// 等比缩放完整图片(当所有切割参数都为0时)
fn scale_full_image(img: &DynamicImage, scale_percent: f32) -> Result<RgbaImage> {
    let rgba_img = img.to_rgba8();
    scale_image(&rgba_img, scale_percent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;
    
    #[test]
    fn test_extract_corner() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            100,
            100,
            Rgba([255, 0, 0, 255]),
        ));
        
        let corner = extract_corner(&img, 0, 0, 10, 10).unwrap();
        assert_eq!(corner.width(), 10);
        assert_eq!(corner.height(), 10);
    }
    
    #[test]
    fn test_scale_image() {
        let img = RgbaImage::from_pixel(100, 100, Rgba([255, 0, 0, 255]));
        
        // 缩小50%
        let scaled = scale_image(&img, 50.0).unwrap();
        assert_eq!(scaled.width(), 50);
        assert_eq!(scaled.height(), 50);
        
        // 放大200%
        let scaled = scale_image(&img, 200.0).unwrap();
        assert_eq!(scaled.width(), 200);
        assert_eq!(scaled.height(), 200);
        
        // 不缩放
        let scaled = scale_image(&img, 100.0).unwrap();
        assert_eq!(scaled.width(), 100);
        assert_eq!(scaled.height(), 100);
    }
    
    #[test]
    fn test_scale_full_image() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            100,
            100,
            Rgba([255, 0, 0, 255]),
        ));
        
        let scaled = scale_full_image(&img, 50.0).unwrap();
        assert_eq!(scaled.width(), 50);
        assert_eq!(scaled.height(), 50);
    }
}
