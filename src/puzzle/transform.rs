use image::RgbaImage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use super::{drawing, PuzzleKind, PuzzleSimulation};

// ============================================================================
// 13. SWIRL VORTEX UNWIND
// ============================================================================

pub struct SwirlVortexPuzzle {
    original: RgbaImage,
    centers: Vec<(f32, f32, f32, f32)>, // (cx, cy, radius, max_angle)
}

impl SwirlVortexPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let w = original.width() as f32;
        let h = original.height() as f32;
        let num_vortices = (complexity as usize / 3 + 1).clamp(1, 4);

        let mut centers = Vec::with_capacity(num_vortices);
        // Main center vortex
        let max_r = (w.hypot(h) * 0.6).max(100.0);
        let max_ang = rng.gen_range(4.0..8.0) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
        centers.push((w * 0.5, h * 0.5, max_r, max_ang));

        for _ in 1..num_vortices {
            let cx = rng.gen_range(w * 0.2..w * 0.8);
            let cy = rng.gen_range(h * 0.2..h * 0.8);
            let r = rng.gen_range(w * 0.2..w * 0.45);
            let ang = rng.gen_range(3.0..6.0) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            centers.push((cx, cy, r, ang));
        }

        Self {
            original: original.clone(),
            centers,
        }
    }
}

impl PuzzleSimulation for SwirlVortexPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::SwirlVortex
    }

    fn total_steps(&self) -> usize {
        100
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Swirl Vortex: {:.0}% unwound", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let ease = 1.0 - progress;

        let raw_pixels: Vec<u8> = (0..h)
            .into_par_iter()
            .flat_map(|y| {
                let mut row = Vec::with_capacity(w as usize * 4);
                let py = y as f32;
                for x in 0..w {
                    let mut cur_x = x as f32;
                    let mut cur_y = py;

                    for &(cx, cy, radius, max_ang) in &self.centers {
                        let dx = cur_x - cx;
                        let dy = cur_y - cy;
                        let r = (dx * dx + dy * dy).sqrt();

                        if r < radius {
                            let factor = 1.0 - (r / radius);
                            let theta = factor * factor * max_ang * ease;
                            let cos_t = theta.cos();
                            let sin_t = theta.sin();
                            cur_x = cx + dx * cos_t - dy * sin_t;
                            cur_y = cy + dx * sin_t + dy * cos_t;
                        }
                    }

                    let color = drawing::sample_bilinear(&self.original, cur_x, cur_y);
                    row.extend_from_slice(&color.0);
                }
                row
            })
            .collect();

        if let Some(buf) = RgbaImage::from_raw(w, h, raw_pixels) {
            *out = buf;
        }
    }
}

// ============================================================================
// 14. SINE WAVE PHASE ALIGNMENT
// ============================================================================

pub struct WavePhaseAlignPuzzle {
    original: RgbaImage,
    harmonics: Vec<(f32, f32, f32, bool)>, // (frequency, amplitude, phase, is_x)
}

impl WavePhaseAlignPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let count = (complexity as usize * 2 + 3).clamp(4, 15);

        let mut harmonics = Vec::with_capacity(count);
        for i in 0..count {
            let is_x = i % 2 == 0;
            let freq = rng.gen_range(0.01..0.08);
            let amp = rng.gen_range(15.0..45.0);
            let phase = rng.gen_range(0.0..std::f32::consts::TAU);
            harmonics.push((freq, amp, phase, is_x));
        }

        Self {
            original: original.clone(),
            harmonics,
        }
    }
}

impl PuzzleSimulation for WavePhaseAlignPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::WavePhaseAlign
    }

    fn total_steps(&self) -> usize {
        self.harmonics.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let aligned = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Phase Alignment: {} / {} waves stabilized", aligned, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let total = self.total_steps();
        let target_f = progress * total as f32;

        let raw_pixels: Vec<u8> = (0..h)
            .into_par_iter()
            .flat_map(|y| {
                let mut row = Vec::with_capacity(w as usize * 4);
                let py = y as f32;
                for x in 0..w {
                    let px = x as f32;
                    let mut off_x = 0.0;
                    let mut off_y = 0.0;

                    for (idx, &(freq, amp, phase, is_x)) in self.harmonics.iter().enumerate() {
                        let harm_p = (target_f - idx as f32).clamp(0.0, 1.0);
                        let ease = 1.0 - (harm_p * harm_p * (3.0 - 2.0 * harm_p));

                        if is_x {
                            off_x += (py * freq + phase).sin() * amp * ease;
                        } else {
                            off_y += (px * freq + phase).sin() * amp * ease;
                        }
                    }

                    let sx = px + off_x;
                    let sy = py + off_y;
                    let color = drawing::sample_bilinear(&self.original, sx, sy);
                    row.extend_from_slice(&color.0);
                }
                row
            })
            .collect();

        if let Some(buf) = RgbaImage::from_raw(w, h, raw_pixels) {
            *out = buf;
        }
    }
}

// ============================================================================
// 15. KALEIDOSCOPE MIRROR UNFOLD
// ============================================================================

pub struct KaleidoscopePuzzle {
    original: RgbaImage,
    folds: usize,
}

impl KaleidoscopePuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let folds = (complexity as usize / 2 + 3).clamp(3, 8) * 2;
        Self {
            original: original.clone(),
            folds,
        }
    }
}

impl PuzzleSimulation for KaleidoscopePuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::KaleidoscopeUnfold
    }

    fn total_steps(&self) -> usize {
        self.folds
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Kaleidoscope: {:.0}% unfolded", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let sector_angle = std::f32::consts::TAU / self.folds as f32;
        let blend = progress * progress * (3.0 - 2.0 * progress);

        let raw_pixels: Vec<u8> = (0..h)
            .into_par_iter()
            .flat_map(|y| {
                let mut row = Vec::with_capacity(w as usize * 4);
                let dy = y as f32 - cy;
                for x in 0..w {
                    let dx = x as f32 - cx;
                    let r = (dx * dx + dy * dy).sqrt();
                    let theta = dy.atan2(dx).rem_euclid(std::f32::consts::TAU);

                    // Mirror within sector
                    let sector = (theta / sector_angle).floor();
                    let mut local_theta = theta - sector * sector_angle;
                    if sector as i32 % 2 == 1 {
                        local_theta = sector_angle - local_theta;
                    }

                    let k_x = cx + r * local_theta.cos();
                    let k_y = cy + r * local_theta.sin();

                    let k_color = drawing::sample_bilinear(&self.original, k_x, k_y);
                    let orig_color = *self.original.get_pixel(x, y);

                    let mut blended = [0u8; 4];
                    for c in 0..3 {
                        let v = k_color.0[c] as f32 * (1.0 - blend) + orig_color.0[c] as f32 * blend;
                        blended[c] = v.round() as u8;
                    }
                    blended[3] = 255;

                    row.extend_from_slice(&blended);
                }
                row
            })
            .collect();

        if let Some(buf) = RgbaImage::from_raw(w, h, raw_pixels) {
            *out = buf;
        }
    }
}

// ============================================================================
// 16. CHROMATIC & SCANLINE GLITCH
// ============================================================================

pub struct ChromaticGlitchPuzzle {
    original: RgbaImage,
    scanline_shifts: Vec<f32>,
    max_aberration: f32,
}

impl ChromaticGlitchPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let h = original.height() as usize;

        let mut scanline_shifts = Vec::with_capacity(h);
        for _ in 0..h {
            let shift = if rng.gen_bool(0.35) {
                rng.gen_range(-35.0..35.0)
            } else {
                0.0
            };
            scanline_shifts.push(shift);
        }

        let max_aberration = (complexity as f32 * 4.0 + 15.0).clamp(15.0, 50.0);

        Self {
            original: original.clone(),
            scanline_shifts,
            max_aberration,
        }
    }
}

impl PuzzleSimulation for ChromaticGlitchPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ChromaticGlitch
    }

    fn total_steps(&self) -> usize {
        60
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Chromatic Glitch: {:.0}% aligned", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let glitch_factor = 1.0 - (progress * progress * (3.0 - 2.0 * progress));
        let aber = self.max_aberration * glitch_factor;

        let raw_pixels: Vec<u8> = (0..h)
            .into_par_iter()
            .flat_map(|y| {
                let mut row = Vec::with_capacity(w as usize * 4);
                let line_shift = self.scanline_shifts[y as usize] * glitch_factor;
                let py = y as f32;

                for x in 0..w {
                    let px = x as f32 + line_shift;

                    // R shifted left, B shifted right
                    let r_color = drawing::sample_bilinear(&self.original, px - aber, py);
                    let g_color = drawing::sample_bilinear(&self.original, px, py);
                    let b_color = drawing::sample_bilinear(&self.original, px + aber, py);

                    row.push(r_color.0[0]);
                    row.push(g_color.0[1]);
                    row.push(b_color.0[2]);
                    row.push(255);
                }
                row
            })
            .collect();

        if let Some(buf) = RgbaImage::from_raw(w, h, raw_pixels) {
            *out = buf;
        }
    }
}

// ============================================================================
// 17. MOSAIC BLOCK COALESCE
// ============================================================================

pub struct MosaicCoalescePuzzle {
    original: RgbaImage,
    initial_block_size: u32,
}

impl MosaicCoalescePuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let initial_block_size = (complexity * 6 + 24).clamp(24, 72);
        Self {
            original: original.clone(),
            initial_block_size,
        }
    }
}

impl PuzzleSimulation for MosaicCoalescePuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::MosaicCoalesce
    }

    fn total_steps(&self) -> usize {
        50
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Mosaic Coalesce: {:.0}% resolution", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let ease = 1.0 - (progress * progress * (3.0 - 2.0 * progress));
        let cur_block = (self.initial_block_size as f32 * ease).round() as u32;

        if cur_block <= 1 {
            *out = self.original.clone();
            return;
        }

        let raw_pixels: Vec<u8> = (0..h)
            .into_par_iter()
            .flat_map(|y| {
                let mut row = Vec::with_capacity(w as usize * 4);
                let block_y0 = (y / cur_block) * cur_block;
                let sample_y = (block_y0 + cur_block / 2).min(h - 1);

                for x in 0..w {
                    let block_x0 = (x / cur_block) * cur_block;
                    let sample_x = (block_x0 + cur_block / 2).min(w - 1);

                    let c = self.original.get_pixel(sample_x, sample_y);
                    row.extend_from_slice(&c.0);
                }
                row
            })
            .collect();

        if let Some(buf) = RgbaImage::from_raw(w, h, raw_pixels) {
            *out = buf;
        }
    }
}
