use image::{Rgba, RgbaImage};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{drawing, PuzzleKind, PuzzleSimulation};

// ============================================================================
// 23. FALLING SAND GRAIN FLOW
// ============================================================================

pub struct SandFlowPuzzle {
    original: RgbaImage,
    columns: usize,
    grain_offsets: Vec<f32>,
    grain_drift: Vec<f32>,
}

impl SandFlowPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let columns = (complexity as usize * 4 + 16).clamp(16, 120);
        let mut grain_offsets = Vec::with_capacity(columns);
        let mut grain_drift = Vec::with_capacity(columns);

        for _ in 0..columns {
            grain_offsets.push(rng.gen_range(0.0..0.4));
            grain_drift.push(rng.gen_range(-0.15..0.15));
        }

        Self {
            original: original.clone(),
            columns,
            grain_offsets,
            grain_drift,
        }
    }
}

impl PuzzleSimulation for SandFlowPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::SandFlow
    }
    fn total_steps(&self) -> usize {
        self.columns * 10
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Sand Flow: {:.1}% settled in hopper", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();
        let wf = w as f32;
        let hf = h as f32;
        let col_w = wf / self.columns as f32;

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        // Background: warm hourglass dark slate
        for y in 0..h {
            for x in 0..w {
                let dark_sand = 16 + ((x + y * 3) % 7) as u8;
                out.put_pixel(x, y, Rgba([dark_sand + 4, dark_sand + 2, dark_sand, 255]));
            }
        }

        for (c, &offset) in self.grain_offsets.iter().enumerate() {
            let col_x0 = (c as f32 * col_w) as u32;
            let col_x1 = (((c + 1) as f32 * col_w) as u32).min(w);
            let drift = self.grain_drift[c];

            // Local progress for this column
            let local_p = ((progress - offset) / (1.0 - offset)).clamp(0.0, 1.0);
            let settle_y = (hf * (1.0 - local_p)) as u32;

            for x in col_x0..col_x1 {
                // Settled grains at the bottom
                for y in settle_y..h {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                }

                // Falling grains in flight above settle line
                if local_p > 0.001 && local_p < 0.999 {
                    let stream_top = (settle_y as f32 - hf * 0.45 * (1.0 - local_p)).max(0.0) as u32;
                    let drift_dx = (drift * hf * (1.0 - local_p)) as i32;

                    for y in stream_top..settle_y {
                        if (y + x * 3) % 4 == 0 {
                            let src_x = ((x as i32 + drift_dx).clamp(0, w as i32 - 1)) as u32;
                            let mut px = *self.original.get_pixel(src_x, y);
                            // Highlight falling grain glimmer
                            px.0[0] = px.0[0].saturating_add(30);
                            px.0[1] = px.0[1].saturating_add(25);
                            out.put_pixel(x, y, px);
                        }
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 24. VISCOUS FLUID DROPLETS
// ============================================================================

pub struct FluidDropletsPuzzle {
    original: RgbaImage,
    droplets: Vec<(f32, f32, f32, f32)>, // (target_x, target_y, splash_radius, start_delay)
}

impl FluidDropletsPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let count = (complexity * 12 + 40).clamp(40, 240) as usize;
        let w = original.width() as f32;
        let h = original.height() as f32;
        let max_r = (w.hypot(h) / (count as f32).sqrt()) * 1.5;

        let mut droplets = Vec::with_capacity(count);
        for _ in 0..count {
            let tx = rng.gen_range(0.0..w);
            let ty = rng.gen_range(0.0..h);
            let r = rng.gen_range(max_r * 0.6..max_r * 1.6);
            let delay = rng.gen_range(0.0..0.7);
            droplets.push((tx, ty, r, delay));
        }

        Self {
            original: original.clone(),
            droplets,
        }
    }
}

impl PuzzleSimulation for FluidDropletsPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::FluidDroplets
    }
    fn total_steps(&self) -> usize {
        self.droplets.len()
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Fluid Droplets: {:.1}% surface coalesced", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        // Viscous dark oil slick background
        for y in 0..h {
            for x in 0..w {
                let ripple = (((x * 5 + y * 7) as f32 * 0.05).sin() * 8.0) as i16;
                let c = (20 + ripple).clamp(10, 40) as u8;
                out.put_pixel(x, y, Rgba([c, c + 3, c + 8, 255]));
            }
        }

        for (tx, ty, max_r, delay) in &self.droplets {
            let p = ((progress - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if p <= 0.001 {
                continue;
            }

            let ease = p * p * (3.0 - 2.0 * p);
            let cur_r = max_r * ease;
            let r_sq = cur_r * cur_r;

            let min_x = (tx - cur_r).floor().max(0.0) as u32;
            let max_x = (tx + cur_r).ceil().min(w as f32 - 1.0) as u32;
            let min_y = (ty - cur_r).floor().max(0.0) as u32;
            let max_y = (ty + cur_r).ceil().min(h as f32 - 1.0) as u32;

            for y in min_y..=max_y {
                let dy = y as f32 - ty;
                for x in min_x..=max_x {
                    let dx = x as f32 - tx;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= r_sq {
                        let mut px = *self.original.get_pixel(x, y);
                        // Droplet surface tension highlight rim
                        let edge_ratio = dist_sq / r_sq;
                        if edge_ratio > 0.85 && p < 0.95 {
                            px.0[0] = px.0[0].saturating_add(40);
                            px.0[1] = px.0[1].saturating_add(40);
                            px.0[2] = px.0[2].saturating_add(60);
                        }
                        out.put_pixel(x, y, px);
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 25. GRAVITATIONAL LENSING VORTEX
// ============================================================================

pub struct GravityLensPuzzle {
    original: RgbaImage,
    black_hole_center: (f32, f32),
    mass: f32,
}

impl GravityLensPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let w = original.width() as f32;
        let h = original.height() as f32;
        let cx = w * rng.gen_range(0.35..0.65);
        let cy = h * rng.gen_range(0.35..0.65);
        let mass = (complexity as f32 * 4.0 + 30.0).clamp(30.0, 110.0);

        Self {
            original: original.clone(),
            black_hole_center: (cx, cy),
            mass,
        }
    }
}

impl PuzzleSimulation for GravityLensPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::GravityLens
    }
    fn total_steps(&self) -> usize {
        100
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Gravity Lens: Deflection curvature at {:.1}%", (1.0 - progress) * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let strength = (1.0 - progress) * (1.0 - progress);
        let (cx, cy) = self.black_hole_center;
        let einstein_radius = self.mass * strength;

        for y in 0..h {
            let dy = y as f32 - cy;
            for x in 0..w {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();

                if r < 1.0 {
                    out.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                    continue;
                }

                // Einstein ring deflection equation: delta_theta = R_e^2 / r
                let deflection = (einstein_radius * einstein_radius) / r;
                let angle = dy.atan2(dx) + strength * 2.5 * (-r * 0.02).exp();
                let src_r = (r - deflection).max(0.0);

                let src_x = cx + angle.cos() * src_r;
                let src_y = cy + angle.sin() * src_r;

                let mut color = drawing::sample_bilinear(&self.original, src_x, src_y);

                // Photon sphere glow
                if (r - einstein_radius).abs() < 3.0 && strength > 0.05 {
                    color.0[0] = color.0[0].saturating_add(80);
                    color.0[1] = color.0[1].saturating_add(60);
                    color.0[2] = color.0[2].saturating_add(100);
                }

                out.put_pixel(x, y, color);
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 26. MAGNETIC FERROFLUID SPIKES
// ============================================================================

pub struct FerrofluidPuzzle {
    original: RgbaImage,
    poles: Vec<(f32, f32, f32)>, // (x, y, pole_strength)
}

impl FerrofluidPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let pole_count = (complexity / 3 + 3).clamp(3, 8) as usize;
        let w = original.width() as f32;
        let h = original.height() as f32;

        let mut poles = Vec::with_capacity(pole_count);
        for _ in 0..pole_count {
            let px = rng.gen_range(0.15..0.85) * w;
            let py = rng.gen_range(0.15..0.85) * h;
            let s = rng.gen_range(0.7..1.3);
            poles.push((px, py, s));
        }

        Self {
            original: original.clone(),
            poles,
        }
    }
}

impl PuzzleSimulation for FerrofluidPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Ferrofluid
    }
    fn total_steps(&self) -> usize {
        self.poles.len() * 20
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Ferrofluid: Field relaxation {:.1}%", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let field_t = 1.0 - progress;

        for y in 0..h {
            let fy = y as f32;
            for x in 0..w {
                let fx = x as f32;

                let mut bx = 0.0f32;
                let mut by = 0.0f32;

                for (px, py, s) in &self.poles {
                    let dx = fx - px;
                    let dy = fy - py;
                    let dist = (dx * dx + dy * dy + 100.0).sqrt();
                    let field = (*s * 1000.0) / (dist * dist);
                    bx += dx * field;
                    by += dy * field;
                }

                let spike_freq = 0.15;
                let spike_wave = ((fx * spike_freq).sin() * (fy * spike_freq).cos()) * 12.0;
                let displacement_x = (bx * 0.04 + spike_wave) * field_t;
                let displacement_y = (by * 0.04 + spike_wave) * field_t;

                let src_x = fx - displacement_x;
                let src_y = fy - displacement_y;

                let mut color = drawing::sample_bilinear(&self.original, src_x, src_y);

                // Magnetic spike crest highlights
                if field_t > 0.1 && (displacement_x.abs() + displacement_y.abs()) > 6.0 {
                    color.0[0] = color.0[0].saturating_add(30);
                    color.0[1] = color.0[1].saturating_add(35);
                    color.0[2] = color.0[2].saturating_add(45);
                }

                out.put_pixel(x, y, color);
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 27. FABRIC & CLOTH UNFURL
// ============================================================================

pub struct ClothUnfurlPuzzle {
    original: RgbaImage,
    fold_frequency: f32,
    crumple_amp: f32,
}

impl ClothUnfurlPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let fold_frequency = (complexity as f32 * 0.008 + 0.03).clamp(0.02, 0.1);
        let crumple_amp = (complexity as f32 * 3.0 + 20.0).clamp(20.0, 80.0) * rng.gen_range(0.9..1.1);

        Self {
            original: original.clone(),
            fold_frequency,
            crumple_amp,
        }
    }
}

impl PuzzleSimulation for ClothUnfurlPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ClothUnfurl
    }
    fn total_steps(&self) -> usize {
        120
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Cloth Unfurl: {:.1}% taut & ironed", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let t = 1.0 - progress;
        let amp = self.crumple_amp * t * t;
        let freq = self.fold_frequency;

        for y in 0..h {
            let fy = y as f32;
            for x in 0..w {
                let fx = x as f32;

                let wave1 = (fx * freq + fy * freq * 0.7).sin();
                let wave2 = (fy * freq * 1.3 - fx * freq * 0.5).cos();
                let z = (wave1 + wave2) * amp;

                // 3D cloth normal approximation for fold shading
                let dx = ((fx + 1.0) * freq + fy * freq * 0.7).sin() * amp - z;
                let dy = ((fy + 1.0) * freq * 1.3 - fx * freq * 0.5).cos() * amp - z;

                let shade = 1.0 + (dx * 0.03 - dy * 0.04) * t;

                let src_x = fx + dx * 0.6;
                let src_y = fy + dy * 0.6;

                let base = drawing::sample_bilinear(&self.original, src_x, src_y);
                let r = ((base.0[0] as f32 * shade).clamp(0.0, 255.0)) as u8;
                let g = ((base.0[1] as f32 * shade).clamp(0.0, 255.0)) as u8;
                let b = ((base.0[2] as f32 * shade).clamp(0.0, 255.0)) as u8;

                out.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 28. BUBBLE RAFT COALESCENCE
// ============================================================================

pub struct BubbleRaftPuzzle {
    original: RgbaImage,
    bubble_centers: Vec<(f32, f32, f32, f32)>, // (x, y, radius, pop_threshold)
}

impl BubbleRaftPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let cols = (complexity * 2 + 10).clamp(10, 50) as usize;
        let w = original.width() as f32;
        let h = original.height() as f32;
        let spacing = w / cols as f32;
        let rows = (h / (spacing * 0.866)).ceil() as usize;

        let mut bubble_centers = Vec::with_capacity(cols * rows);
        for r in 0..rows {
            let offset_x = if r % 2 == 1 { spacing * 0.5 } else { 0.0 };
            let cy = r as f32 * spacing * 0.866;
            for c in 0..cols {
                let cx = c as f32 * spacing + offset_x;
                let rad = spacing * 0.55;
                let pop_t = rng.gen_range(0.05..0.95);
                bubble_centers.push((cx, cy, rad, pop_t));
            }
        }

        Self {
            original: original.clone(),
            bubble_centers,
        }
    }
}

impl PuzzleSimulation for BubbleRaftPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::BubbleRaft
    }
    fn total_steps(&self) -> usize {
        self.bubble_centers.len()
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Bubble Raft: {:.1}% foam burst & clear", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        *out = self.original.clone();

        // Overlay un-popped soap bubbles
        for (cx, cy, radius, pop_t) in &self.bubble_centers {
            if progress >= *pop_t {
                continue; // Popped, clear image shows through
            }

            let rad = *radius;
            let r_sq = rad * rad;
            let min_x = (cx - rad).floor().max(0.0) as u32;
            let max_x = (cx + rad).ceil().min(w as f32 - 1.0) as u32;
            let min_y = (cy - rad).floor().max(0.0) as u32;
            let max_y = (cy + rad).ceil().min(h as f32 - 1.0) as u32;

            for y in min_y..=max_y {
                let dy = y as f32 - cy;
                for x in min_x..=max_x {
                    let dx = x as f32 - cx;
                    let d_sq = dx * dx + dy * dy;
                    if d_sq <= r_sq {
                        let ratio = (d_sq / r_sq).sqrt();
                        let mut px = *out.get_pixel(x, y);

                        // Soap film iridescence highlight
                        let soap_sheen = ((ratio * 12.0).sin() * 40.0) as i16;
                        px.0[0] = (px.0[0] as i16 + soap_sheen + 20).clamp(0, 255) as u8;
                        px.0[1] = (px.0[1] as i16 + (soap_sheen / 2) + 25).clamp(0, 255) as u8;
                        px.0[2] = (px.0[2] as i16 - soap_sheen + 40).clamp(0, 255) as u8;

                        if ratio > 0.88 {
                            // Dark hexagonal perimeter wall
                            px.0[0] = px.0[0] / 3;
                            px.0[1] = px.0[1] / 3;
                            px.0[2] = px.0[2] / 2;
                        }
                        out.put_pixel(x, y, px);
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 29. BOIDS FLOCKING SWARM
// ============================================================================

pub struct BoidFlockPuzzle {
    original: RgbaImage,
    boids: Vec<(f32, f32, f32, f32, f32, f32)>, // (home_x, home_y, start_x, start_y, mid_cx, mid_cy)
}

impl BoidFlockPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_steps = (complexity * 2 + 12).clamp(12, 48);
        let w = original.width() as f32;
        let h = original.height() as f32;
        let step_x = w / grid_steps as f32;
        let step_y = h / grid_steps as f32;

        let mut boids = Vec::with_capacity((grid_steps * grid_steps) as usize);

        for gy in 0..grid_steps {
            for gx in 0..grid_steps {
                let home_x = (gx as f32 + 0.5) * step_x;
                let home_y = (gy as f32 + 0.5) * step_y;

                // Scatter starting from outer boundary
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let dist = rng.gen_range(0.5..1.2) * (w + h);
                let start_x = home_x + angle.cos() * dist;
                let start_y = home_y + angle.sin() * dist;

                // Swooping curved midpoint trajectory
                let mid_cx = (home_x + start_x) * 0.5 + rng.gen_range(-120.0..120.0);
                let mid_cy = (home_y + start_y) * 0.5 + rng.gen_range(-120.0..120.0);

                boids.push((home_x, home_y, start_x, start_y, mid_cx, mid_cy));
            }
        }

        Self {
            original: original.clone(),
            boids,
        }
    }
}

impl PuzzleSimulation for BoidFlockPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::BoidFlock
    }
    fn total_steps(&self) -> usize {
        self.boids.len()
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Boid Flock: {:.1}% birds nested in canvas", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        // Night sky canvas
        for y in 0..h {
            for x in 0..w {
                out.put_pixel(x, y, Rgba([12, 14, 22, 255]));
            }
        }

        let t = progress;
        let omt = 1.0 - t;

        for (hx, hy, sx, sy, mx, my) in &self.boids {
            // Quadratic Bezier interpolation: B(t) = (1-t)^2 S + 2(1-t)t M + t^2 H
            let cur_x = omt * omt * sx + 2.0 * omt * t * mx + t * t * hx;
            let cur_y = omt * omt * sy + 2.0 * omt * t * my + t * t * hy;

            let ix = cur_x.round() as i32;
            let iy = cur_y.round() as i32;

            let src_px = (*hx as u32).min(w - 1);
            let src_py = (*hy as u32).min(h - 1);
            let color = *self.original.get_pixel(src_px, src_py);

            // Stamp bird particle (size 3x3)
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let px = ix + dx;
                    let py = iy + dy;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        out.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 30. PACHINKO PEG DROP
// ============================================================================

pub struct PachinkoDropPuzzle {
    original: RgbaImage,
    bins: usize,
    bin_fill_order: Vec<usize>,
}

impl PachinkoDropPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let bins = (complexity as usize * 3 + 12).clamp(12, 60);

        let mut bin_fill_order: Vec<usize> = (0..bins).collect();
        bin_fill_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            bins,
            bin_fill_order,
        }
    }
}

impl PuzzleSimulation for PachinkoDropPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::PachinkoDrop
    }
    fn total_steps(&self) -> usize {
        self.bins * 10
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Pachinko Pegs: {:.1}% balls collected in bins", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        // Dark arcade cabinet backdrop
        for y in 0..h {
            for x in 0..w {
                out.put_pixel(x, y, Rgba([20, 20, 28, 255]));
            }
        }

        let bin_w = w as f32 / self.bins as f32;
        let total_bins = self.bins;

        for (rank, &b) in self.bin_fill_order.iter().enumerate() {
            let bin_t = (progress * total_bins as f32 - rank as f32).clamp(0.0, 1.0);
            let fill_height = (h as f32 * bin_t) as u32;

            let bx0 = (b as f32 * bin_w) as u32;
            let bx1 = (((b + 1) as f32 * bin_w) as u32).min(w);

            // Filled bin column at bottom
            for y in (h - fill_height)..h {
                for x in bx0..bx1 {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                }
            }

            // Falling pachinko balls above fill level
            if bin_t > 0.01 && bin_t < 0.99 {
                let ball_y = (h - fill_height) as f32 - 15.0;
                let ball_x = (bx0 + bx1) as f32 * 0.5;
                let ix = ball_x as i32;
                let iy = ball_y as i32;

                for dy in -3..=3 {
                    for dx in -3..=3 {
                        if dx * dx + dy * dy <= 9 {
                            let px = ix + dx;
                            let py = iy + dy;
                            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                                out.put_pixel(px as u32, py as u32, Rgba([240, 220, 100, 255]));
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 31. JELLY SPRING LATTICE
// ============================================================================

pub struct JellyLatticePuzzle {
    original: RgbaImage,
    damping: f32,
    lattice_freq: f32,
}

impl JellyLatticePuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let damping = (complexity as f32 * 0.2 + 2.5).clamp(2.0, 6.5);
        let lattice_freq = (complexity as f32 * 0.4 + 4.0).clamp(4.0, 12.0);

        Self {
            original: original.clone(),
            damping,
            lattice_freq,
        }
    }
}

impl PuzzleSimulation for JellyLatticePuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::JellyLattice
    }
    fn total_steps(&self) -> usize {
        100
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Jelly Lattice: Oscillation damping {:.1}%", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let decay = (-self.damping * progress).exp();
        let phase = progress * std::f32::consts::TAU * 5.0;
        let amplitude = 28.0 * decay;

        let k = self.lattice_freq * std::f32::consts::PI / w as f32;

        for y in 0..h {
            let fy = y as f32;
            for x in 0..w {
                let fx = x as f32;

                // 2D standing wave damped harmonic oscillator
                let mode_x = (fx * k).sin() * (fy * k).cos();
                let mode_y = (fx * k).cos() * (fy * k).sin();

                let disp_x = mode_x * amplitude * phase.cos();
                let disp_y = mode_y * amplitude * phase.sin();

                let src_x = fx + disp_x;
                let src_y = fy + disp_y;

                let px = drawing::sample_bilinear(&self.original, src_x, src_y);
                out.put_pixel(x, y, px);
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}
