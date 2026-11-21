use image::{Rgba, RgbaImage};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{drawing, PuzzleKind, PuzzleSimulation};

// ============================================================================
// 18. MAZE SPANNING-TREE REVEAL
// ============================================================================

pub struct MazeSolverPuzzle {
    original: RgbaImage,
    grid_w: usize,
    grid_h: usize,
    solution_path: Vec<(usize, usize)>,
    visited_cells: Vec<(usize, usize)>,
}

impl MazeSolverPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_w = (complexity as usize * 3 + 12).clamp(15, 40);
        let grid_h = (complexity as usize * 3 + 12).clamp(15, 40);

        // Generate maze via randomized DFS
        let mut walls_horiz = vec![vec![true; grid_w]; grid_h + 1];
        let mut walls_vert = vec![vec![true; grid_w + 1]; grid_h];
        let mut visited = vec![vec![false; grid_w]; grid_h];

        let mut stack = vec![(0, 0)];
        visited[0][0] = true;

        while let Some(&(cx, cy)) = stack.last() {
            let mut neighbors = Vec::new();
            if cy > 0 && !visited[cy - 1][cx] { neighbors.push((cx, cy - 1, 0)); } // Up
            if cx + 1 < grid_w && !visited[cy][cx + 1] { neighbors.push((cx + 1, cy, 1)); } // Right
            if cy + 1 < grid_h && !visited[cy + 1][cx] { neighbors.push((cx, cy + 1, 2)); } // Down
            if cx > 0 && !visited[cy][cx - 1] { neighbors.push((cx - 1, cy, 3)); } // Left

            if let Some(&(nx, ny, dir)) = neighbors.choose(&mut rng) {
                visited[ny][nx] = true;
                match dir {
                    0 => walls_horiz[cy][cx] = false,
                    1 => walls_vert[cy][cx + 1] = false,
                    2 => walls_horiz[cy + 1][cx] = false,
                    3 => walls_vert[cy][cx] = false,
                    _ => {}
                }
                stack.push((nx, ny));
            } else {
                stack.pop();
            }
        }

        // BFS to find shortest path from (0,0) to (grid_w-1, grid_h-1)
        let mut q = std::collections::VecDeque::new();
        let mut parent = vec![vec![None; grid_w]; grid_h];
        let mut bfs_visited = vec![vec![false; grid_w]; grid_h];
        let mut visited_cells = Vec::new();

        q.push_back((0, 0));
        bfs_visited[0][0] = true;

        while let Some((cx, cy)) = q.pop_front() {
            visited_cells.push((cx, cy));
            if cx == grid_w - 1 && cy == grid_h - 1 {
                break;
            }

            // Up
            if cy > 0 && !walls_horiz[cy][cx] && !bfs_visited[cy - 1][cx] {
                bfs_visited[cy - 1][cx] = true;
                parent[cy - 1][cx] = Some((cx, cy));
                q.push_back((cx, cy - 1));
            }
            // Right
            if cx + 1 < grid_w && !walls_vert[cy][cx + 1] && !bfs_visited[cy][cx + 1] {
                bfs_visited[cy][cx + 1] = true;
                parent[cy][cx + 1] = Some((cx, cy));
                q.push_back((cx + 1, cy));
            }
            // Down
            if cy + 1 < grid_h && !walls_horiz[cy + 1][cx] && !bfs_visited[cy + 1][cx] {
                bfs_visited[cy + 1][cx] = true;
                parent[cy + 1][cx] = Some((cx, cy));
                q.push_back((cx, cy + 1));
            }
            // Left
            if cx > 0 && !walls_vert[cy][cx] && !bfs_visited[cy][cx - 1] {
                bfs_visited[cy][cx - 1] = true;
                parent[cy][cx - 1] = Some((cx, cy));
                q.push_back((cx - 1, cy));
            }
        }

        let mut solution_path = Vec::new();
        let mut cur = (grid_w - 1, grid_h - 1);
        while let Some(p) = parent[cur.1][cur.0] {
            solution_path.push(cur);
            cur = p;
        }
        solution_path.push((0, 0));
        solution_path.reverse();

        Self {
            original: original.clone(),
            grid_w,
            grid_h,
            solution_path,
            visited_cells,
        }
    }
}

impl PuzzleSimulation for MazeSolverPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::MazeSolver
    }

    fn total_steps(&self) -> usize {
        self.visited_cells.len() + self.solution_path.len()
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Maze Solver: {:.0}% explored & solved", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let cw = w as f32 / self.grid_w as f32;
        let ch = h as f32 / self.grid_h as f32;

        let total_explore = self.visited_cells.len();
        let total_solve = self.solution_path.len();
        let total = total_explore + total_solve;

        let cur_step = (progress * total as f32).floor() as usize;

        // Dim base image
        for y in 0..h {
            for x in 0..w {
                let mut c = *self.original.get_pixel(x, y);
                c.0[0] = (c.0[0] as f32 * 0.12) as u8;
                c.0[1] = (c.0[1] as f32 * 0.12) as u8;
                c.0[2] = (c.0[2] as f32 * 0.18) as u8;
                out.put_pixel(x, y, c);
            }
        }

        // Illuminate visited cells
        let visited_count = cur_step.min(total_explore);
        for i in 0..visited_count {
            let (gx, gy) = self.visited_cells[i];
            let x0 = (gx as f32 * cw) as u32;
            let y0 = (gy as f32 * ch) as u32;
            let x1 = ((gx + 1) as f32 * cw).min(w as f32) as u32;
            let y1 = ((gy + 1) as f32 * ch).min(h as f32) as u32;

            for y in y0..y1 {
                for x in x0..x1 {
                    let mut c = *self.original.get_pixel(x, y);
                    c.0[0] = (c.0[0] as f32 * 0.55) as u8;
                    c.0[1] = (c.0[1] as f32 * 0.55) as u8;
                    c.0[2] = (c.0[2] as f32 * 0.65) as u8;
                    out.put_pixel(x, y, c);
                }
            }
        }

        // Full illumination along solution path
        if cur_step > total_explore {
            let path_count = (cur_step - total_explore).min(total_solve);
            for i in 0..path_count {
                let (gx, gy) = self.solution_path[i];
                let x0 = (gx as f32 * cw) as u32;
                let y0 = (gy as f32 * ch) as u32;
                let x1 = ((gx + 1) as f32 * cw).min(w as f32) as u32;
                let y1 = ((gy + 1) as f32 * ch).min(h as f32) as u32;

                for y in y0..y1 {
                    for x in x0..x1 {
                        out.put_pixel(x, y, *self.original.get_pixel(x, y));
                    }
                }
            }
        }
    }
}

// ============================================================================
// 19. NONOGRAM LOGIC FILL
// ============================================================================

pub struct NonogramFillPuzzle {
    original: RgbaImage,
    grid_size: usize,
    solve_order: Vec<(usize, usize)>,
}

impl NonogramFillPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let grid_size = (complexity as usize * 3 + 12).clamp(12, 35);
        let mut solve_order = Vec::with_capacity(grid_size * grid_size);

        for r in 0..grid_size {
            for c in 0..grid_size {
                solve_order.push((c, r));
            }
        }
        solve_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            grid_size,
            solve_order,
        }
    }
}

impl PuzzleSimulation for NonogramFillPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::NonogramFill
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let placed = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Nonogram Fill: {} / {} cells resolved", placed, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let n = self.grid_size;
        let cw = w as f32 / n as f32;
        let ch = h as f32 / n as f32;

        let total = self.total_steps();
        let target_step = ((progress * total as f32).floor() as usize).min(total);

        // Blueprint grid background
        for y in 0..h {
            for x in 0..w {
                out.put_pixel(x, y, Rgba([20, 26, 38, 255]));
            }
        }

        // Draw grid lines
        for i in 0..=n {
            let x = (i as f32 * cw) as i32;
            let col = if i % 5 == 0 { Rgba([70, 90, 130, 255]) } else { Rgba([35, 45, 65, 255]) };
            drawing::draw_line(out, x, 0, x, h as i32 - 1, col, 1);

            let y = (i as f32 * ch) as i32;
            drawing::draw_line(out, 0, y, w as i32 - 1, y, col, 1);
        }

        // Fill revealed cells
        for i in 0..target_step {
            let (c, r) = self.solve_order[i];
            let x0 = (c as f32 * cw) as u32;
            let y0 = (r as f32 * ch) as u32;
            let x1 = ((c + 1) as f32 * cw).min(w as f32) as u32;
            let y1 = ((r + 1) as f32 * ch).min(h as f32) as u32;

            for y in y0..y1 {
                for x in x0..x1 {
                    out.put_pixel(x, y, *self.original.get_pixel(x, y));
                }
            }
        }
    }
}

// ============================================================================
// 20. DELAUNAY TRIANGULATION
// ============================================================================

pub struct DelaunayMeshPuzzle {
    original: RgbaImage,
    nodes: Vec<(f32, f32)>,
    triangles: Vec<[usize; 3]>,
    solve_order: Vec<usize>,
}

impl DelaunayMeshPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let num_nodes = (complexity as usize * 12 + 40).clamp(50, 160);
        let w = original.width() as f32;
        let h = original.height() as f32;

        let mut nodes = Vec::with_capacity(num_nodes + 4);
        // Corners
        nodes.push((0.0, 0.0));
        nodes.push((w, 0.0));
        nodes.push((w, h));
        nodes.push((0.0, h));

        for _ in 0..num_nodes {
            nodes.push((rng.gen_range(5.0..w - 5.0), rng.gen_range(5.0..h - 5.0)));
        }

        // Simple triangulation via radial sweep
        let cx = w * 0.5;
        let cy = h * 0.5;
        let mut inner_indices: Vec<usize> = (4..nodes.len()).collect();
        inner_indices.sort_by(|&a, &b| {
            let (ax, ay) = nodes[a];
            let (bx, by) = nodes[b];
            (ay - cy).atan2(ax - cx).partial_cmp(&(by - cy).atan2(bx - cx)).unwrap()
        });

        let mut triangles = Vec::new();
        // Fan from center or neighbors
        for i in 0..inner_indices.len() {
            let next = (i + 1) % inner_indices.len();
            triangles.push([0, inner_indices[i], inner_indices[next]]);
            triangles.push([2, inner_indices[i], inner_indices[next]]);
        }

        let total = triangles.len();
        let mut solve_order: Vec<usize> = (0..total).collect();
        solve_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            nodes,
            triangles,
            solve_order,
        }
    }
}

impl PuzzleSimulation for DelaunayMeshPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::DelaunayMesh
    }

    fn total_steps(&self) -> usize {
        self.solve_order.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let placed = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Delaunay Mesh: {} / {} triangles filled", placed, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let total = self.total_steps();
        let target_step = ((progress * total as f32).floor() as usize).min(total);

        // Dark blueprint background
        for px in out.pixels_mut() {
            *px = Rgba([16, 20, 30, 255]);
        }

        // Draw wireframe for upcoming triangles
        for i in target_step..total {
            let [i0, i1, i2] = self.triangles[self.solve_order[i]];
            let (p0x, p0y) = self.nodes[i0];
            let (p1x, p1y) = self.nodes[i1];
            let (p2x, p2y) = self.nodes[i2];

            let col = Rgba([45, 60, 90, 255]);
            drawing::draw_line(out, p0x as i32, p0y as i32, p1x as i32, p1y as i32, col, 1);
            drawing::draw_line(out, p1x as i32, p1y as i32, p2x as i32, p2y as i32, col, 1);
            drawing::draw_line(out, p2x as i32, p2y as i32, p0x as i32, p0y as i32, col, 1);
        }

        // Fill solved triangles
        for i in 0..target_step {
            let [i0, i1, i2] = self.triangles[self.solve_order[i]];
            let (p0x, p0y) = self.nodes[i0];
            let (p1x, p1y) = self.nodes[i1];
            let (p2x, p2y) = self.nodes[i2];

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
                        out.put_pixel(x as u32, y as u32, *self.original.get_pixel(x as u32, y as u32));
                    }
                }
            }
        }
    }
}

// ============================================================================
// 21. SCRATCH-OFF FOIL BRUSH
// ============================================================================

pub struct ScratchRevealPuzzle {
    original: RgbaImage,
    brush_path: Vec<(f32, f32)>,
    brush_radius: f32,
}

impl ScratchRevealPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, _seed: u64) -> Self {
        let w = original.width() as f32;
        let h = original.height() as f32;

        let num_sweeps = (complexity * 2 + 5).clamp(7, 25);
        let brush_radius = (h / num_sweeps as f32) * 1.35;

        let mut brush_path = Vec::new();
        let step_y = h / num_sweeps as f32;

        for s in 0..num_sweeps {
            let y = (s as f32 + 0.5) * step_y;
            let num_pts = 40;
            for p in 0..=num_pts {
                let frac = p as f32 / num_pts as f32;
                let x = if s % 2 == 0 { frac * w } else { (1.0 - frac) * w };
                // Add smooth sine wobble
                let wobble = (frac * std::f32::consts::PI * 8.0).sin() * 8.0;
                brush_path.push((x, y + wobble));
            }
        }

        Self {
            original: original.clone(),
            brush_path,
            brush_radius,
        }
    }
}

impl PuzzleSimulation for ScratchRevealPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ScratchReveal
    }

    fn total_steps(&self) -> usize {
        self.brush_path.len()
    }

    fn status_label(&self, progress: f32) -> String {
        format!("Scratch Foil: {:.0}% revealed", progress * 100.0)
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let total = self.total_steps();
        let target_step = ((progress * total as f32).floor() as usize).min(total);

        // Silver/frosted metallic foil
        for y in 0..h {
            for x in 0..w {
                let grain = (((x * 19 + y * 41) % 17) as u8) * 2;
                let base = 180 + grain;
                out.put_pixel(x, y, Rgba([base, base + 4, base + 8, 255]));
            }
        }

        // Stamp scratched brush circles
        let r_sq = self.brush_radius * self.brush_radius;
        for i in 0..target_step {
            let (bx, by) = self.brush_path[i];
            let min_x = (bx - self.brush_radius).floor().max(0.0) as u32;
            let max_x = (bx + self.brush_radius).ceil().min(w as f32 - 1.0) as u32;
            let min_y = (by - self.brush_radius).floor().max(0.0) as u32;
            let max_y = (by + self.brush_radius).ceil().min(h as f32 - 1.0) as u32;

            for py in min_y..=max_y {
                let dy = py as f32 - by;
                for px in min_x..=max_x {
                    let dx = px as f32 - bx;
                    if dx * dx + dy * dy <= r_sq {
                        out.put_pixel(px, py, *self.original.get_pixel(px, py));
                    }
                }
            }
        }

        // Draw glowing brush coin/head at tip
        if target_step > 0 && target_step < total {
            let (cx, cy) = self.brush_path[target_step];
            let cr = 10.0;
            for oy in -10..=10 {
                let py = cy as i32 + oy;
                if py < 0 || py >= h as i32 { continue; }
                for ox in -10..=10 {
                    let px = cx as i32 + ox;
                    if px < 0 || px >= w as i32 { continue; }
                    if (ox * ox + oy * oy) as f32 <= cr * cr {
                        out.put_pixel(px as u32, py as u32, Rgba([255, 235, 120, 255]));
                    }
                }
            }
        }
    }
}

// ============================================================================
// 22. COLOR-BY-NUMBER FLOOD FILL
// ============================================================================

pub struct ColorByNumberPuzzle {
    original: RgbaImage,
    cluster_map: Vec<u8>, // palette index per pixel
    palette: Vec<Rgba<u8>>,
    fill_order: Vec<usize>,
}

impl ColorByNumberPuzzle {
    pub fn new(original: &RgbaImage, complexity: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let num_colors = (complexity as usize * 3 + 8).clamp(8, 28);
        let w = original.width();
        let h = original.height();
        let total_pixels = (w * h) as usize;

        // Sample initial palette seeds
        let mut palette = Vec::with_capacity(num_colors);
        for _ in 0..num_colors {
            let rx = rng.gen_range(0..w);
            let ry = rng.gen_range(0..h);
            palette.push(*original.get_pixel(rx, ry));
        }

        // Assign each pixel to nearest palette color
        let mut cluster_map = vec![0u8; total_pixels];
        for y in 0..h {
            for x in 0..w {
                let p = original.get_pixel(x, y);
                let mut best_dist = i32::MAX;
                let mut best_c = 0;

                for (idx, pal) in palette.iter().enumerate() {
                    let dr = p[0] as i32 - pal[0] as i32;
                    let dg = p[1] as i32 - pal[1] as i32;
                    let db = p[2] as i32 - pal[2] as i32;
                    let dist = dr * dr + dg * dg + db * db;
                    if dist < best_dist {
                        best_dist = dist;
                        best_c = idx;
                    }
                }
                cluster_map[(y * w + x) as usize] = best_c as u8;
            }
        }

        let mut fill_order: Vec<usize> = (0..num_colors).collect();
        fill_order.shuffle(&mut rng);

        Self {
            original: original.clone(),
            cluster_map,
            palette,
            fill_order,
        }
    }
}

impl PuzzleSimulation for ColorByNumberPuzzle {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::ColorByNumber
    }

    fn total_steps(&self) -> usize {
        self.palette.len()
    }

    fn status_label(&self, progress: f32) -> String {
        let filled = ((progress * self.total_steps() as f32).round() as usize).min(self.total_steps());
        format!("Color-by-Number: {} / {} palette zones painted", filled, self.total_steps())
    }

    fn render_frame(&self, progress: f32, out: &mut RgbaImage) {
        if progress >= 0.999 {
            *out = self.original.clone();
            return;
        }

        let w = self.original.width();
        let h = self.original.height();
        let total = self.total_steps();
        let target_step = ((progress * total as f32).floor() as usize).min(total);

        // Precompute which palette colors are unlocked
        let mut unlocked = vec![false; self.palette.len()];
        for i in 0..target_step {
            unlocked[self.fill_order[i]] = true;
        }

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let pal_id = self.cluster_map[idx] as usize;

                if unlocked[pal_id] {
                    // Painted color
                    out.put_pixel(x, y, self.palette[pal_id]);
                } else {
                    // Unpainted canvas with subtle boundary sketch
                    let edge = (x > 0 && self.cluster_map[idx - 1] as usize != pal_id)
                        || (y > 0 && self.cluster_map[idx - w as usize] as usize != pal_id);

                    if edge {
                        out.put_pixel(x, y, Rgba([140, 140, 150, 255]));
                    } else {
                        out.put_pixel(x, y, Rgba([245, 245, 240, 255]));
                    }
                }
            }
        }
    }
}
