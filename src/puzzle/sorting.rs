use image::{Rgba, RgbaImage};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{PuzzleKind, PuzzleSimulation};

// ============================================================================
// 9. STRIP SORT (QUICKSORT ON IMAGE STRIPS)
// ============================================================================

pub struct StripSortPuzzle {
    original: RgbaImage,
    num_strips: usize,
    swaps: Vec<(usize, usize)>,
}

impl StripSortPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let num_strips = (complexity as usize * 6 + 12).clamp(16, 128);

        // Scramble strip indices
        let mut perm: Vec<usize> = (0..num_strips).collect();
        perm.shuffle(&mut rng);

        // Quicksort swap recording
        let mut cur = perm.clone();
        let mut swaps = Vec::new();

        fn partition(arr: &mut [usize], low: usize, high: usize, swaps: &mut Vec<(usize, usize)>) -> usize {
            let pivot = arr[high];
            let mut i = low;
            for j in low..high {
                if arr[j] < pivot {
                    arr.swap(i, j);
                    swaps.push((i, j));
                    i += 1;
                }
            }
            arr.swap(i, high);
            swaps.push((i, high));
            i
        }

        fn quicksort(arr: &mut [usize], low: usize, high: usize, swaps: &mut Vec<(usize, usize)>) {
            if low < high {
                let pi = partition(arr, low, high, swaps);
                if pi > 0 {
                    quicksort(arr, low, pi - 1, swaps);
                }
                quicksort(arr, pi + 1, high, swaps);
            }
        }

        quicksort(&mut cur, 0, num_strips - 1, &mut swaps);

        Self {
            original: original.clone(),
            num_strips,
            swaps,
        }
    }

    fn state_at_step(&self, step: usize) -> Vec<usize> {
        let mut arr: Vec<usize> = (0..self.num_strips).collect();
        // Undo swaps from final sorted state
        for &(a, b) in self.swaps.iter().rev() {
            arr.swap(a, b);
        }
        // Apply forward up to step
        for i in 0..step.min(self.swaps.len()) {
            let (a, b) = self.swaps[i];
            arr.swap(a, b);
        }
        arr
    }
}

impl PuzzleSimulation for StripSortPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::StripSort
    }

    fn total_steps(&self) -> usize {
        self.swaps.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let step = (progress * self.total_steps() as f32).floor() as usize;
        format!("Quicksort Strips: swap {} / {}", step.min(self.total_steps()), self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width();
        let h = self.original.height();
        let strip_w = w as f32 / self.num_strips as f32;

        let arr = self.state_at_step(step_idx);
        let active = if step_idx < total { Some(self.swaps[step_idx]) } else { None };

        for pos in 0..self.num_strips {
            let strip_id = arr[pos];

            let cur_x = match active {
                Some((a, b)) if pos == a => {
                    let x0 = a as f32 * strip_w;
                    let x1 = b as f32 * strip_w;
                    x0 + (x1 - x0) * ease
                }
                Some((a, b)) if pos == b => {
                    let x0 = b as f32 * strip_w;
                    let x1 = a as f32 * strip_w;
                    x0 + (x1 - x0) * ease
                }
                _ => pos as f32 * strip_w,
            };

            let src_x = (strip_id as f32 * strip_w) as u32;
            let sw = strip_w.ceil() as u32;

            for y in 0..h {
                for sx in 0..sw {
                    let src_px = (src_x + sx).min(w - 1);
                    let dst_px = (cur_x + sx as f32).round() as i32;
                    if dst_px >= 0 && dst_px < w as i32 {
                        let mut color = *self.original.get_pixel(src_px, y);
                        if let Some((a, b)) = active {
                            if (pos == a || pos == b) && progress < 0.999 {
                                color.0[0] = color.0[0].saturating_add(30);
                                color.0[1] = color.0[1].saturating_add(30);
                            }
                        }
                        out.put_pixel(dst_px as u32, y, color);
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width();
        let h = self.original.height();
        let strip_w = w as f32 / self.num_strips as f32;

        let reverse_step = total.saturating_sub(step_idx);
        let arr = self.state_at_step(reverse_step);
        let active = if step_idx < total && reverse_step > 0 {
            Some(self.swaps[reverse_step - 1])
        } else {
            None
        };

        for pos in 0..self.num_strips {
            let strip_id = arr[pos];

            let cur_x = match active {
                Some((a, b)) if pos == a => {
                    let x0 = a as f32 * strip_w;
                    let x1 = b as f32 * strip_w;
                    x0 + (x1 - x0) * ease
                }
                Some((a, b)) if pos == b => {
                    let x0 = b as f32 * strip_w;
                    let x1 = a as f32 * strip_w;
                    x0 + (x1 - x0) * ease
                }
                _ => pos as f32 * strip_w,
            };

            let src_x = (strip_id as f32 * strip_w) as u32;
            let sw = strip_w.ceil() as u32;

            for y in 0..h {
                for sx in 0..sw {
                    let src_px = (src_x + sx).min(w - 1);
                    let dst_px = (cur_x + sx as f32).round() as i32;
                    if dst_px >= 0 && dst_px < w as i32 {
                        let mut color = *self.original.get_pixel(src_px, y);
                        if let Some((a, b)) = active {
                            if (pos == a || pos == b) && progress > 0.001 {
                                color.0[2] = color.0[2].saturating_add(40);
                            }
                        }
                        out.put_pixel(dst_px as u32, y, color);
                    }
                }
            }
        }
    }
}

// ============================================================================
// 10. HILBERT CURVE SORT
// ============================================================================

pub struct HilbertCurvePuzzle {
    original: RgbaImage,
    order: usize,
    hilbert_points: Vec<(usize, usize)>,
    swaps: Vec<(usize, usize)>,
}

fn d2xy(n: usize, mut d: usize) -> (usize, usize) {
    let mut x = 0;
    let mut y = 0;
    let mut s = 1;
    while s < n {
        let rx = 1 & (d / 2);
        let ry = 1 & (d ^ rx);
        // rot
        if ry == 0 {
            if rx == 1 {
                x = s - 1 - x;
                y = s - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        x += s * rx;
        y += s * ry;
        d /= 4;
        s *= 2;
    }
    (x, y)
}

impl HilbertCurvePuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        // Hilbert curve order 3 (8x8=64), order 4 (16x16=256), or order 5 (32x32=1024)
        let order = if complexity >= 12 { 5 } else if complexity >= 6 { 4 } else { 3 };
        let n = 1 << order;
        let total = n * n;

        let mut hilbert_points = Vec::with_capacity(total);
        for d in 0..total {
            hilbert_points.push(d2xy(n, d));
        }

        // Scramble along Hilbert curve
        let mut perm: Vec<usize> = (0..total).collect();
        perm.shuffle(&mut rng);

        // Sorting swaps
        let mut cur = perm.clone();
        let mut swaps = Vec::new();
        for i in 0..total {
            while cur[i] != i {
                let target = cur[i];
                cur.swap(i, target);
                swaps.push((i, target));
            }
        }

        Self {
            original: original.clone(),
            order,
            hilbert_points,
            swaps,
        }
    }

    fn state_at_step(&self, step: usize) -> Vec<usize> {
        let total = self.hilbert_points.len();
        let mut arr: Vec<usize> = (0..total).collect();
        for &(a, b) in self.swaps.iter().rev() {
            arr.swap(a, b);
        }
        for i in 0..step.min(self.swaps.len()) {
            let (a, b) = self.swaps[i];
            arr.swap(a, b);
        }
        arr
    }
}

impl PuzzleSimulation for HilbertCurvePuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::HilbertCurveSort
    }

    fn total_steps(&self) -> usize {
        self.swaps.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let step = (progress * self.total_steps() as f32).floor() as usize;
        format!("Hilbert Sort: {} / {} swaps along curve", step.min(self.total_steps()), self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width();
        let h = self.original.height();
        let n = 1 << self.order;
        let bw = w as f32 / n as f32;
        let bh = h as f32 / n as f32;

        let arr = self.state_at_step(step_idx);
        let active = if step_idx < total { Some(self.swaps[step_idx]) } else { None };

        for pos in 0..arr.len() {
            let block_id = arr[pos];
            let (curr_hx, curr_hy) = self.hilbert_points[pos];

            let (dst_x, dst_y) = match active {
                Some((a, b)) if pos == a => {
                    let (bx, by) = self.hilbert_points[b];
                    let x0 = curr_hx as f32 * bw;
                    let y0 = curr_hy as f32 * bh;
                    let x1 = bx as f32 * bw;
                    let y1 = by as f32 * bh;
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease)
                }
                Some((a, b)) if pos == b => {
                    let (ax, ay) = self.hilbert_points[a];
                    let x0 = curr_hx as f32 * bw;
                    let y0 = curr_hy as f32 * bh;
                    let x1 = ax as f32 * bw;
                    let y1 = ay as f32 * bh;
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease)
                }
                _ => (curr_hx as f32 * bw, curr_hy as f32 * bh),
            };

            let (src_hx, src_hy) = self.hilbert_points[block_id];
            let sx0 = (src_hx as f32 * bw) as u32;
            let sy0 = (src_hy as f32 * bh) as u32;

            let dxi = dst_x.round() as i32;
            let dyi = dst_y.round() as i32;
            let block_wi = bw.ceil() as u32;
            let block_hi = bh.ceil() as u32;

            for by in 0..block_hi {
                let dy = dyi + by as i32;
                if dy < 0 || dy >= h as i32 { continue; }
                let sy = (sy0 + by).min(h - 1);
                for bx in 0..block_wi {
                    let dx = dxi + bx as i32;
                    if dx < 0 || dx >= w as i32 { continue; }
                    let sx = (sx0 + bx).min(w - 1);

                    out.put_pixel(dx as u32, dy as u32, *self.original.get_pixel(sx, sy));
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width();
        let h = self.original.height();
        let n = 1 << self.order;
        let bw = w as f32 / n as f32;
        let bh = h as f32 / n as f32;

        let reverse_step = total.saturating_sub(step_idx);
        let arr = self.state_at_step(reverse_step);
        let active = if step_idx < total && reverse_step > 0 {
            Some(self.swaps[reverse_step - 1])
        } else {
            None
        };

        for pos in 0..arr.len() {
            let block_id = arr[pos];
            let (curr_hx, curr_hy) = self.hilbert_points[pos];

            let (dst_x, dst_y) = match active {
                Some((a, b)) if pos == a => {
                    let (bx, by) = self.hilbert_points[b];
                    let x0 = curr_hx as f32 * bw;
                    let y0 = curr_hy as f32 * bh;
                    let x1 = bx as f32 * bw;
                    let y1 = by as f32 * bh;
                    let loft = 18.0 * (sub_p * (1.0 - sub_p) * 4.0);
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease - loft)
                }
                Some((a, b)) if pos == b => {
                    let (ax, ay) = self.hilbert_points[a];
                    let x0 = curr_hx as f32 * bw;
                    let y0 = curr_hy as f32 * bh;
                    let x1 = ax as f32 * bw;
                    let y1 = ay as f32 * bh;
                    let loft = 18.0 * (sub_p * (1.0 - sub_p) * 4.0);
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease + loft)
                }
                _ => (curr_hx as f32 * bw, curr_hy as f32 * bh),
            };

            let (src_hx, src_hy) = self.hilbert_points[block_id];
            let sx0 = (src_hx as f32 * bw) as u32;
            let sy0 = (src_hy as f32 * bh) as u32;

            let dxi = dst_x.round() as i32;
            let dyi = dst_y.round() as i32;
            let block_wi = bw.ceil() as u32;
            let block_hi = bh.ceil() as u32;

            for by in 0..block_hi {
                let dy = dyi + by as i32;
                if dy < 0 || dy >= h as i32 { continue; }
                let sy = (sy0 + by).min(h - 1);
                for bx in 0..block_wi {
                    let dx = dxi + bx as i32;
                    if dx < 0 || dx >= w as i32 { continue; }
                    let sx = (sx0 + bx).min(w - 1);

                    out.put_pixel(dx as u32, dy as u32, *self.original.get_pixel(sx, sy));
                }
            }
        }
    }
}

// ============================================================================
// 11. VORONOI SHATTERED GLASS
// ============================================================================

pub struct VoronoiShardsPuzzle {
    original: RgbaImage,
    seed_points: Vec<(f32, f32)>,
    scatter: Vec<(f32, f32, f32)>,
    solve_order: Vec<usize>,
}

impl VoronoiShardsPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let num_seeds = (complexity as usize * 5 + 15).clamp(20, 120);
        let w = original.width() as f32;
        let h = original.height() as f32;

        let mut seed_points = Vec::with_capacity(num_seeds);
        for _ in 0..num_seeds {
            seed_points.push((rng.gen_range(0.0..w), rng.gen_range(0.0..h)));
        }

        let mut scatter = Vec::with_capacity(num_seeds);
        for i in 0..num_seeds {
            let (sx, sy) = seed_points[i];
            let dx = sx - w * 0.5;
            let dy = sy - h * 0.5;
            let dist = dx.hypot(dy) * rng.gen_range(0.6..1.4) + 60.0;
            let ang = dy.atan2(dx) + rng.gen_range(-0.4..0.4);
            let rot = rng.gen_range(-1.5..1.5);
            scatter.push((ang.cos() * dist, ang.sin() * dist, rot));
        }

        let mut solve_order: Vec<usize> = (0..num_seeds).collect();
        // Outside shards fly in first, center last
        solve_order.sort_by(|&a, &b| {
            let da = (seed_points[a].0 - w * 0.5).hypot(seed_points[a].1 - h * 0.5);
            let db = (seed_points[b].0 - w * 0.5).hypot(seed_points[b].1 - h * 0.5);
            db.partial_cmp(&da).unwrap()
        });

        Self {
            original: original.clone(),
            seed_points,
            scatter,
            solve_order,
        }
    }
}

impl PuzzleSimulation for VoronoiShardsPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::VoronoiShards
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let placed = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Voronoi Shards: {} / {} shards reassembled", placed, self.total_steps())
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

        // Background
        for px in out.pixels_mut() {
            *px = Rgba([14, 16, 22, 255]);
        }

        // For each pixel, find nearest Voronoi seed
        for y in 0..h {
            let py = y as f32;
            for x in 0..w {
                let px = x as f32;

                let mut min_dist_sq = f32::MAX;
                let mut best_seed = 0;

                for (idx, &(sx, sy)) in self.seed_points.iter().enumerate() {
                    let d = (px - sx).powi(2) + (py - sy).powi(2);
                    if d < min_dist_sq {
                        min_dist_sq = d;
                        best_seed = idx;
                    }
                }

                // Determine if this seed is in motion
                let order_pos = self.solve_order.iter().position(|&s| s == best_seed).unwrap_or(0);
                let p = (target_f - order_pos as f32).clamp(0.0, 1.0);
                let ease = p * p * (3.0 - 2.0 * p);

                let (sdx, sdy, srot) = self.scatter[best_seed];
                let (sx, sy) = self.seed_points[best_seed];

                let cur_dx = sdx * (1.0 - ease);
                let cur_dy = sdy * (1.0 - ease);
                let cur_rot = srot * (1.0 - ease);

                // Transform pixel
                let lx = px - sx;
                let ly = py - sy;
                let cos_r = cur_rot.cos();
                let sin_r = cur_rot.sin();
                let rx = lx * cos_r - ly * sin_r;
                let ry = lx * sin_r + ly * cos_r;

                let dst_x = (sx + cur_dx + rx).round() as i32;
                let dst_y = (sy + cur_dy + ry).round() as i32;

                if dst_x >= 0 && dst_x < w as i32 && dst_y >= 0 && dst_y < h as i32 {
                    let mut color = *self.original.get_pixel(x, y);
                    if ease < 0.999 {
                        // Glass shard edge sheen
                        if min_dist_sq.sqrt() < 3.0 {
                            color.0[0] = color.0[0].saturating_add(60);
                            color.0[1] = color.0[1].saturating_add(60);
                            color.0[2] = color.0[2].saturating_add(90);
                        }
                    }
                    out.put_pixel(dst_x as u32, dst_y as u32, color);
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress <= 0.001 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let total = self.total_steps();
        let target_f = progress * total as f32;

        for px in out.pixels_mut() {
            *px = Rgba([14, 16, 22, 255]);
        }

        for y in 0..h {
            let py = y as f32;
            for x in 0..w {
                let px = x as f32;

                let mut min_dist_sq = f32::MAX;
                let mut best_seed = 0;

                for (idx, &(sx, sy)) in self.seed_points.iter().enumerate() {
                    let d = (px - sx).powi(2) + (py - sy).powi(2);
                    if d < min_dist_sq {
                        min_dist_sq = d;
                        best_seed = idx;
                    }
                }

                // Rollback explodes center shards outward first (reversed solve order)
                let order_pos = self.solve_order.iter().rev().position(|&s| s == best_seed).unwrap_or(0);
                let p = (target_f - order_pos as f32).clamp(0.0, 1.0);
                let ease = p * p * (3.0 - 2.0 * p);

                let (sdx, sdy, srot) = self.scatter[best_seed];
                let (sx, sy) = self.seed_points[best_seed];

                let cur_dx = sdx * ease;
                let cur_dy = sdy * ease;
                let cur_rot = srot * ease;

                let lx = px - sx;
                let ly = py - sy;
                let cos_r = cur_rot.cos();
                let sin_r = cur_rot.sin();
                let rx = lx * cos_r - ly * sin_r;
                let ry = lx * sin_r + ly * cos_r;

                let dst_x = (sx + cur_dx + rx).round() as i32;
                let dst_y = (sy + cur_dy + ry).round() as i32;

                if dst_x >= 0 && dst_x < w as i32 && dst_y >= 0 && dst_y < h as i32 {
                    let mut color = *self.original.get_pixel(x, y);
                    if min_dist_sq.sqrt() < 3.0 && progress < 0.999 {
                        color.0[0] = color.0[0].saturating_add(60);
                        color.0[1] = color.0[1].saturating_add(60);
                        color.0[2] = color.0[2].saturating_add(90);
                    }
                    out.put_pixel(dst_x as u32, dst_y as u32, color);
                }
            }
        }
    }
}

// ============================================================================
// 12. QUADTREE RECURSIVE SPLIT
// ============================================================================

pub struct QuadtreeUnfoldPuzzle {
    original: RgbaImage,
    split_sequence: Vec<(u32, u32, u32, u32)>, // (x, y, w, h)
}

impl QuadtreeUnfoldPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let max_depth = (complexity / 2 + 3).clamp(3, 8);
        let mut queue = vec![(0, 0, original.width(), original.height(), 0)];
        let mut split_sequence = Vec::new();

        while let Some((x, y, w, h, depth)) = queue.pop() {
            if depth < max_depth && w > 4 && h > 4 {
                split_sequence.push((x, y, w, h));
                let hw = w / 2;
                let hh = h / 2;
                queue.push((x + hw, y + hh, w - hw, h - hh, depth + 1));
                queue.push((x, y + hh, hw, h - hh, depth + 1));
                queue.push((x + hw, y, w - hw, hh, depth + 1));
                queue.push((x, y, hw, hh, depth + 1));
            }
        }

        Self {
            original: original.clone(),
            split_sequence,
        }
    }

    fn average_color(&self, x: u32, y: u32, w: u32, h: u32) -> Rgba<u8> {
        let mut r = 0u64;
        let mut g = 0u64;
        let mut b = 0u64;
        let mut count = 0u64;

        for py in y..(y + h).min(self.original.height()) {
            for px in x..(x + w).min(self.original.width()) {
                let c = self.original.get_pixel(px, py);
                r += c[0] as u64;
                g += c[1] as u64;
                b += c[2] as u64;
                count += 1;
            }
        }

        if count == 0 {
            Rgba([0, 0, 0, 255])
        } else {
            Rgba([
                (r / count) as u8,
                (g / count) as u8,
                (b / count) as u8,
                255,
            ])
        }
    }
}

impl PuzzleSimulation for QuadtreeUnfoldPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::QuadtreeUnfold
    }

    fn total_steps(&self) -> usize {
        self.split_sequence.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let step = (progress * self.total_steps() as f32).floor() as usize;
        format!("Quadtree: {} / {} quadrant splits", step.min(self.total_steps()), self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let total = self.total_steps();
        let target_step = (progress * total as f32).floor() as usize;

        // Render leaf blocks
        *out = self.original.clone();

        // For blocks not yet split, fill with average color
        for i in (target_step..total).rev() {
            let (x, y, w, h) = self.split_sequence[i];
            let avg = self.average_color(x, y, w, h);
            for py in y..(y + h).min(out.height()) {
                for px in x..(x + w).min(out.width()) {
                    out.put_pixel(px, py, avg);
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress <= 0.001 {
            *out = self.original.clone();
            return;
        }

        let total = self.total_steps();
        let target_step = ((1.0 - progress) * total as f32).floor() as usize;

        *out = self.original.clone();

        for i in (target_step..total).rev() {
            let (x, y, w, h) = self.split_sequence[i];
            let avg = self.average_color(x, y, w, h);
            for py in y..(y + h).min(out.height()) {
                for px in x..(x + w).min(out.width()) {
                    out.put_pixel(px, py, avg);
                }
            }
        }
    }
}
