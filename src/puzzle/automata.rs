use image::{Rgba, RgbaImage};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{drawing, PuzzleKind, PuzzleSimulation};

// ============================================================================
// 32. CONWAY'S GAME OF LIFE
// ============================================================================

pub struct ConwayLifePuzzle {
    original: RgbaImage,
    grid_size: usize,
    activation_thresholds: Vec<f32>,
}

impl ConwayLifePuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_size = (complexity as usize * 3 + 16).clamp(16, 64);
        let total = grid_size * grid_size;

        let mut grid = vec![false; total];
        // Initialize random soup
        for cell in grid.iter_mut() {
            *cell = rng.gen_bool(0.35);
        }

        // Simulate 40 generations and record earliest activation / stability
        let mut activation_thresholds = vec![1.0f32; total];
        let steps = 40;

        for step in 0..steps {
            let mut next_grid = grid.clone();
            for y in 0..grid_size {
                for x in 0..grid_size {
                    let idx = y * grid_size + x;
                    let mut neighbors = 0;
                    for dy in [-1, 0, 1] {
                        for dx in [-1, 0, 1] {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = (x as i32 + dx).rem_euclid(grid_size as i32) as usize;
                            let ny = (y as i32 + dy).rem_euclid(grid_size as i32) as usize;
                            if grid[ny * grid_size + nx] {
                                neighbors += 1;
                            }
                        }
                    }
                    let alive = grid[idx];
                    let next_alive = if alive {
                        neighbors == 2 || neighbors == 3
                    } else {
                        neighbors == 3
                    };
                    next_grid[idx] = next_alive;

                    if next_alive && activation_thresholds[idx] >= 0.99 {
                        activation_thresholds[idx] = (step as f32 / steps as f32) * 0.85;
                    }
                }
            }
            grid = next_grid;
        }

        // Fill any unvisited cells with smooth distribution
        for (i, val) in activation_thresholds.iter_mut().enumerate() {
            if *val >= 0.99 {
                let r = i / grid_size;
                let c = i % grid_size;
                *val = 0.5 + 0.45 * ((r + c) as f32 / (2.0 * grid_size as f32));
            }
        }

        Self {
            original: original.clone(),
            grid_size,
            activation_thresholds,
        }
    }
}

impl PuzzleSimulation for ConwayLifePuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ConwayLife
    }
    fn total_steps(&self) -> usize {
        self.grid_size * self.grid_size
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Game of Life: {:.1}% cellular colonies stable", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let cell_w = w as f32 / self.grid_size as f32;
        let cell_h = h as f32 / self.grid_size as f32;

        for gy in 0..self.grid_size {
            let y0 = (gy as f32 * cell_h) as u32;
            let y1 = (((gy + 1) as f32 * cell_h) as u32).min(h);

            for gx in 0..self.grid_size {
                let x0 = (gx as f32 * cell_w) as u32;
                let x1 = (((gx + 1) as f32 * cell_w) as u32).min(w);

                let thresh = self.activation_thresholds[gy * self.grid_size + gx];
                let is_revealed = progress >= thresh;

                if is_revealed {
                    for y in y0..y1 {
                        for x in x0..x1 {
                            out.put_pixel(x, y, *self.original.get_pixel(x, y));
                        }
                    }
                } else {
                    // Cellular matrix glow grid
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let border = x == x0 || y == y0;
                            let color = if border {
                                Rgba([20, 60, 30, 255])
                            } else {
                                Rgba([10, 22, 14, 255])
                            };
                            out.put_pixel(x, y, color);
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
// 33. LANGTON'S ANT HIGHWAY
// ============================================================================

pub struct LangtonsAntPuzzle {
    original: RgbaImage,
    visit_times: Vec<f32>,
    grid_w: usize,
    grid_h: usize,
}

impl LangtonsAntPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_w = (complexity as usize * 4 + 20).clamp(20, 80);
        let grid_h = (grid_w * original.height() as usize) / original.width() as usize;
        let total = grid_w * grid_h;

        let mut visit_times = vec![1.0f32; total];
        let mut grid = vec![false; total];

        // 3 ants moving simultaneously
        let mut ants: Vec<(i32, i32, usize)> = vec![
            (grid_w as i32 / 2, grid_h as i32 / 2, 0),
            (grid_w as i32 / 4, grid_h as i32 / 3, 1),
            ((grid_w * 3) as i32 / 4, (grid_h * 2) as i32 / 3, 2),
        ];

        let dirs = [(0, -1), (1, 0), (0, 1), (-1, 0)]; // Up, Right, Down, Left
        let total_steps = 15000;

        for step in 0..total_steps {
            let t = (step as f32 / total_steps as f32) * 0.95;
            for (ax, ay, adir) in ants.iter_mut() {
                let x = (*ax).rem_euclid(grid_w as i32) as usize;
                let y = (*ay).rem_euclid(grid_h as i32) as usize;
                let idx = y * grid_w + x;

                if visit_times[idx] >= 0.99 {
                    visit_times[idx] = t;
                }

                // Langton's Ant rule: White -> turn 90 deg right; Black -> turn 90 deg left
                if grid[idx] {
                    *adir = (*adir + 3) % 4; // left
                    grid[idx] = false;
                } else {
                    *adir = (*adir + 1) % 4; // right
                    grid[idx] = true;
                }

                *ax += dirs[*adir].0;
                *ay += dirs[*adir].1;
            }
        }

        // Fill unvisited cells
        for (i, v) in visit_times.iter_mut().enumerate() {
            if *v >= 0.99 {
                let r = (i / grid_w) as f32 / grid_h as f32;
                let c = (i % grid_w) as f32 / grid_w as f32;
                *v = 0.7 + 0.28 * rng.gen_range(0.0..1.0) * (r + c) * 0.5;
            }
        }

        Self {
            original: original.clone(),
            visit_times,
            grid_w,
            grid_h,
        }
    }
}

impl PuzzleSimulation for LangtonsAntPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::LangtonsAnt
    }
    fn total_steps(&self) -> usize {
        self.grid_w * self.grid_h
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Langton's Ant: {:.1}% highways paved", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let cell_w = w as f32 / self.grid_w as f32;
        let cell_h = h as f32 / self.grid_h as f32;

        for gy in 0..self.grid_h {
            let y0 = (gy as f32 * cell_h) as u32;
            let y1 = (((gy + 1) as f32 * cell_h) as u32).min(h);

            for gx in 0..self.grid_w {
                let x0 = (gx as f32 * cell_w) as u32;
                let x1 = (((gx + 1) as f32 * cell_w) as u32).min(w);

                let v = self.visit_times[gy * self.grid_w + gx];

                if progress >= v {
                    for y in y0..y1 {
                        for x in x0..x1 {
                            out.put_pixel(x, y, *self.original.get_pixel(x, y));
                        }
                    }
                } else {
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let checker = ((gx + gy) % 2) as u8 * 8;
                            out.put_pixel(x, y, Rgba([18 + checker, 16 + checker, 24 + checker, 255]));
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
// 34. REACTION-DIFFUSION TURING WAVES
// ============================================================================

pub struct ReactionDiffusionPuzzle {
    original: RgbaImage,
    freq_u: f32,
    freq_v: f32,
}

impl ReactionDiffusionPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let freq_u = (complexity as f32 * 0.015 + 0.05) * rng.gen_range(0.9..1.1);
        let freq_v = (complexity as f32 * 0.012 + 0.04) * rng.gen_range(0.9..1.1);

        Self {
            original: original.clone(),
            freq_u,
            freq_v,
        }
    }
}

impl PuzzleSimulation for ReactionDiffusionPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ReactionDiffusion
    }
    fn total_steps(&self) -> usize {
        100
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Reaction-Diffusion: Turing morphogen convergence {:.1}%", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let blend = progress * progress * (3.0 - 2.0 * progress);
        let t_inv = 1.0 - progress;

        for y in 0..h {
            let fy = y as f32;
            for x in 0..w {
                let fx = x as f32;

                // Gray-Scott activator-inhibitor Turing pattern approximation
                let u = (fx * self.freq_u).sin() * (fy * self.freq_u).cos();
                let v = ((fx + fy) * self.freq_v * 0.707).sin();
                let morphogen = ((u * 0.6 + v * 0.4) * 128.0 + 128.0).clamp(0.0, 255.0) as u8;

                let orig = self.original.get_pixel(x, y);

                let r = ((orig.0[0] as f32 * blend) + (morphogen as f32 * 0.9 * t_inv)) as u8;
                let g = ((orig.0[1] as f32 * blend) + (morphogen as f32 * 0.5 * t_inv)) as u8;
                let b = ((orig.0[2] as f32 * blend) + (morphogen as f32 * 1.1 * t_inv).min(255.0)) as u8;

                out.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 35. MANDELBROT ITERATION DISSOLVE
// ============================================================================

pub struct MandelbrotFractalPuzzle {
    original: RgbaImage,
    max_iter: usize,
}

impl MandelbrotFractalPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let max_iter = (complexity as usize * 3 + 24).clamp(24, 80);
        Self {
            original: original.clone(),
            max_iter,
        }
    }
}

impl PuzzleSimulation for MandelbrotFractalPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::MandelbrotFractal
    }
    fn total_steps(&self) -> usize {
        self.max_iter
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Mandelbrot: Escape boundary iteration {}/{}", (progress * self.max_iter as f32) as usize, self.max_iter)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let cur_iter = (progress * self.max_iter as f32) as usize;

        for y in 0..h {
            let cy = (y as f32 / h as f32) * 2.4 - 1.2;
            for x in 0..w {
                let cx = (x as f32 / w as f32) * 3.2 - 2.2;

                let mut zx = 0.0f32;
                let mut zy = 0.0f32;
                let mut iter = 0;

                while zx * zx + zy * zy <= 4.0 && iter < self.max_iter {
                    let tmp = zx * zx - zy * zy + cx;
                    zy = 2.0 * zx * zy + cy;
                    zx = tmp;
                    iter += 1;
                }

                if iter <= cur_iter || progress >= 0.95 {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                } else {
                    // Electric psychedelic escape band palette
                    let norm = iter as f32 / self.max_iter as f32;
                    let r = ((norm * 6.28).sin() * 127.0 + 128.0) as u8;
                    let g = (((norm * 6.28) + 2.09).sin() * 127.0 + 128.0) as u8;
                    let b = (((norm * 6.28) + 4.18).sin() * 127.0 + 128.0) as u8;
                    out.put_pixel(x, y, Rgba([r / 2, g / 2, b / 2, 255]));
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 36. L-SYSTEM BOTANICAL BLOOM
// ============================================================================

pub struct LSystemBloomPuzzle {
    original: RgbaImage,
    branches: Vec<(f32, f32, f32, f32, f32)>, // (x0, y0, x1, y1, reveal_time)
}

impl LSystemBloomPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let depth = (complexity / 4 + 4).clamp(4, 8) as usize;
        let w = original.width() as f32;
        let h = original.height() as f32;

        let mut branches = Vec::new();

        fn build_tree(
            x: f32,
            y: f32,
            angle: f32,
            len: f32,
            d: usize,
            t: f32,
            branches: &mut Vec<(f32, f32, f32, f32, f32)>,
            rng: &mut ChaCha8Rng,
        ) {
            if d == 0 || len < 4.0 {
                return;
            }
            let nx = x + angle.cos() * len;
            let ny = y + angle.sin() * len;
            branches.push((x, y, nx, ny, t));

            let branch_angle = rng.gen_range(0.35..0.55);
            let next_len = len * rng.gen_range(0.7..0.82);

            build_tree(nx, ny, angle - branch_angle, next_len, d - 1, t + 0.12, branches, rng);
            build_tree(nx, ny, angle + branch_angle, next_len, d - 1, t + 0.12, branches, rng);
        }

        // 3 tree trunks sprouting from bottom and sides
        build_tree(w * 0.5, h, -std::f32::consts::FRAC_PI_2, h * 0.28, depth, 0.05, &mut branches, &mut rng);
        build_tree(w * 0.2, h, -std::f32::consts::FRAC_PI_2 + 0.2, h * 0.22, depth - 1, 0.1, &mut branches, &mut rng);
        build_tree(w * 0.8, h, -std::f32::consts::FRAC_PI_2 - 0.2, h * 0.22, depth - 1, 0.1, &mut branches, &mut rng);

        Self {
            original: original.clone(),
            branches,
        }
    }
}

impl PuzzleSimulation for LSystemBloomPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::LSystemBloom
    }
    fn total_steps(&self) -> usize {
        self.branches.len()
    }
    fn status_label(&self, progress: f32) -> String {
        format!("L-System: {:.1}% branches & leaves blossomed", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        // Night greenhouse backdrop
        for y in 0..h {
            for x in 0..w {
                out.put_pixel(x, y, Rgba([14, 20, 18, 255]));
            }
        }

        let bloom_radius = (w as f32 * 0.06 * (progress * 2.0).min(1.0)).max(4.0);

        for (x0, y0, x1, y1, t) in &self.branches {
            if progress < *t {
                continue;
            }

            let local_t = ((progress - t) / 0.15).clamp(0.0, 1.0);
            let cur_x1 = x0 + (x1 - x0) * local_t;
            let cur_y1 = y0 + (y1 - y0) * local_t;

            // Draw branch
            drawing::draw_line(
                out,
                *x0 as i32,
                *y0 as i32,
                cur_x1 as i32,
                cur_y1 as i32,
                Rgba([60, 120, 70, 255]),
                2,
            );

            // Bloom leaf/pixel cluster at the tip
            if local_t >= 0.95 {
                let bx = *x1 as i32;
                let by = *y1 as i32;
                let br = bloom_radius as i32;

                for dy in -br..=br {
                    for dx in -br..=br {
                        if dx * dx + dy * dy <= br * br {
                            let px = bx + dx;
                            let py = by + dy;
                            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                                out.put_pixel(px as u32, py as u32, *self.original.get_pixel(px as u32, py as u32));
                            }
                        }
                    }
                }
            }
        }

        // Final full-canvas blend in last 10%
        if progress > 0.88 {
            let final_blend = (progress - 0.88) / 0.12;
            for y in 0..h {
                for x in 0..w {
                    let cur = out.get_pixel(x, y);
                    let orig = self.original.get_pixel(x, y);
                    let r = ((cur.0[0] as f32 * (1.0 - final_blend)) + (orig.0[0] as f32 * final_blend)) as u8;
                    let g = ((cur.0[1] as f32 * (1.0 - final_blend)) + (orig.0[1] as f32 * final_blend)) as u8;
                    let b = ((cur.0[2] as f32 * (1.0 - final_blend)) + (orig.0[2] as f32 * final_blend)) as u8;
                    out.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}

// ============================================================================
// 37. FOREST FIRE PERCOLATION
// ============================================================================

pub struct ForestFirePuzzle {
    original: RgbaImage,
    burn_times: Vec<f32>,
    grid_w: usize,
    grid_h: usize,
}

impl ForestFirePuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_w = (complexity as usize * 4 + 20).clamp(20, 80);
        let grid_h = (grid_w * original.height() as usize) / original.width() as usize;
        let total = grid_w * grid_h;

        let mut burn_times = vec![1.0f32; total];
        let cx = rng.gen_range(0.2..0.8) * grid_w as f32;
        let cy = rng.gen_range(0.2..0.8) * grid_h as f32;

        let max_d = (grid_w as f32).hypot(grid_h as f32);

        for gy in 0..grid_h {
            for gx in 0..grid_w {
                let dx = gx as f32 - cx;
                let dy = gy as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let noise = rng.gen_range(-0.15..0.15);
                let t = ((dist / max_d) + noise).clamp(0.02, 0.95);
                burn_times[gy * grid_w + gx] = t;
            }
        }

        Self {
            original: original.clone(),
            burn_times,
            grid_w,
            grid_h,
        }
    }
}

impl PuzzleSimulation for ForestFirePuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ForestFire
    }
    fn total_steps(&self) -> usize {
        self.grid_w * self.grid_h
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Forest Fire: {:.1}% embers cleared to reveal image", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let cell_w = w as f32 / self.grid_w as f32;
        let cell_h = h as f32 / self.grid_h as f32;

        for gy in 0..self.grid_h {
            let y0 = (gy as f32 * cell_h) as u32;
            let y1 = (((gy + 1) as f32 * cell_h) as u32).min(h);

            for gx in 0..self.grid_w {
                let x0 = (gx as f32 * cell_w) as u32;
                let x1 = (((gx + 1) as f32 * cell_w) as u32).min(w);

                let bt = self.burn_times[gy * self.grid_w + gx];
                let diff = progress - bt;

                if diff >= 0.05 {
                    // Burned through: crisp picture revealed
                    for y in y0..y1 {
                        for x in x0..x1 {
                            out.put_pixel(x, y, *self.original.get_pixel(x, y));
                        }
                    }
                } else if diff >= 0.0 {
                    // Firefront boundary: intense orange/yellow flame
                    for y in y0..y1 {
                        for x in x0..x1 {
                            out.put_pixel(x, y, Rgba([255, 160, 40, 255]));
                        }
                    }
                } else {
                    // Unburned charcoal ash forest
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let ash = 15 + ((x * 3 + y * 7) % 11) as u8;
                            out.put_pixel(x, y, Rgba([ash, ash + 4, ash + 2, 255]));
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
// 38. WIREWORLD LOGIC PULSES
// ============================================================================

pub struct WireworldPuzzle {
    original: RgbaImage,
    channels: usize,
}

impl WireworldPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let channels = (complexity as usize * 2 + 8).clamp(8, 36);
        Self {
            original: original.clone(),
            channels,
        }
    }
}

impl PuzzleSimulation for WireworldPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Wireworld
    }
    fn total_steps(&self) -> usize {
        self.channels * 8
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Wireworld: {:.1}% circuit logic traces closed", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        // Silicon substrate circuit board background
        for y in 0..h {
            for x in 0..w {
                out.put_pixel(x, y, Rgba([10, 28, 22, 255]));
            }
        }

        let ch_h = h as f32 / self.channels as f32;

        for c in 0..self.channels {
            let y0 = (c as f32 * ch_h) as u32;
            let y1 = (((c + 1) as f32 * ch_h) as u32).min(h);

            let pulse_speed = 1.0 + (c % 3) as f32 * 0.2;
            let ch_p = (progress * pulse_speed).clamp(0.0, 1.0);
            let frontier_x = (w as f32 * ch_p) as u32;

            // Resolved copper / image behind the pulse
            for y in y0..y1 {
                for x in 0..frontier_x {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                }
            }

            // Electron head pulse (bright blue) & tail (bright red)
            if ch_p > 0.01 && ch_p < 0.99 {
                let fx = frontier_x.min(w - 1);
                for y in y0..y1 {
                    out.put_pixel(fx, y, Rgba([80, 200, 255, 255]));
                    if fx > 1 {
                        out.put_pixel(fx - 1, y, Rgba([255, 80, 80, 255]));
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
// 39. CYCLIC CELLULAR AUTOMATON
// ============================================================================

pub struct CyclicAutomatonPuzzle {
    original: RgbaImage,
    states: usize,
    spiral_freq: f32,
}

impl CyclicAutomatonPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let states = (complexity as usize / 3 + 6).clamp(6, 12);
        let spiral_freq = (complexity as f32 * 0.04 + 0.15) * rng.gen_range(0.9..1.1);

        Self {
            original: original.clone(),
            states,
            spiral_freq,
        }
    }
}

impl PuzzleSimulation for CyclicAutomatonPuzzle {
    fn update_source(&mut self, image: &RgbaImage) {
        self.original = image.clone();
    }
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::CyclicAutomaton
    }
    fn total_steps(&self) -> usize {
        100
    }
    fn status_label(&self, progress: f32) -> String {
        format!("Cyclic Automaton: State cycle convergence {:.1}%", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();

        if progress >= 0.9999 {
            *out = self.original.clone();
            return;
        }

        let blend = progress * progress * (3.0 - 2.0 * progress);
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;

        for y in 0..h {
            let dy = y as f32 - cy;
            for x in 0..w {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                // Multi-armed spiral state pattern
                let spiral_val = ((r * self.spiral_freq - angle * 4.0).rem_euclid(std::f32::consts::TAU)
                    / std::f32::consts::TAU)
                    * self.states as f32;
                let state_idx = spiral_val as usize % self.states;

                let hue = (state_idx as f32 / self.states as f32) * std::f32::consts::TAU;
                let cr = (hue.sin() * 127.0 + 128.0) as u8;
                let cg = ((hue + 2.09).sin() * 127.0 + 128.0) as u8;
                let cb = ((hue + 4.18).sin() * 127.0 + 128.0) as u8;

                let orig = self.original.get_pixel(x, y);

                let r_out = ((orig.0[0] as f32 * blend) + (cr as f32 * (1.0 - blend))) as u8;
                let g_out = ((orig.0[1] as f32 * blend) + (cg as f32 * (1.0 - blend))) as u8;
                let b_out = ((orig.0[2] as f32 * blend) + (cb as f32 * (1.0 - blend))) as u8;

                out.put_pixel(x, y, Rgba([r_out, g_out, b_out, 255]));
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        self.render_frame(1.0 - progress, out);
    }
}
