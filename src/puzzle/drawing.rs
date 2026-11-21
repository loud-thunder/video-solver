use image::{Rgba, RgbaImage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleImageType {
    VibrantMandala,
    CyberGrid,
    SunsetLandscape,
    BauhausGeometric,
}

impl SampleImageType {
    pub fn all() -> &'static [SampleImageType] {
        &[
            SampleImageType::VibrantMandala,
            SampleImageType::CyberGrid,
            SampleImageType::SunsetLandscape,
            SampleImageType::BauhausGeometric,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SampleImageType::VibrantMandala => "Vibrant Mandala",
            SampleImageType::CyberGrid => "Cyber Neon Grid",
            SampleImageType::SunsetLandscape => "Sunset Gradient Waves",
            SampleImageType::BauhausGeometric => "Bauhaus Geometric",
        }
    }

    pub fn generate(&self, width: u32, height: u32) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        let wf = width as f32;
        let hf = height as f32;

        match self {
            SampleImageType::VibrantMandala => {
                let cx = wf * 0.5;
                let cy = hf * 0.5;
                let max_r = cx.hypot(cy);

                for y in 0..height {
                    for x in 0..width {
                        let dx = x as f32 - cx;
                        let dy = y as f32 - cy;
                        let r = (dx * dx + dy * dy).sqrt();
                        let theta = dy.atan2(dx);

                        let petals = 8.0;
                        let wave = (theta * petals).sin() * 0.5 + 0.5;
                        let rings = (r / 20.0).sin() * 0.5 + 0.5;
                        let swirl = ((r / 30.0) + theta * 3.0).cos() * 0.5 + 0.5;

                        let norm_r = (r / max_r).clamp(0.0, 1.0);

                        let red = ((wave * 200.0 + (1.0 - norm_r) * 55.0 + swirl * 40.0) as u32).min(255) as u8;
                        let green = ((rings * 220.0 + norm_r * 80.0) as u32).min(255) as u8;
                        let blue = (((1.0 - wave) * 180.0 + swirl * 120.0) as u32).min(255) as u8;

                        img.put_pixel(x, y, Rgba([red, green, blue, 255]));
                    }
                }
            }
            SampleImageType::CyberGrid => {
                for y in 0..height {
                    let ny = y as f32 / hf;
                    for x in 0..width {
                        let nx = x as f32 / wf;

                        // Perspective horizon grid
                        let horizon = 0.45;
                        let is_ground = ny > horizon;

                        if is_ground {
                            let depth = (ny - horizon) / (1.0 - horizon);
                            let grid_x = (((nx - 0.5) / depth * 8.0).sin().abs() < 0.08) as i32;
                            let grid_y = (((1.0 / depth) * 4.0).sin().abs() < 0.12) as i32;

                            let base_r = (depth * 180.0) as u8;
                            let base_b = ((1.0 - depth * 0.5) * 220.0) as u8;

                            let line_boost = if grid_x > 0 || grid_y > 0 { 240 } else { 30 };
                            let r = (base_r / 4).saturating_add(line_boost / 2);
                            let g = line_boost / 3;
                            let b = base_b.max(line_boost);
                            img.put_pixel(x, y, Rgba([r, g, b, 255]));
                        } else {
                            // Synthwave sunset sky
                            let sky_t = ny / horizon;
                            let sun_dx = nx - 0.5;
                            let sun_dy = ny - (horizon - 0.05);
                            let in_sun = (sun_dx * sun_dx * 2.0 + sun_dy * sun_dy * 4.0) < 0.06;

                            let r = (255.0 * (0.2 + 0.8 * sky_t)) as u8;
                            let g = (120.0 * sky_t * sky_t) as u8;
                            let b = (255.0 * (1.0 - sky_t * 0.8)) as u8;

                            if in_sun && (y % 12 > 2) {
                                img.put_pixel(x, y, Rgba([255, 230, 80, 255]));
                            } else {
                                img.put_pixel(x, y, Rgba([r, g, b, 255]));
                            }
                        }
                    }
                }
            }
            SampleImageType::SunsetLandscape => {
                for y in 0..height {
                    let ny = y as f32 / hf;
                    for x in 0..width {
                        let nx = x as f32 / wf;

                        let mountain1 = 0.65 + (nx * 6.0).sin() * 0.12 + (nx * 14.0).cos() * 0.04;
                        let mountain2 = 0.78 + (nx * 9.0 + 1.2).sin() * 0.08 + (nx * 20.0).sin() * 0.02;

                        if ny > mountain2 {
                            let g_val = (30.0 + nx * 50.0) as u8;
                            img.put_pixel(x, y, Rgba([25, g_val, 40, 255]));
                        } else if ny > mountain1 {
                            let r_val = (90.0 + ny * 60.0) as u8;
                            img.put_pixel(x, y, Rgba([r_val, 40, 80, 255]));
                        } else {
                            // Warm gradient
                            let r = (255.0 * (1.0 - ny * 0.4)) as u8;
                            let g = (140.0 * (1.0 - ny * 0.7)) as u8;
                            let b = (50.0 + 180.0 * (1.0 - ny)) as u8;
                            img.put_pixel(x, y, Rgba([r, g, b, 255]));
                        }
                    }
                }
            }
            SampleImageType::BauhausGeometric => {
                // Primary colors, bold shapes and diagonals
                for y in 0..height {
                    let ny = y as f32 / hf;
                    for x in 0..width {
                        let nx = x as f32 / wf;

                        let in_circle = ((nx - 0.35).powi(2) + (ny - 0.4).powi(2)).sqrt() < 0.28;
                        let in_diag = (nx + ny * 0.8) > 0.9 && (nx + ny * 0.8) < 1.3;
                        let in_rect = nx > 0.6 && nx < 0.9 && ny > 0.2 && ny < 0.85;

                        if in_circle && in_rect {
                            img.put_pixel(x, y, Rgba([230, 60, 45, 255])); // Red
                        } else if in_circle {
                            img.put_pixel(x, y, Rgba([245, 195, 35, 255])); // Yellow
                        } else if in_rect {
                            img.put_pixel(x, y, Rgba([30, 95, 185, 255])); // Blue
                        } else if in_diag {
                            img.put_pixel(x, y, Rgba([20, 20, 25, 255])); // Black
                        } else {
                            // Cream canvas background
                            let grain = ((x * 17 + y * 31) % 7) as u8;
                            img.put_pixel(x, y, Rgba([240 - grain, 236 - grain, 224 - grain, 255]));
                        }
                    }
                }
            }
        }

        img
    }
}

/// Bilinear interpolation sampler with clamped boundary
#[inline(always)]
pub fn sample_bilinear(img: &RgbaImage, x: f32, y: f32) -> Rgba<u8> {
    let w = img.width() as f32;
    let h = img.height() as f32;

    let x = x.clamp(0.0, w - 1.001);
    let y = y.clamp(0.0, h - 1.001);

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(img.width() - 1);
    let y1 = (y0 + 1).min(img.height() - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let p00 = img.get_pixel(x0, y0).0;
    let p10 = img.get_pixel(x1, y0).0;
    let p01 = img.get_pixel(x0, y1).0;
    let p11 = img.get_pixel(x1, y1).0;

    let mut out = [0u8; 4];
    for c in 0..4 {
        let v0 = p00[c] as f32 * (1.0 - fx) + p10[c] as f32 * fx;
        let v1 = p01[c] as f32 * (1.0 - fx) + p11[c] as f32 * fx;
        let v = v0 * (1.0 - fy) + v1 * fy;
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }

    Rgba(out)
}

/// Fast blit of an image block onto another with bounds checking
#[allow(dead_code)]
pub fn blit_rect(
    src: &RgbaImage,
    src_rect: (u32, u32, u32, u32),
    dst: &mut RgbaImage,
    dst_x: i32,
    dst_y: i32,
) {
    let (sx, sy, sw, sh) = src_rect;
    let dw = dst.width() as i32;
    let dh = dst.height() as i32;

    for y in 0..sh {
        let dy = dst_y + y as i32;
        if dy < 0 || dy >= dh {
            continue;
        }
        for x in 0..sw {
            let dx = dst_x + x as i32;
            if dx < 0 || dx >= dw {
                continue;
            }
            let px = src.get_pixel(sx + x, sy + y);
            dst.put_pixel(dx as u32, dy as u32, *px);
        }
    }
}

/// Draw a simple anti-aliased or solid line
pub fn draw_line(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: Rgba<u8>,
    thickness: i32,
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut cx = x0;
    let mut cy = y0;

    let r = thickness / 2;
    let w = img.width() as i32;
    let h = img.height() as i32;

    loop {
        for oy in -r..=r {
            for ox in -r..=r {
                let px = cx + ox;
                let py = cy + oy;
                if px >= 0 && px < w && py >= 0 && py < h {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }

        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
}
