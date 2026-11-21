use image::{Rgba, RgbaImage};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{drawing, PuzzleKind, PuzzleSimulation};

// ============================================================================
// 1. JIGSAW CLASSIC
// ============================================================================

pub struct JigsawPuzzle {
    original: RgbaImage,
    cols: u32,
    rows: u32,
    scatter_positions: Vec<(f32, f32, f32)>, // (dx, dy, rot)
    solve_order: Vec<usize>,
}

impl JigsawPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let cols = (complexity / 2 + 3).clamp(3, 12);
        let rows = (complexity / 2 + 3).clamp(3, 12);
        let total = (cols * rows) as usize;

        let mut scatter_positions = Vec::with_capacity(total);
        let w = original.width() as f32;
        let h = original.height() as f32;

        for _ in 0..total {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(0.5..1.2) * (w + h) * 0.45;
            let dx = angle.cos() * dist;
            let dy = angle.sin() * dist;
            let rot = rng.gen_range(-1.2..1.2);
            scatter_positions.push((dx, dy, rot));
        }

        let mut solve_order: Vec<usize> = (0..total).collect();
        // Border pieces first, then center
        solve_order.sort_by_key(|&idx| {
            let r = idx as u32 / cols;
            let c = idx as u32 % cols;
            let is_border = r == 0 || r == rows - 1 || c == 0 || c == cols - 1;
            if is_border { 0 } else { 1 }
        });

        Self {
            original: original.clone(),
            cols,
            rows,
            scatter_positions,
            solve_order,
        }
    }
}

impl PuzzleSimulation for JigsawPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::JigsawClassic
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let placed = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Jigsaw: {} / {} pieces locked", placed, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();
        let pw = w as f32 / self.cols as f32;
        let ph = h as f32 / self.rows as f32;

        // Dark textured mat background
        for y in 0..h {
            for x in 0..w {
                let wood = 18 + (((x * 7 + y * 13) % 9) as u8);
                out.put_pixel(x, y, Rgba([wood, wood + 2, wood + 6, 255]));
            }
        }

        // Draw faint guide grid
        for c in 0..=self.cols {
            let x = (c as f32 * pw) as i32;
            drawing::draw_line(out, x, 0, x, h as i32 - 1, Rgba([40, 44, 55, 255]), 1);
        }
        for r in 0..=self.rows {
            let y = (r as f32 * ph) as i32;
            drawing::draw_line(out, 0, y, w as i32 - 1, y, Rgba([40, 44, 55, 255]), 1);
        }

        let total_pieces = self.solve_order.len();
        let target_f = progress * total_pieces as f32;

        for (order_idx, &piece_idx) in self.solve_order.iter().enumerate() {
            let r = piece_idx as u32 / self.cols;
            let c = piece_idx as u32 % self.cols;

            let target_cx = (c as f32 + 0.5) * pw;
            let target_cy = (r as f32 + 0.5) * ph;

            let (scatter_dx, scatter_dy, scatter_rot) = self.scatter_positions[piece_idx];
            let start_cx = target_cx + scatter_dx;
            let start_cy = target_cy + scatter_dy;

            let piece_p = (target_f - order_idx as f32).clamp(0.0, 1.0);
            let ease = piece_p * piece_p * (3.0 - 2.0 * piece_p);

            let cur_cx = start_cx + (target_cx - start_cx) * ease;
            let cur_cy = start_cy + (target_cy - start_cy) * ease;
            let cur_rot = scatter_rot * (1.0 - ease);

            // Blit piece
            let cos_r = cur_rot.cos();
            let sin_r = cur_rot.sin();

            let _half_w = pw * 0.5;
            let _half_h = ph * 0.5;

            // Sample piece interior
            let src_x0 = (c as f32 * pw) as u32;
            let src_y0 = (r as f32 * ph) as u32;
            let src_x1 = ((c + 1) as f32 * pw).min(w as f32) as u32;
            let src_y1 = ((r + 1) as f32 * ph).min(h as f32) as u32;

            if ease >= 0.999 && cur_rot.abs() < 0.001 {
                // Pixel-perfect blit when locked
                for py in src_y0..src_y1 {
                    for px in src_x0..src_x1 {
                        out.put_pixel(px, py, *self.original.get_pixel(px, py));
                    }
                }
            } else {
                for py in src_y0..src_y1 {
                    let local_y = py as f32 - target_cy;
                    for px in src_x0..src_x1 {
                        let local_x = px as f32 - target_cx;

                        let rot_x = local_x * cos_r - local_y * sin_r;
                        let rot_y = local_x * sin_r + local_y * cos_r;

                        let dst_x = (cur_cx + rot_x).round() as i32;
                        let dst_y = (cur_cy + rot_y).round() as i32;

                        if dst_x >= 0 && dst_x < w as i32 && dst_y >= 0 && dst_y < h as i32 {
                            let mut color = *self.original.get_pixel(px, py);
                            if ease > 0.001 && ease < 0.999 {
                                // Add slight brightness boost while flying
                                color.0[0] = color.0[0].saturating_add(15);
                                color.0[1] = color.0[1].saturating_add(15);
                                color.0[2] = color.0[2].saturating_add(25);
                            }
                            out.put_pixel(dst_x as u32, dst_y as u32, color);
                        }
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();
        let pw = w as f32 / self.cols as f32;
        let ph = h as f32 / self.rows as f32;

        if progress <= 0.001 {
            *out = self.original.clone();
            return;
        }

        // Dark textured mat background
        for y in 0..h {
            for x in 0..w {
                let wood = 18 + (((x * 7 + y * 13) % 9) as u8);
                out.put_pixel(x, y, Rgba([wood, wood + 2, wood + 6, 255]));
            }
        }

        // Faint guide grid
        for c in 0..=self.cols {
            let x = (c as f32 * pw) as i32;
            drawing::draw_line(out, x, 0, x, h as i32 - 1, Rgba([40, 44, 55, 255]), 1);
        }
        for r in 0..=self.rows {
            let y = (r as f32 * ph) as i32;
            drawing::draw_line(out, 0, y, w as i32 - 1, y, Rgba([40, 44, 55, 255]), 1);
        }

        let total_pieces = self.solve_order.len();
        let target_f = progress * total_pieces as f32;

        // Disassembly delay: center pieces lift off first
        let mut order_by_center: Vec<(usize, f32)> = self.solve_order.iter().map(|&p| {
            let r = p as u32 / self.cols;
            let c = p as u32 % self.cols;
            let cx = (c as f32 + 0.5) * pw;
            let cy = (r as f32 + 0.5) * ph;
            let d = (cx - w as f32 * 0.5).hypot(cy - h as f32 * 0.5);
            (p, d)
        }).collect();
        order_by_center.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut piece_delay = vec![0.0f32; total_pieces];
        for (rank, &(p, _)) in order_by_center.iter().enumerate() {
            piece_delay[p] = rank as f32;
        }

        // Iterate in solve_order so stacking order matches initial scramble perfectly
        for &piece_idx in &self.solve_order {
            let r = piece_idx as u32 / self.cols;
            let c = piece_idx as u32 % self.cols;

            let solved_cx = (c as f32 + 0.5) * pw;
            let solved_cy = (r as f32 + 0.5) * ph;

            let (scatter_dx, scatter_dy, scatter_rot) = self.scatter_positions[piece_idx];
            let target_cx = solved_cx + scatter_dx;
            let target_cy = solved_cy + scatter_dy;

            let delay = piece_delay[piece_idx];
            let piece_p = (target_f - delay).clamp(0.0, 1.0);
            let ease = piece_p * piece_p * (3.0 - 2.0 * piece_p);

            // Parabolic loft arc perpendicular to motion
            let loft = 35.0 * (piece_p * (1.0 - piece_p) * 4.0);
            let cur_cx = solved_cx + (target_cx - solved_cx) * ease;
            let cur_cy = solved_cy + (target_cy - solved_cy) * ease - loft;
            // Secondary spin while in flight
            let cur_rot = scatter_rot * ease + (piece_p * (1.0 - piece_p) * 1.5);

            let cos_r = cur_rot.cos();
            let sin_r = cur_rot.sin();

            let src_x0 = (c as f32 * pw) as u32;
            let src_y0 = (r as f32 * ph) as u32;
            let src_x1 = ((c + 1) as f32 * pw).min(w as f32) as u32;
            let src_y1 = ((r + 1) as f32 * ph).min(h as f32) as u32;

            if ease <= 0.001 {
                for py in src_y0..src_y1 {
                    for px in src_x0..src_x1 {
                        out.put_pixel(px, py, *self.original.get_pixel(px, py));
                    }
                }
            } else {
                for py in src_y0..src_y1 {
                    let local_y = py as f32 - solved_cy;
                    for px in src_x0..src_x1 {
                        let local_x = px as f32 - solved_cx;

                        let rot_x = local_x * cos_r - local_y * sin_r;
                        let rot_y = local_x * sin_r + local_y * cos_r;

                        let dst_x = (cur_cx + rot_x).round() as i32;
                        let dst_y = (cur_cy + rot_y).round() as i32;

                        if dst_x >= 0 && dst_x < w as i32 && dst_y >= 0 && dst_y < h as i32 {
                            let mut color = *self.original.get_pixel(px, py);
                            if ease > 0.001 && ease < 0.999 {
                                color.0[0] = color.0[0].saturating_add(20);
                                color.0[1] = color.0[1].saturating_add(10);
                            }
                            out.put_pixel(dst_x as u32, dst_y as u32, color);
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// 2. SLIDING GRID (15-PUZZLE)
// ============================================================================

pub struct SlidingGridPuzzle {
    original: RgbaImage,
    grid_size: usize,
    solution_path: Vec<(usize, usize)>, // sequence of (from_pos, to_pos)
}

impl SlidingGridPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_size = (complexity as usize / 3 + 3).clamp(3, 8);
        let total = grid_size * grid_size;

        // Scramble by legal moves from solved state
        let scramble_moves = (complexity as usize * 15 + 20).clamp(25, 250);
        let mut empty = total - 1;
        let mut scramble_history = Vec::new();

        let mut prev_empty = total;
        for _ in 0..scramble_moves {
            let er = empty / grid_size;
            let ec = empty % grid_size;
            let mut neighbors = Vec::new();

            if er > 0 { neighbors.push(empty - grid_size); }
            if er + 1 < grid_size { neighbors.push(empty + grid_size); }
            if ec > 0 { neighbors.push(empty - 1); }
            if ec + 1 < grid_size { neighbors.push(empty + 1); }

            // Prefer not to undo immediate previous move
            let valid: Vec<usize> = neighbors.clone().into_iter().filter(|&p| p != prev_empty).collect();
            let chosen = if valid.is_empty() {
                neighbors[0]
            } else {
                *valid.choose(&mut rng).unwrap()
            };

            scramble_history.push((chosen, empty));
            prev_empty = empty;
            empty = chosen;
        }

        // The solving path is the exact reverse of scramble moves!
        let solution_path: Vec<(usize, usize)> = scramble_history
            .into_iter()
            .rev()
            .map(|(from, to)| (to, from))
            .collect();

        Self {
            original: original.clone(),
            grid_size,
            solution_path,
        }
    }

    fn state_at_step(&self, step: usize) -> Vec<usize> {
        let total = self.grid_size * self.grid_size;
        let mut board: Vec<usize> = (0..total).collect();

        // Apply all moves up to step
        // Replay scramble
        for &(to, from) in &self.solution_path {
            board.swap(to, from);
        }

        // Now replay the solution up to `step`
        for i in 0..step.min(self.solution_path.len()) {
            let (from, to) = self.solution_path[i];
            board.swap(from, to);
        }

        board
    }
}

impl PuzzleSimulation for SlidingGridPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::SlidingGrid
    }

    fn total_steps(&self) -> usize {
        self.solution_path.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let step = (progress * self.total_steps() as f32).floor() as usize;
        format!("Sliding Grid: move {} / {}", step.min(self.total_steps()), self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width();
        let h = self.original.height();
        let n = self.grid_size;
        let tw = w as f32 / n as f32;
        let th = h as f32 / n as f32;

        let board = self.state_at_step(step_idx);
        let empty_tile_id = n * n - 1;

        // Background
        for px in out.pixels_mut() {
            *px = Rgba([15, 17, 22, 255]);
        }

        // Determine moving tile if in transition
        let (moving_from, moving_to) = if step_idx < total {
            self.solution_path[step_idx]
        } else {
            (0, 0)
        };

        for pos in 0..(n * n) {
            let tile = board[pos];
            if tile == empty_tile_id && progress < 0.999 {
                continue;
            }

            let dest_r = pos / n;
            let dest_c = pos % n;

            let (render_x, render_y) = if pos == moving_from && step_idx < total {
                let to_r = moving_to / n;
                let to_c = moving_to % n;
                let x0 = dest_c as f32 * tw;
                let y0 = dest_r as f32 * th;
                let x1 = to_c as f32 * tw;
                let y1 = to_r as f32 * th;
                (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease)
            } else {
                (dest_c as f32 * tw, dest_r as f32 * th)
            };

            // Tile source coordinates
            let src_r = tile / n;
            let src_c = tile % n;
            let sx0 = (src_c as f32 * tw) as u32;
            let sy0 = (src_r as f32 * th) as u32;

            let gap = if progress > 0.999 { 0 } else { 2 };
            let render_xi = render_x.round() as i32 + gap;
            let render_yi = render_y.round() as i32 + gap;
            let tile_w = (tw as i32 - gap * 2).max(1) as u32;
            let tile_h = (th as i32 - gap * 2).max(1) as u32;

            for y in 0..tile_h {
                let dy = render_yi + y as i32;
                if dy < 0 || dy >= h as i32 { continue; }
                let sy = sy0 + y;
                if sy >= h { continue; }

                for x in 0..tile_w {
                    let dx = render_xi + x as i32;
                    if dx < 0 || dx >= w as i32 { continue; }
                    let sx = sx0 + x;
                    if sx >= w { continue; }

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
        let n = self.grid_size;
        let tw = w as f32 / n as f32;
        let th = h as f32 / n as f32;

        let reverse_step = total.saturating_sub(step_idx);
        let board = self.state_at_step(reverse_step);
        let active_move = if step_idx < total && reverse_step > 0 {
            Some(self.solution_path[reverse_step - 1])
        } else {
            None
        };
        let empty_tile_id = n * n - 1;

        for px in out.pixels_mut() {
            *px = Rgba([15, 17, 22, 255]);
        }

        for pos in 0..(n * n) {
            let tile = board[pos];
            if tile == empty_tile_id && progress > 0.001 {
                continue;
            }
            let r = pos / n;
            let c = pos % n;

            let (render_x, render_y) = match active_move {
                Some((from, to)) if pos == to => {
                    let from_r = from / n;
                    let from_c = from % n;
                    let x0 = c as f32 * tw;
                    let y0 = r as f32 * th;
                    let x1 = from_c as f32 * tw;
                    let y1 = from_r as f32 * th;
                    let loft = 15.0 * (sub_p * (1.0 - sub_p) * 4.0);
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease - loft)
                }
                _ => (c as f32 * tw, r as f32 * th),
            };

            let src_r = tile / n;
            let src_c = tile % n;
            let sx0 = (src_c as f32 * tw) as u32;
            let sy0 = (src_r as f32 * th) as u32;

            let gap = if progress <= 0.001 { 0 } else { 2 };
            let rxi = render_x.round() as i32 + gap;
            let ryi = render_y.round() as i32 + gap;
            let tile_w = (tw as i32 - gap * 2).max(1) as u32;
            let tile_h = (th as i32 - gap * 2).max(1) as u32;

            for y in 0..tile_h {
                let dy = ryi + y as i32;
                if dy < 0 || dy >= h as i32 { continue; }
                let sy = sy0 + y;
                if sy >= h { continue; }

                for x in 0..tile_w {
                    let dx = rxi + x as i32;
                    if dx < 0 || dx >= w as i32 { continue; }
                    let sx = sx0 + x;
                    if sx >= w { continue; }

                    out.put_pixel(dx as u32, dy as u32, *self.original.get_pixel(sx, sy));
                }
            }
        }
    }
}

// ============================================================================
// 3. TILE SWAP PERMUTATION
// ============================================================================

pub struct TileSwapPuzzle {
    original: RgbaImage,
    grid_size: usize,
    swaps: Vec<(usize, usize)>,
}

impl TileSwapPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_size = (complexity as usize / 2 + 3).clamp(3, 10);
        let total = grid_size * grid_size;

        // Scramble permutation
        let mut perm: Vec<usize> = (0..total).collect();
        perm.shuffle(&mut rng);

        // Find swap sequence to sort perm back to identity
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
            grid_size,
            swaps,
        }
    }

    fn state_at_step(&self, step: usize) -> Vec<usize> {
        let total = self.grid_size * self.grid_size;
        let mut board: Vec<usize> = (0..total).collect();
        // Reconstruct initial perm by undoing swaps
        for &(a, b) in self.swaps.iter().rev() {
            board.swap(a, b);
        }
        // Apply forward up to step
        for i in 0..step.min(self.swaps.len()) {
            let (a, b) = self.swaps[i];
            board.swap(a, b);
        }
        board
    }
}

impl PuzzleSimulation for TileSwapPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::TileSwap
    }

    fn total_steps(&self) -> usize {
        self.swaps.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let step = (progress * self.total_steps() as f32).floor() as usize;
        format!("Tile Swap: {} / {} swaps completed", step.min(self.total_steps()), self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width();
        let h = self.original.height();
        let n = self.grid_size;
        let tw = w as f32 / n as f32;
        let th = h as f32 / n as f32;

        let board = self.state_at_step(step_idx);
        let active_swap = if step_idx < total { Some(self.swaps[step_idx]) } else { None };

        // Background
        for px in out.pixels_mut() {
            *px = Rgba([12, 14, 20, 255]);
        }

        for pos in 0..(n * n) {
            let tile = board[pos];
            let r = pos / n;
            let c = pos % n;

            let (render_x, render_y) = match active_swap {
                Some((a, b)) if pos == a => {
                    let br = b / n;
                    let bc = b % n;
                    let x0 = c as f32 * tw;
                    let y0 = r as f32 * th;
                    let x1 = bc as f32 * tw;
                    let y1 = br as f32 * th;
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease)
                }
                Some((a, b)) if pos == b => {
                    let ar = a / n;
                    let ac = a % n;
                    let x0 = c as f32 * tw;
                    let y0 = r as f32 * th;
                    let x1 = ac as f32 * tw;
                    let y1 = ar as f32 * th;
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease)
                }
                _ => (c as f32 * tw, r as f32 * th),
            };

            let src_r = tile / n;
            let src_c = tile % n;
            let sx0 = (src_c as f32 * tw) as u32;
            let sy0 = (src_r as f32 * th) as u32;

            let gap = if progress >= 0.999 { 0 } else { 1 };
            let rxi = render_x.round() as i32 + gap;
            let ryi = render_y.round() as i32 + gap;
            let tile_w = (tw as i32 - gap * 2).max(1) as u32;
            let tile_h = (th as i32 - gap * 2).max(1) as u32;

            for y in 0..tile_h {
                let dy = ryi + y as i32;
                if dy < 0 || dy >= h as i32 { continue; }
                let sy = sy0 + y;
                if sy >= h { continue; }

                for x in 0..tile_w {
                    let dx = rxi + x as i32;
                    if dx < 0 || dx >= w as i32 { continue; }
                    let sx = sx0 + x;
                    if sx >= w { continue; }

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
        let n = self.grid_size;
        let tw = w as f32 / n as f32;
        let th = h as f32 / n as f32;

        let reverse_step = total.saturating_sub(step_idx);
        let board = self.state_at_step(reverse_step);
        let active_swap = if step_idx < total && reverse_step > 0 {
            Some(self.swaps[reverse_step - 1])
        } else {
            None
        };

        for px in out.pixels_mut() {
            *px = Rgba([12, 14, 20, 255]);
        }

        for pos in 0..(n * n) {
            let tile = board[pos];
            let r = pos / n;
            let c = pos % n;

            let (render_x, render_y) = match active_swap {
                Some((a, b)) if pos == a => {
                    let br = b / n;
                    let bc = b % n;
                    let x0 = c as f32 * tw;
                    let y0 = r as f32 * th;
                    let x1 = bc as f32 * tw;
                    let y1 = br as f32 * th;
                    let loft = 25.0 * (sub_p * (1.0 - sub_p) * 4.0);
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease - loft)
                }
                Some((a, b)) if pos == b => {
                    let ar = a / n;
                    let ac = a % n;
                    let x0 = c as f32 * tw;
                    let y0 = r as f32 * th;
                    let x1 = ac as f32 * tw;
                    let y1 = ar as f32 * th;
                    let loft = 25.0 * (sub_p * (1.0 - sub_p) * 4.0);
                    (x0 + (x1 - x0) * ease, y0 + (y1 - y0) * ease + loft)
                }
                _ => (c as f32 * tw, r as f32 * th),
            };

            let src_r = tile / n;
            let src_c = tile % n;
            let sx0 = (src_c as f32 * tw) as u32;
            let sy0 = (src_r as f32 * th) as u32;

            let gap = if progress <= 0.001 { 0 } else { 1 };
            let rxi = render_x.round() as i32 + gap;
            let ryi = render_y.round() as i32 + gap;
            let tile_w = (tw as i32 - gap * 2).max(1) as u32;
            let tile_h = (th as i32 - gap * 2).max(1) as u32;

            for y in 0..tile_h {
                let dy = ryi + y as i32;
                if dy < 0 || dy >= h as i32 { continue; }
                let sy = sy0 + y;
                if sy >= h { continue; }

                for x in 0..tile_w {
                    let dx = rxi + x as i32;
                    if dx < 0 || dx >= w as i32 { continue; }
                    let sx = sx0 + x;
                    if sx >= w { continue; }

                    out.put_pixel(dx as u32, dy as u32, *self.original.get_pixel(sx, sy));
                }
            }
        }
    }
}

// ============================================================================
// 4. ROTATING TILES GRID
// ============================================================================

pub struct RotatingTilesPuzzle {
    original: RgbaImage,
    grid_size: usize,
    initial_rotations: Vec<u8>, // 1, 2, or 3 (90, 180, 270 deg)
    solve_order: Vec<usize>,
}

impl RotatingTilesPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_size = (complexity as usize / 2 + 3).clamp(3, 12);
        let total = grid_size * grid_size;

        let mut initial_rotations = Vec::with_capacity(total);
        for _ in 0..total {
            initial_rotations.push(rng.gen_range(1..=3));
        }

        let mut solve_order: Vec<usize> = (0..total).collect();
        solve_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            grid_size,
            initial_rotations,
            solve_order,
        }
    }
}

impl PuzzleSimulation for RotatingTilesPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::RotatingTiles
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let solved = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Rotating Tiles: {} / {} oriented", solved, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let target_f = progress * total as f32;

        let w = self.original.width();
        let h = self.original.height();
        let n = self.grid_size;
        let tw = w as f32 / n as f32;
        let th = h as f32 / n as f32;

        for (order_idx, &tile_idx) in self.solve_order.iter().enumerate() {
            let r = tile_idx / n;
            let c = tile_idx % n;

            let initial_rot = self.initial_rotations[tile_idx] as f32 * std::f32::consts::FRAC_PI_2;
            let tile_p = (target_f - order_idx as f32).clamp(0.0, 1.0);
            let ease = tile_p * tile_p * (3.0 - 2.0 * tile_p);
            let angle = initial_rot * (1.0 - ease);

            let cx = (c as f32 + 0.5) * tw;
            let cy = (r as f32 + 0.5) * th;

            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let x0 = (c as f32 * tw) as u32;
            let y0 = (r as f32 * th) as u32;
            let x1 = ((c + 1) as f32 * tw).min(w as f32) as u32;
            let y1 = ((r + 1) as f32 * th).min(h as f32) as u32;

            if angle.abs() < 0.001 {
                for py in y0..y1 {
                    for px in x0..x1 {
                        out.put_pixel(px, py, *self.original.get_pixel(px, py));
                    }
                }
            } else {
                for py in y0..y1 {
                    let dy = py as f32 - cy;
                    for px in x0..x1 {
                        let dx = px as f32 - cx;

                        // Inverse rotation to sample source
                        let rx = dx * cos_a + dy * sin_a;
                        let ry = -dx * sin_a + dy * cos_a;

                        let src_x = cx + rx;
                        let src_y = cy + ry;

                        let color = drawing::sample_bilinear(&self.original, src_x, src_y);
                        out.put_pixel(px, py, color);
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let target_f = progress * total as f32;

        let w = self.original.width();
        let h = self.original.height();
        let n = self.grid_size;
        let tw = w as f32 / n as f32;
        let th = h as f32 / n as f32;

        // Rollback disassembles in reversed order with dynamic wave
        for (order_idx, &tile_idx) in self.solve_order.iter().rev().enumerate() {
            let r = tile_idx / n;
            let c = tile_idx % n;

            let target_rot = self.initial_rotations[tile_idx] as f32 * std::f32::consts::FRAC_PI_2;
            let tile_p = (target_f - order_idx as f32).clamp(0.0, 1.0);
            let ease = tile_p * tile_p * (3.0 - 2.0 * tile_p);
            // Spin from 0 up to target_rot, adding extra rotation spin during flight
            let extra_spin = tile_p * (1.0 - tile_p) * std::f32::consts::PI;
            let angle = target_rot * ease + extra_spin;

            let cx = (c as f32 + 0.5) * tw;
            let cy = (r as f32 + 0.5) * th;

            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let x0 = (c as f32 * tw) as u32;
            let y0 = (r as f32 * th) as u32;
            let x1 = ((c + 1) as f32 * tw).min(w as f32) as u32;
            let y1 = ((r + 1) as f32 * th).min(h as f32) as u32;

            if angle.abs() < 0.001 {
                for py in y0..y1 {
                    for px in x0..x1 {
                        out.put_pixel(px, py, *self.original.get_pixel(px, py));
                    }
                }
            } else {
                for py in y0..y1 {
                    let dy = py as f32 - cy;
                    for px in x0..x1 {
                        let dx = px as f32 - cx;

                        let rx = dx * cos_a + dy * sin_a;
                        let ry = -dx * sin_a + dy * cos_a;

                        let src_x = cx + rx;
                        let src_y = cy + ry;

                        let color = drawing::sample_bilinear(&self.original, src_x, src_y);
                        out.put_pixel(px, py, color);
                    }
                }
            }
        }
    }
}

// ============================================================================
// 5. CONCENTRIC ROTATING RINGS
// ============================================================================

pub struct RotatingRingsPuzzle {
    original: RgbaImage,
    ring_count: usize,
    ring_offsets: Vec<f32>,
}

impl RotatingRingsPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let ring_count = (complexity as usize + 3).clamp(4, 24);

        let mut ring_offsets = Vec::with_capacity(ring_count);
        for i in 0..ring_count {
            let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
            let angle = dir * rng.gen_range(0.8..std::f32::consts::PI * 1.5);
            ring_offsets.push(angle);
        }

        Self {
            original: original.clone(),
            ring_count,
            ring_offsets,
        }
    }
}

impl PuzzleSimulation for RotatingRingsPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::RotatingRings
    }

    fn total_steps(&self) -> usize {
        self.ring_count
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Rotating Rings: {:.0}% aligned", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let max_r = cx.hypot(cy);

        let ring_w = max_r / self.ring_count as f32;

        for y in 0..h {
            let dy = y as f32 - cy;
            for x in 0..w {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let ring_idx = ((r / ring_w).floor() as usize).min(self.ring_count - 1);

                // Outside to inside solve or staged
                let ring_p = ((progress * self.ring_count as f32) - ring_idx as f32).clamp(0.0, 1.0);
                let ease = ring_p * ring_p * (3.0 - 2.0 * ring_p);
                let cur_offset = self.ring_offsets[ring_idx] * (1.0 - ease);

                if cur_offset.abs() < 0.001 {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                } else {
                    let theta = dy.atan2(dx) - cur_offset;
                    let src_x = cx + r * theta.cos();
                    let src_y = cy + r * theta.sin();
                    let color = drawing::sample_bilinear(&self.original, src_x, src_y);
                    out.put_pixel(x, y, color);
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let w = self.original.width();
        let h = self.original.height();
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let max_r = cx.hypot(cy);

        let ring_w = max_r / self.ring_count as f32;

        for y in 0..h {
            let dy = y as f32 - cy;
            for x in 0..w {
                let dx = x as f32 - cx;
                let r = (dx * dx + dy * dy).sqrt();
                let ring_idx = ((r / ring_w).floor() as usize).min(self.ring_count - 1);

                // Rollback disassembles inside-out (reverse ring index order)
                let rev_idx = self.ring_count - 1 - ring_idx;
                let ring_p = ((progress * self.ring_count as f32) - rev_idx as f32).clamp(0.0, 1.0);
                let ease = ring_p * ring_p * (3.0 - 2.0 * ring_p);
                // Rotate out from 0 to initial ring_offset with dynamic ripple
                let ripple = (ring_p * (1.0 - ring_p) * 0.8) * if ring_idx % 2 == 0 { 1.0 } else { -1.0 };
                let cur_offset = self.ring_offsets[ring_idx] * ease + ripple;

                if cur_offset.abs() < 0.001 {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                } else {
                    let theta = dy.atan2(dx) - cur_offset;
                    let src_x = cx + r * theta.cos();
                    let src_y = cy + r * theta.sin();
                    let color = drawing::sample_bilinear(&self.original, src_x, src_y);
                    out.put_pixel(x, y, color);
                }
            }
        }
    }
}

// ============================================================================
// 6. TOROIDAL RUBIK SHIFT
// ============================================================================

pub struct ToroidalShiftPuzzle {
    original: RgbaImage,
    grid_size: usize,
    shifts: Vec<(bool, usize, i32)>, // (is_row, index, offset)
}

impl ToroidalShiftPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_size = (complexity as usize / 2 + 3).clamp(3, 10);
        let count = (complexity as usize * 3 + 6).clamp(8, 50);

        let mut shifts = Vec::with_capacity(count);
        for _ in 0..count {
            let is_row = rng.gen_bool(0.5);
            let idx = rng.gen_range(0..grid_size);
            let offset = if rng.gen_bool(0.5) { 1 } else { -1 };
            shifts.push((is_row, idx, offset));
        }

        Self {
            original: original.clone(),
            grid_size,
            shifts,
        }
    }
}

impl PuzzleSimulation for ToroidalShiftPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ToroidalShift
    }

    fn total_steps(&self) -> usize {
        self.shifts.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let step = (progress * self.total_steps() as f32).floor() as usize;
        format!("Toroidal Shift: move {} / {}", step.min(self.total_steps()), self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width() as f32;
        let h = self.original.height() as f32;
        let n = self.grid_size;
        let tw = w / n as f32;
        let th = h / n as f32;

        // Active shift
        let _active = if step_idx < total {
            Some(self.shifts[total - 1 - step_idx])
        } else {
            None
        };

        // Render pixel coordinates wrapping toroidally
        for y in 0..self.original.height() {
            let r = ((y as f32 / th).floor() as usize).min(n - 1);
            for x in 0..self.original.width() {
                let c = ((x as f32 / tw).floor() as usize).min(n - 1);

                // Compute total shift accumulated by remaining reverse moves
                let mut offset_x = 0.0;
                let mut offset_y = 0.0;

                for i in step_idx..total {
                    let (is_row, idx, off) = self.shifts[total - 1 - i];
                    let weight = if i == step_idx { 1.0 - ease } else { 1.0 };
                    if is_row && idx == r {
                        offset_x += (off as f32 * tw) * weight;
                    } else if !is_row && idx == c {
                        offset_y += (off as f32 * th) * weight;
                    }
                }

                let src_x = (x as f32 + offset_x).rem_euclid(w);
                let src_y = (y as f32 + offset_y).rem_euclid(h);

                let color = drawing::sample_bilinear(&self.original, src_x, src_y);
                out.put_pixel(x, y, color);
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let cur_f = progress * total as f32;
        let step_idx = (cur_f.floor() as usize).min(total);
        let sub_p = if step_idx >= total { 0.0 } else { cur_f - step_idx as f32 };
        let ease = sub_p * sub_p * (3.0 - 2.0 * sub_p);

        let w = self.original.width() as f32;
        let h = self.original.height() as f32;
        let n = self.grid_size;
        let tw = w / n as f32;
        let th = h / n as f32;

        for y in 0..self.original.height() {
            let r = ((y as f32 / th).floor() as usize).min(n - 1);
            for x in 0..self.original.width() {
                let c = ((x as f32 / tw).floor() as usize).min(n - 1);

                let mut offset_x = 0.0;
                let mut offset_y = 0.0;

                // Re-apply shifts in forward order to scramble back out
                for i in 0..step_idx.min(total) {
                    let (is_row, idx, off) = self.shifts[total - 1 - i];
                    if is_row && idx == r {
                        offset_x += off as f32 * tw;
                    } else if !is_row && idx == c {
                        offset_y += off as f32 * th;
                    }
                }
                if step_idx < total {
                    let (is_row, idx, off) = self.shifts[total - 1 - step_idx];
                    if is_row && idx == r {
                        offset_x += (off as f32 * tw) * ease;
                    } else if !is_row && idx == c {
                        offset_y += (off as f32 * th) * ease;
                    }
                }

                let src_x = (x as f32 + offset_x).rem_euclid(w);
                let src_y = (y as f32 + offset_y).rem_euclid(h);

                let color = drawing::sample_bilinear(&self.original, src_x, src_y);
                out.put_pixel(x, y, color);
            }
        }
    }
}

// ============================================================================
// 7. HEXAGONAL HONEYCOMB
// ============================================================================

pub struct HexHoneycombPuzzle {
    original: RgbaImage,
    centers: Vec<(f32, f32)>,
    scatter: Vec<(f32, f32, f32)>,
    solve_order: Vec<usize>,
    radius: f32,
}

impl HexHoneycombPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let w = original.width() as f32;
        let h = original.height() as f32;

        let radius = (60.0 - complexity as f32 * 2.0).clamp(18.0, 50.0);
        let hex_w = 3.0_f32.sqrt() * radius;
        let hex_h = 1.5 * radius;

        let mut centers = Vec::new();
        let mut row = 0;
        let mut y = radius;
        while y < h + radius {
            let offset_x = if row % 2 == 1 { hex_w * 0.5 } else { 0.0 };
            let mut x = offset_x;
            while x < w + hex_w {
                centers.push((x, y));
                x += hex_w;
            }
            y += hex_h;
            row += 1;
        }

        let total = centers.len();
        let mut scatter = Vec::with_capacity(total);
        for _ in 0..total {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(0.3..0.8) * (w + h) * 0.5;
            let rot = rng.gen_range(-1.5..1.5);
            scatter.push((ang.cos() * dist, ang.sin() * dist, rot));
        }

        let mut solve_order: Vec<usize> = (0..total).collect();
        solve_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            centers,
            scatter,
            solve_order,
            radius,
        }
    }
}

impl PuzzleSimulation for HexHoneycombPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::HexHoneycomb
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let placed = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Hex Honeycomb: {} / {} tiles locked", placed, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let target_f = progress * total as f32;

        // Background
        for px in out.pixels_mut() {
            *px = Rgba([18, 20, 26, 255]);
        }

        for (order_idx, &hex_idx) in self.solve_order.iter().enumerate() {
            let (cx, cy) = self.centers[hex_idx];
            let (sdx, sdy, srot) = self.scatter[hex_idx];

            let p = (target_f - order_idx as f32).clamp(0.0, 1.0);
            let ease = p * p * (3.0 - 2.0 * p);

            let cur_cx = cx + sdx * (1.0 - ease);
            let cur_cy = cy + sdy * (1.0 - ease);
            let cur_rot = srot * (1.0 - ease);

            let cos_r = cur_rot.cos();
            let sin_r = cur_rot.sin();

            let r = self.radius;
            let min_x = (cur_cx - r).floor() as i32;
            let max_x = (cur_cx + r).ceil() as i32;
            let min_y = (cur_cy - r).floor() as i32;
            let max_y = (cur_cy + r).ceil() as i32;

            for y in min_y..=max_y {
                if y < 0 || y >= out.height() as i32 { continue; }
                let l_y = y as f32 - cur_cy;
                for x in min_x..=max_x {
                    if x < 0 || x >= out.width() as i32 { continue; }
                    let l_x = x as f32 - cur_cx;

                    // Hexagon test in local space
                    let q2x = (l_x / (3.0_f32.sqrt() * 0.5 * r)).abs();
                    let q2y = (l_y / r).abs();
                    if q2x > 1.0 || q2y > 1.0 || (0.5 * q2x + 0.5 * q2y) > 1.0 {
                        continue;
                    }

                    // Rotate back to sample image
                    let orig_lx = l_x * cos_r + l_y * sin_r;
                    let orig_ly = -l_x * sin_r + l_y * cos_r;

                    let sx = cx + orig_lx;
                    let sy = cy + orig_ly;

                    let color = drawing::sample_bilinear(&self.original, sx, sy);
                    out.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let target_f = progress * total as f32;

        for px in out.pixels_mut() {
            *px = Rgba([18, 20, 26, 255]);
        }

        // Rollback disassembles in reversed order with loft arcs
        for (order_idx, &hex_idx) in self.solve_order.iter().rev().enumerate() {
            let (cx, cy) = self.centers[hex_idx];
            let (sdx, sdy, srot) = self.scatter[hex_idx];

            let p = (target_f - order_idx as f32).clamp(0.0, 1.0);
            let ease = p * p * (3.0 - 2.0 * p);

            let loft = 30.0 * (p * (1.0 - p) * 4.0);
            let cur_cx = cx + sdx * ease;
            let cur_cy = cy + sdy * ease - loft;
            let cur_rot = srot * ease;

            let cos_r = cur_rot.cos();
            let sin_r = cur_rot.sin();

            let r = self.radius;
            let min_x = (cur_cx - r).floor() as i32;
            let max_x = (cur_cx + r).ceil() as i32;
            let min_y = (cur_cy - r).floor() as i32;
            let max_y = (cur_cy + r).ceil() as i32;

            for y in min_y..=max_y {
                if y < 0 || y >= out.height() as i32 { continue; }
                let l_y = y as f32 - cur_cy;
                for x in min_x..=max_x {
                    if x < 0 || x >= out.width() as i32 { continue; }
                    let l_x = x as f32 - cur_cx;

                    let q2x = (l_x / (3.0_f32.sqrt() * 0.5 * r)).abs();
                    let q2y = (l_y / r).abs();
                    if q2x > 1.0 || q2y > 1.0 || (0.5 * q2x + 0.5 * q2y) > 1.0 {
                        continue;
                    }

                    let orig_lx = l_x * cos_r + l_y * sin_r;
                    let orig_ly = -l_x * sin_r + l_y * cos_r;

                    let sx = cx + orig_lx;
                    let sy = cy + orig_ly;

                    let color = drawing::sample_bilinear(&self.original, sx, sy);
                    out.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

// ============================================================================
// 8. TRIANGULAR MESH
// ============================================================================

pub struct TriangleMeshPuzzle {
    original: RgbaImage,
    triangles: Vec<[usize; 3]>,
    vertices: Vec<(f32, f32)>,
    scatter: Vec<(f32, f32, f32)>,
    solve_order: Vec<usize>,
}

impl TriangleMeshPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let cols = (complexity / 2 + 3).clamp(3, 10);
        let rows = (complexity / 2 + 3).clamp(3, 10);

        let w = original.width() as f32;
        let h = original.height() as f32;
        let dx = w / cols as f32;
        let dy = h / rows as f32;

        let mut vertices = Vec::new();
        for r in 0..=rows {
            for c in 0..=cols {
                vertices.push((c as f32 * dx, r as f32 * dy));
            }
        }

        let mut triangles = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let v0 = (r * (cols + 1) + c) as usize;
                let v1 = v0 + 1;
                let v2 = ((r + 1) * (cols + 1) + c) as usize;
                let v3 = v2 + 1;

                triangles.push([v0, v1, v2]);
                triangles.push([v1, v3, v2]);
            }
        }

        let total = triangles.len();
        let mut scatter = Vec::with_capacity(total);
        for _ in 0..total {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(0.3..0.7) * (w + h) * 0.5;
            let rot = rng.gen_range(-1.2..1.2);
            scatter.push((ang.cos() * dist, ang.sin() * dist, rot));
        }

        let mut solve_order: Vec<usize> = (0..total).collect();
        solve_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            triangles,
            vertices,
            scatter,
            solve_order,
        }
    }
}

impl PuzzleSimulation for TriangleMeshPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::TriangleMesh
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let placed = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Triangle Mesh: {} / {} triangles assembled", placed, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let target_f = progress * total as f32;

        // Background
        for px in out.pixels_mut() {
            *px = Rgba([20, 22, 28, 255]);
        }

        for (order_idx, &tri_idx) in self.solve_order.iter().enumerate() {
            let [i0, i1, i2] = self.triangles[tri_idx];
            let (v0x, v0y) = self.vertices[i0];
            let (v1x, v1y) = self.vertices[i1];
            let (v2x, v2y) = self.vertices[i2];

            let centroid_x = (v0x + v1x + v2x) / 3.0;
            let centroid_y = (v0y + v1y + v2y) / 3.0;

            let (sdx, sdy, srot) = self.scatter[tri_idx];
            let p = (target_f - order_idx as f32).clamp(0.0, 1.0);
            let ease = p * p * (3.0 - 2.0 * p);

            let cur_cx = centroid_x + sdx * (1.0 - ease);
            let cur_cy = centroid_y + sdy * (1.0 - ease);
            let cur_rot = srot * (1.0 - ease);

            let cos_r = cur_rot.cos();
            let sin_r = cur_rot.sin();

            // Transform triangle vertices
            let transform = |vx: f32, vy: f32| -> (f32, f32) {
                let lx = vx - centroid_x;
                let ly = vy - centroid_y;
                let rx = lx * cos_r - ly * sin_r;
                let ry = lx * sin_r + ly * cos_r;
                (cur_cx + rx, cur_cy + ry)
            };

            let (p0x, p0y) = transform(v0x, v0y);
            let (p1x, p1y) = transform(v1x, v1y);
            let (p2x, p2y) = transform(v2x, v2y);

            let min_x = p0x.min(p1x).min(p2x).floor() as i32;
            let max_x = p0x.max(p1x).max(p2x).ceil() as i32;
            let min_y = p0y.min(p1y).min(p2y).floor() as i32;
            let max_y = p0y.max(p1y).max(p2y).ceil() as i32;

            // Barycentric rasterizer
            let denom = (p1y - p2y) * (p0x - p2x) + (p2x - p1x) * (p0y - p2y);
            if denom.abs() < 1e-4 { continue; }

            for y in min_y..=max_y {
                if y < 0 || y >= out.height() as i32 { continue; }
                let py = y as f32;
                for x in min_x..=max_x {
                    if x < 0 || x >= out.width() as i32 { continue; }
                    let px = x as f32;

                    let w0 = ((p1y - p2y) * (px - p2x) + (p2x - p1x) * (py - p2y)) / denom;
                    let w1 = ((p2y - p0y) * (px - p2x) + (p0x - p2x) * (py - p2y)) / denom;
                    let w2 = 1.0 - w0 - w1;

                    if w0 >= -0.01 && w1 >= -0.01 && w2 >= -0.01 {
                        let sx = w0 * v0x + w1 * v1x + w2 * v2x;
                        let sy = w0 * v0y + w1 * v1y + w2 * v2y;
                        let color = drawing::sample_bilinear(&self.original, sx, sy);
                        out.put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage) {
        let total = self.total_steps();
        let target_f = progress * total as f32;

        for px in out.pixels_mut() {
            *px = Rgba([20, 22, 28, 255]);
        }

        // Rollback disassembles in reversed triangle order with parabolic arcs
        for (order_idx, &tri_idx) in self.solve_order.iter().enumerate() {
            let [i0, i1, i2] = self.triangles[tri_idx];
            let (v0x, v0y) = self.vertices[i0];
            let (v1x, v1y) = self.vertices[i1];
            let (v2x, v2y) = self.vertices[i2];

            let centroid_x = (v0x + v1x + v2x) / 3.0;
            let centroid_y = (v0y + v1y + v2y) / 3.0;

            let (sdx, sdy, srot) = self.scatter[tri_idx];
            let delay = (total.saturating_sub(1 + order_idx)) as f32;
            let p = (target_f - delay).clamp(0.0, 1.0);
            let ease = p * p * (3.0 - 2.0 * p);

            let loft = 28.0 * (p * (1.0 - p) * 4.0);
            let cur_cx = centroid_x + sdx * ease;
            let cur_cy = centroid_y + sdy * ease - loft;
            let cur_rot = srot * ease;

            let cos_r = cur_rot.cos();
            let sin_r = cur_rot.sin();

            let transform = |vx: f32, vy: f32| -> (f32, f32) {
                let lx = vx - centroid_x;
                let ly = vy - centroid_y;
                let rx = lx * cos_r - ly * sin_r;
                let ry = lx * sin_r + ly * cos_r;
                (cur_cx + rx, cur_cy + ry)
            };

            let (p0x, p0y) = transform(v0x, v0y);
            let (p1x, p1y) = transform(v1x, v1y);
            let (p2x, p2y) = transform(v2x, v2y);

            let min_x = p0x.min(p1x).min(p2x).floor() as i32;
            let max_x = p0x.max(p1x).max(p2x).ceil() as i32;
            let min_y = p0y.min(p1y).min(p2y).floor() as i32;
            let max_y = p0y.max(p1y).max(p2y).ceil() as i32;

            let denom = (p1y - p2y) * (p0x - p2x) + (p2x - p1x) * (p0y - p2y);
            if denom.abs() < 1e-4 { continue; }

            for y in min_y..=max_y {
                if y < 0 || y >= out.height() as i32 { continue; }
                let py = y as f32;
                for x in min_x..=max_x {
                    if x < 0 || x >= out.width() as i32 { continue; }
                    let px = x as f32;

                    let w0 = ((p1y - p2y) * (px - p2x) + (p2x - p1x) * (py - p2y)) / denom;
                    let w1 = ((p2y - p0y) * (px - p2x) + (p0x - p2x) * (py - p2y)) / denom;
                    let w2 = 1.0 - w0 - w1;

                    if w0 >= -0.01 && w1 >= -0.01 && w2 >= -0.01 {
                        let sx = w0 * v0x + w1 * v1x + w2 * v2x;
                        let sy = w0 * v0y + w1 * v1y + w2 * v2y;
                        let color = drawing::sample_bilinear(&self.original, sx, sy);
                        out.put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }
}
