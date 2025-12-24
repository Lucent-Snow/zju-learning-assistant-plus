use img_hash::{HashAlg, HasherConfig, ImageHash};
use log::{debug, info, warn};

/// 计算图片的pHash值
pub fn calculate_phash(image_path: &str) -> Result<ImageHash<Box<[u8]>>, Box<dyn std::error::Error>> {
    let img = image::open(image_path)?;
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let raw_pixels = rgb_img.into_raw();

    let img_hash_buffer = img_hash::image::ImageBuffer::<img_hash::image::Rgb<u8>, Vec<u8>>::from_raw(
        width,
        height,
        raw_pixels,
    ).ok_or("Failed to create ImageBuffer")?;

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(8, 8)
        .to_hasher();

    let hash = hasher.hash_image(&img_hash_buffer);
    Ok(hash)
}

/// 比较两个hash的相似度（汉明距离）
pub fn is_similar(hash1: &ImageHash<Box<[u8]>>, hash2: &ImageHash<Box<[u8]>>, threshold: u32) -> bool {
    let distance = hash1.dist(hash2);
    distance <= threshold
}

/// 对图片列表去重（只和前一张比较，保留最后一张）
pub fn deduplicate_images(
    image_paths: Vec<String>,
    threshold: u32,
) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    if image_paths.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let total = image_paths.len();
    info!("开始图片去重处理，总数：{}，阈值：{}", total, threshold);

    // 先计算所有图片的哈希值
    let mut hashes: Vec<Option<ImageHash<Box<[u8]>>>> = Vec::with_capacity(total);
    for path in &image_paths {
        match calculate_phash(path) {
            Ok(h) => hashes.push(Some(h)),
            Err(e) => {
                warn!("计算图片哈希失败 ({}): {}", path, e);
                hashes.push(None);
            }
        }
    }

    // 标记哪些图片需要保留（只和前一张比较，相似时保留后一张）
    let mut keep = vec![true; total];

    for i in 1..total {
        // 如果当前图片或前一张图片哈希计算失败，都保留
        if let (Some(ref curr_hash), Some(ref prev_hash)) = (&hashes[i], &hashes[i - 1]) {
            if is_similar(curr_hash, prev_hash, threshold) {
                // 相似时，移除前一张，保留当前这张（最后一张）
                keep[i - 1] = false;
                debug!("发现重复图片: {} 与 {} 相似，移除前者", &image_paths[i - 1], &image_paths[i]);
            }
        }
    }

    // 根据标记分离结果
    let mut result = Vec::new();
    let mut skipped = Vec::new();

    for (i, path) in image_paths.into_iter().enumerate() {
        if keep[i] {
            result.push(path);
        } else {
            skipped.push(path);
        }
    }

    info!("去重完成：保留 {} 张，跳过 {} 张", result.len(), skipped.len());
    Ok((result, skipped))
}
