/// 生成应用图标的 RGBA 数据（参数化尺寸，与运行时图标共享同一算法）
pub fn generate_icon_rgba(size: u32) -> Vec<u8> {
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
                rgba[i + 3] = 0; // transparent outside circle
                continue;
            }

            // Background: teal (#0d9488) to indigo (#4338ca) gradient
            let t = dist / radius;
            rgba[i] = (13.0 + (67.0 - 13.0) * t) as u8;
            rgba[i + 1] = (148.0 - (148.0 - 56.0) * t) as u8;
            rgba[i + 2] = (136.0 - (136.0 - 202.0) * t) as u8;
            rgba[i + 3] = 255;

            let nx = dx / radius; // -1..1
            let ny = dy / radius; // -1..1

            // Draw white "crane in flight" silhouette
            let mut white = false;

            // Head: small circle at top
            let hx = 0.0;
            let hy = -0.45;
            if (nx - hx).powi(2) + (ny - hy).powi(2) < 0.035 {
                white = true;
            }

            // Beak: small triangle pointing right from head
            if ny > -0.50 && ny < -0.40 && nx > 0.12 && nx < 0.30 {
                let beak_top = -0.50 + (nx - 0.12) * 0.2;
                let beak_bot = -0.40 - (nx - 0.12) * 0.2;
                if ny > beak_top && ny < beak_bot {
                    white = true;
                }
            }

            // Body: thin vertical oval
            if nx.abs() < 0.08 && ny > -0.35 && ny < 0.25 {
                white = true;
            }

            // Left wing: triangular shape spreading up-left
            if nx < -0.05 && ny > -0.45 && ny < 0.05 {
                let wing_upper = -0.45 + (nx + 0.7) * 0.6;
                let wing_lower = 0.05 - (nx + 0.7) * 0.4;
                if ny > wing_upper && ny < wing_lower && nx > -0.7 {
                    white = true;
                }
            }

            // Right wing: triangular shape spreading up-right
            if nx > 0.05 && ny > -0.45 && ny < 0.05 {
                let wing_upper = -0.45 + (0.7 - nx) * 0.6;
                let wing_lower = 0.05 - (0.7 - nx) * 0.4;
                if ny > wing_upper && ny < wing_lower && nx < 0.7 {
                    white = true;
                }
            }

            // Tail feathers: small fan at bottom
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
