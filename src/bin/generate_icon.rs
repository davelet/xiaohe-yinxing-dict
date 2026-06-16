//! 从代码生成 macOS .iconset + .icns 图标文件
//!
//! 用法:
//!   cargo run --features generate-icon --bin generate_icon -- <output_dir>
//!
//! 默认输出到当前目录，生成 AppIcon.iconset/ 和 AppIcon.icns

#[cfg(not(feature = "generate-icon"))]
fn main() {
    eprintln!("This binary requires the 'generate-icon' feature.");
    eprintln!("Run: cargo run --features generate-icon --bin generate_icon");
    std::process::exit(1);
}

#[cfg(feature = "generate-icon")]
fn main() {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    let iconset_dir = format!("{out_dir}/AppIcon.iconset");
    std::fs::create_dir_all(&iconset_dir).expect("Failed to create iconset directory");

    // macOS iconset 需要的尺寸和文件名
    let sizes: [(u32, &str); 10] = [
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ];

    for (size, name) in &sizes {
        let rgba = generate_icon_rgba(*size);
        let path = format!("{iconset_dir}/{name}");
        write_png(&path, *size, &rgba).unwrap_or_else(|e| {
            eprintln!("Failed to write {path}: {e}");
            std::process::exit(1);
        });
    }

    // 使用 iconutil 生成 .icns
    let icns_path = format!("{out_dir}/AppIcon.icns");
    let status = std::process::Command::new("iconutil")
        .args(["-c", "icns", &iconset_dir, "-o", &icns_path])
        .status()
        .expect("Failed to run iconutil (only available on macOS)");

    if status.success() {
        println!("Generated {icns_path}");
    } else {
        eprintln!("iconutil failed; iconset is at {iconset_dir}");
        std::process::exit(1);
    }
}

/// 生成应用图标的 RGBA 数据（与 src/icon.rs 和运行时图标共享同一算法）
#[cfg(feature = "generate-icon")]
fn generate_icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let half = size as f32 / 2.0;
    let radius = half - 1.0;

    for y in 0..size {
        for x in 0..size {
            let i = (y * size + x) as usize * 4;
            let dx = x as f32 - half;
            let dy = y as f32 - half;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius {
                rgba[i + 3] = 0;
                continue;
            }

            // Background: teal (#0d9488) to indigo (#4338ca) gradient
            let t = dist / radius;
            rgba[i] = (13.0 + (67.0 - 13.0) * t) as u8;
            rgba[i + 1] = (148.0 - (148.0 - 56.0) * t) as u8;
            rgba[i + 2] = (136.0 - (136.0 - 202.0) * t) as u8;
            rgba[i + 3] = 255;

            let nx = dx / radius;
            let ny = dy / radius;

            let mut white = false;

            // Head
            let hx = 0.0;
            let hy = -0.45;
            if (nx - hx).powi(2) + (ny - hy).powi(2) < 0.035 {
                white = true;
            }

            // Beak
            if ny > -0.50 && ny < -0.40 && nx > 0.12 && nx < 0.30 {
                let beak_top = -0.50 + (nx - 0.12) * 0.2;
                let beak_bot = -0.40 - (nx - 0.12) * 0.2;
                if ny > beak_top && ny < beak_bot {
                    white = true;
                }
            }

            // Body
            if nx.abs() < 0.08 && ny > -0.35 && ny < 0.25 {
                white = true;
            }

            // Left wing
            if nx < -0.05 && ny > -0.45 && ny < 0.05 {
                let wing_upper = -0.45 + (nx + 0.7) * 0.6;
                let wing_lower = 0.05 - (nx + 0.7) * 0.4;
                if ny > wing_upper && ny < wing_lower && nx > -0.7 {
                    white = true;
                }
            }

            // Right wing
            if nx > 0.05 && ny > -0.45 && ny < 0.05 {
                let wing_upper = -0.45 + (0.7 - nx) * 0.6;
                let wing_lower = 0.05 - (0.7 - nx) * 0.4;
                if ny > wing_upper && ny < wing_lower && nx < 0.7 {
                    white = true;
                }
            }

            // Tail
            if ny > 0.20 && ny < 0.45 && nx.abs() < 0.15 {
                let tail_width = 0.15 * (1.0 - (ny - 0.20) / 0.25);
                if nx.abs() < tail_width {
                    white = true;
                }
            }

            if white {
                rgba[i] = 255;
                rgba[i + 1] = 255;
                rgba[i + 2] = 255;
            }
        }
    }

    rgba
}

#[cfg(feature = "generate-icon")]
fn write_png(path: &str, size: u32, rgba: &[u8]) -> std::io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(())
}
