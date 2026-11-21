pub mod drawing;
pub mod path_logic;
pub mod sorting;
pub mod spatial;
pub mod transform;

use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuzzleCategory {
    SpatialAndTile,
    SortingAndAlgorithmic,
    TransformAndSignal,
    PathAndLogic,
}

impl PuzzleCategory {
    pub fn all() -> &'static [PuzzleCategory] {
        &[
            PuzzleCategory::SpatialAndTile,
            PuzzleCategory::SortingAndAlgorithmic,
            PuzzleCategory::TransformAndSignal,
            PuzzleCategory::PathAndLogic,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            PuzzleCategory::SpatialAndTile => "Spatial & Tile",
            PuzzleCategory::SortingAndAlgorithmic => "Sorting & Algorithmic",
            PuzzleCategory::TransformAndSignal => "Transform & Signal",
            PuzzleCategory::PathAndLogic => "Path & Logic Reveal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuzzleKind {
    // 1. Spatial & Tile (8)
    JigsawClassic,
    SlidingGrid,
    TileSwap,
    RotatingTiles,
    RotatingRings,
    ToroidalShift,
    HexHoneycomb,
    TriangleMesh,

    // 2. Sorting & Algorithmic (4)
    StripSort,
    HilbertCurveSort,
    VoronoiShards,
    QuadtreeUnfold,

    // 3. Transform & Signal (5)
    SwirlVortex,
    WavePhaseAlign,
    KaleidoscopeUnfold,
    ChromaticGlitch,
    MosaicCoalesce,

    // 4. Path & Logic Reveal (5)
    MazeSolver,
    NonogramFill,
    DelaunayMesh,
    ScratchReveal,
    ColorByNumber,
}

impl PuzzleKind {
    pub const ALL: [PuzzleKind; 22] = [
        // Spatial (8)
        PuzzleKind::JigsawClassic,
        PuzzleKind::SlidingGrid,
        PuzzleKind::TileSwap,
        PuzzleKind::RotatingTiles,
        PuzzleKind::RotatingRings,
        PuzzleKind::ToroidalShift,
        PuzzleKind::HexHoneycomb,
        PuzzleKind::TriangleMesh,
        // Sorting (4)
        PuzzleKind::StripSort,
        PuzzleKind::HilbertCurveSort,
        PuzzleKind::VoronoiShards,
        PuzzleKind::QuadtreeUnfold,
        // Transform (5)
        PuzzleKind::SwirlVortex,
        PuzzleKind::WavePhaseAlign,
        PuzzleKind::KaleidoscopeUnfold,
        PuzzleKind::ChromaticGlitch,
        PuzzleKind::MosaicCoalesce,
        // Path & Logic (5)
        PuzzleKind::MazeSolver,
        PuzzleKind::NonogramFill,
        PuzzleKind::DelaunayMesh,
        PuzzleKind::ScratchReveal,
        PuzzleKind::ColorByNumber,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PuzzleKind::JigsawClassic => "1. Classic Jigsaw",
            PuzzleKind::SlidingGrid => "2. Sliding 15-Puzzle",
            PuzzleKind::TileSwap => "3. Tile Permutation Swap",
            PuzzleKind::RotatingTiles => "4. Rotating Tile Grid",
            PuzzleKind::RotatingRings => "5. Concentric Rotating Rings",
            PuzzleKind::ToroidalShift => "6. Toroidal Rubik Shift",
            PuzzleKind::HexHoneycomb => "7. Hexagonal Honeycomb",
            PuzzleKind::TriangleMesh => "8. Triangular Mesh",

            PuzzleKind::StripSort => "9. Quicksort Image Strips",
            PuzzleKind::HilbertCurveSort => "10. Hilbert Curve Unfold",
            PuzzleKind::VoronoiShards => "11. Voronoi Shattered Glass",
            PuzzleKind::QuadtreeUnfold => "12. Quadtree Recursive Split",

            PuzzleKind::SwirlVortex => "13. Swirl Vortex Unwind",
            PuzzleKind::WavePhaseAlign => "14. Sine Wave Phase Alignment",
            PuzzleKind::KaleidoscopeUnfold => "15. Kaleidoscope Mirror Unfold",
            PuzzleKind::ChromaticGlitch => "16. Chromatic & Scanline Glitch",
            PuzzleKind::MosaicCoalesce => "17. Mosaic Block Coalesce",

            PuzzleKind::MazeSolver => "18. Maze Spanning-Tree Reveal",
            PuzzleKind::NonogramFill => "19. Nonogram Logic Fill",
            PuzzleKind::DelaunayMesh => "20. Delaunay Triangulation",
            PuzzleKind::ScratchReveal => "21. Scratch-Off Foil Brush",
            PuzzleKind::ColorByNumber => "22. Color-by-Number Flood Fill",
        }
    }

    pub fn category(&self) -> PuzzleCategory {
        match self {
            PuzzleKind::JigsawClassic
            | PuzzleKind::SlidingGrid
            | PuzzleKind::TileSwap
            | PuzzleKind::RotatingTiles
            | PuzzleKind::RotatingRings
            | PuzzleKind::ToroidalShift
            | PuzzleKind::HexHoneycomb
            | PuzzleKind::TriangleMesh => PuzzleCategory::SpatialAndTile,

            PuzzleKind::StripSort
            | PuzzleKind::HilbertCurveSort
            | PuzzleKind::VoronoiShards
            | PuzzleKind::QuadtreeUnfold => PuzzleCategory::SortingAndAlgorithmic,

            PuzzleKind::SwirlVortex
            | PuzzleKind::WavePhaseAlign
            | PuzzleKind::KaleidoscopeUnfold
            | PuzzleKind::ChromaticGlitch
            | PuzzleKind::MosaicCoalesce => PuzzleCategory::TransformAndSignal,

            PuzzleKind::MazeSolver
            | PuzzleKind::NonogramFill
            | PuzzleKind::DelaunayMesh
            | PuzzleKind::ScratchReveal
            | PuzzleKind::ColorByNumber => PuzzleCategory::PathAndLogic,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PuzzleKind::JigsawClassic => "Classic jigsaw pieces with interlocking tabs flying into place.",
            PuzzleKind::SlidingGrid => "N x N sliding tiles navigating to their target positions via an empty slot.",
            PuzzleKind::TileSwap => "Scrambled grid cells swapping along smooth arcs toward their true positions.",
            PuzzleKind::RotatingTiles => "Grid cells rotated 90/180/270 degrees spinning into upright orientation.",
            PuzzleKind::RotatingRings => "Concentric radial rings rotating at distinct speeds into alignment.",
            PuzzleKind::ToroidalShift => "Row and column cyclic shifts wrapping smoothly around edges.",
            PuzzleKind::HexHoneycomb => "Hexagonal honeycomb tiles flying and locking into hexagonal geometry.",
            PuzzleKind::TriangleMesh => "Triangular mesh fragments rotating and fitting together.",
            PuzzleKind::StripSort => "Vertical/horizontal strips sorting into position via algorithmic swaps.",
            PuzzleKind::HilbertCurveSort => "Blocks ordered along a 1D space-filling Hilbert curve reordering into 2D.",
            PuzzleKind::VoronoiShards => "Organic shattered glass shards converging from an explosive scatter.",
            PuzzleKind::QuadtreeUnfold => "Hierarchical quadtree subdividing high-contrast blocks recursively.",
            PuzzleKind::SwirlVortex => "Radial swirling whirlpool unwinding non-linear angular distortion.",
            PuzzleKind::WavePhaseAlign => "Harmonic multi-frequency sinusoidal displacement waves locking in phase.",
            PuzzleKind::KaleidoscopeUnfold => "Radial kaleidoscopic reflection unfolding back to the original image.",
            PuzzleKind::ChromaticGlitch => "Separated RGB channels and jittered scanlines realigning into crisp focus.",
            PuzzleKind::MosaicCoalesce => "Coarse pixelated mosaic blocks smoothly refining to full resolution.",
            PuzzleKind::MazeSolver => "A labyrinth drawn over the image solved by an illuminating BFS path.",
            PuzzleKind::NonogramFill => "Rows and columns deduced and filled in by logical constraint solving.",
            PuzzleKind::DelaunayMesh => "Geometric wireframe mesh triangulating and shading into full artwork.",
            PuzzleKind::ScratchReveal => "Metallic frosted coating scratched away in a fluid continuous path.",
            PuzzleKind::ColorByNumber => "Quantized color regions numbered and filled sequentially.",
        }
    }

    pub fn create_simulation(
        &self,
        image: &RgbaImage,
        complexity: u32,
        seed: u64,
    ) -> Box<dyn PuzzleSimulation> {
        let c = complexity.clamp(1, 20);
        match self {
            // Spatial
            PuzzleKind::JigsawClassic => Box::new(spatial::JigsawPuzzle::new(image, c, seed)),
            PuzzleKind::SlidingGrid => Box::new(spatial::SlidingGridPuzzle::new(image, c, seed)),
            PuzzleKind::TileSwap => Box::new(spatial::TileSwapPuzzle::new(image, c, seed)),
            PuzzleKind::RotatingTiles => Box::new(spatial::RotatingTilesPuzzle::new(image, c, seed)),
            PuzzleKind::RotatingRings => Box::new(spatial::RotatingRingsPuzzle::new(image, c, seed)),
            PuzzleKind::ToroidalShift => Box::new(spatial::ToroidalShiftPuzzle::new(image, c, seed)),
            PuzzleKind::HexHoneycomb => Box::new(spatial::HexHoneycombPuzzle::new(image, c, seed)),
            PuzzleKind::TriangleMesh => Box::new(spatial::TriangleMeshPuzzle::new(image, c, seed)),

            // Sorting
            PuzzleKind::StripSort => Box::new(sorting::StripSortPuzzle::new(image, c, seed)),
            PuzzleKind::HilbertCurveSort => Box::new(sorting::HilbertCurvePuzzle::new(image, c, seed)),
            PuzzleKind::VoronoiShards => Box::new(sorting::VoronoiShardsPuzzle::new(image, c, seed)),
            PuzzleKind::QuadtreeUnfold => Box::new(sorting::QuadtreeUnfoldPuzzle::new(image, c, seed)),

            // Transform
            PuzzleKind::SwirlVortex => Box::new(transform::SwirlVortexPuzzle::new(image, c, seed)),
            PuzzleKind::WavePhaseAlign => Box::new(transform::WavePhaseAlignPuzzle::new(image, c, seed)),
            PuzzleKind::KaleidoscopeUnfold => Box::new(transform::KaleidoscopePuzzle::new(image, c, seed)),
            PuzzleKind::ChromaticGlitch => Box::new(transform::ChromaticGlitchPuzzle::new(image, c, seed)),
            PuzzleKind::MosaicCoalesce => Box::new(transform::MosaicCoalescePuzzle::new(image, c, seed)),

            // Path & Logic
            PuzzleKind::MazeSolver => Box::new(path_logic::MazeSolverPuzzle::new(image, c, seed)),
            PuzzleKind::NonogramFill => Box::new(path_logic::NonogramFillPuzzle::new(image, c, seed)),
            PuzzleKind::DelaunayMesh => Box::new(path_logic::DelaunayMeshPuzzle::new(image, c, seed)),
            PuzzleKind::ScratchReveal => Box::new(path_logic::ScratchRevealPuzzle::new(image, c, seed)),
            PuzzleKind::ColorByNumber => Box::new(path_logic::ColorByNumberPuzzle::new(image, c, seed)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackPhase {
    Solve(f32),     // progress 0.0 -> 1.0 (Phase 1)
    Hold,           // crisp solved image (Phase 2)
    Rollback(f32),  // rollback progress 0.0 -> 1.0 (Phase 3)
}

pub trait PuzzleSimulation: Send + Sync {
    #[allow(dead_code)]
    fn kind(&self) -> PuzzleKind;
    fn total_steps(&self) -> usize;
    fn status_label(&self, progress: f32) -> String;
    fn render_frame(&self, progress: f32, out: &mut RgbaImage);
    fn render_rollback_frame(&self, progress: f32, out: &mut RgbaImage);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingCurve {
    Linear,
    EaseInOut,
    Accelerate,
    Decelerate,
}

impl EasingCurve {
    pub fn all() -> &'static [EasingCurve] {
        &[
            EasingCurve::Linear,
            EasingCurve::EaseInOut,
            EasingCurve::Accelerate,
            EasingCurve::Decelerate,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EasingCurve::Linear => "Linear",
            EasingCurve::EaseInOut => "Smooth (Ease-In-Out)",
            EasingCurve::Accelerate => "Accelerating",
            EasingCurve::Decelerate => "Decelerating",
        }
    }

    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingCurve::Linear => t,
            EasingCurve::EaseInOut => {
                // Smoothstep / cubic Hermite
                t * t * (3.0 - 2.0 * t)
            }
            EasingCurve::Accelerate => t * t,
            EasingCurve::Decelerate => 1.0 - (1.0 - t) * (1.0 - t),
        }
    }
}

pub fn format_duration_mmss(seconds: f32) -> String {
    let total_secs = seconds.max(0.0);
    let mins = (total_secs / 60.0).floor() as u32;
    let secs = (total_secs % 60.0).floor() as u32;
    format!("{:02}:{:02}", mins, secs)
}

pub fn format_duration_mmss_sub(seconds: f32) -> String {
    let total_secs = seconds.max(0.0);
    let mins = (total_secs / 60.0).floor() as u32;
    let secs = total_secs % 60.0;
    format!("{:02}:{:04.1}", mins, secs)
}

pub fn parse_duration_string(s: &str) -> Option<f32> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    // Handle "5:57" or "05:57.5"
    if let Some((min_str, sec_str)) = s.split_once(':') {
        let mins: f32 = min_str.trim().parse().ok()?;
        let secs: f32 = sec_str.trim().parse().ok()?;
        if mins >= 0.0 && secs >= 0.0 {
            return Some((mins * 60.0 + secs).clamp(0.5, 420.0));
        }
    }

    // Handle "5m 57s" or "5m57s" or "5m" or "57s"
    if s.contains('m') || s.contains('s') {
        let mut total = 0.0f32;
        let mut found_any = false;

        if let Some(m_pos) = s.find('m') {
            let m_val: f32 = s[..m_pos].trim().parse().ok()?;
            total += m_val * 60.0;
            found_any = true;

            let remainder = s[(m_pos + 1)..].trim();
            if !remainder.is_empty() {
                let sec_part = remainder.trim_end_matches('s').trim();
                if !sec_part.is_empty() {
                    let s_val: f32 = sec_part.parse().ok()?;
                    total += s_val;
                }
            }
        } else if s.ends_with('s') {
            let s_val: f32 = s.trim_end_matches('s').trim().parse().ok()?;
            total += s_val;
            found_any = true;
        }

        if found_any {
            return Some(total.clamp(0.5, 420.0));
        }
    }

    // Handle raw seconds e.g. "357" or "357.5"
    s.parse::<f32>().ok().map(|v| v.clamp(0.5, 420.0))
}

#[derive(Debug, Clone)]
pub struct TimingConfig {
    pub total_duration_sec: f32,
    pub hold_duration_sec: f32,
    pub fps: u32,
    pub easing: EasingCurve,
    pub complexity: u32,
    pub auto_scale_steps: bool,
    pub rollback: bool,
    pub seed: u64,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            total_duration_sec: 5.0,
            hold_duration_sec: 1.0,
            fps: 60,
            easing: EasingCurve::EaseInOut,
            complexity: 5,
            auto_scale_steps: false,
            rollback: false,
            seed: 42,
        }
    }
}

impl TimingConfig {
    pub fn effective_complexity(&self) -> u32 {
        if self.auto_scale_steps && self.total_duration_sec > 30.0 {
            let scale_bonus = ((self.total_duration_sec - 30.0) / 39.0).round() as u32; // up to +10 for 420s
            (self.complexity + scale_bonus).clamp(1, 20)
        } else {
            self.complexity.clamp(1, 20)
        }
    }

    pub fn solve_duration_sec(&self) -> f32 {
        let max_hold = (self.total_duration_sec * 0.33).max(0.0);
        let hold = self.hold_duration_sec.min(max_hold).max(0.0);
        let active = (self.total_duration_sec - hold).max(0.1);
        if self.rollback {
            active * 0.5
        } else {
            active
        }
    }

    pub fn rollback_duration_sec(&self) -> f32 {
        if self.rollback {
            self.solve_duration_sec()
        } else {
            0.0
        }
    }

    pub fn effective_hold_duration_sec(&self) -> f32 {
        let max_hold = (self.total_duration_sec * 0.33).max(0.0);
        self.hold_duration_sec.min(max_hold).max(0.0)
    }

    pub fn total_frames(&self) -> u32 {
        (self.total_duration_sec * self.fps as f32).round().max(1.0) as u32
    }

    pub fn playback_state_at_time(&self, time_sec: f32) -> PlaybackPhase {
        let d_solve = self.solve_duration_sec();
        let d_hold = self.effective_hold_duration_sec();

        if !self.rollback {
            let linear_u = (time_sec / d_solve).clamp(0.0, 1.0);
            if linear_u >= 1.0 {
                PlaybackPhase::Hold
            } else {
                PlaybackPhase::Solve(self.easing.apply(linear_u))
            }
        } else {
            if time_sec < d_solve {
                let linear_u = (time_sec / d_solve).clamp(0.0, 1.0);
                PlaybackPhase::Solve(self.easing.apply(linear_u))
            } else if time_sec < d_solve + d_hold {
                PlaybackPhase::Hold
            } else {
                let d_rb = self.rollback_duration_sec();
                let rb_t = time_sec - (d_solve + d_hold);
                let linear_u = (rb_t / d_rb).clamp(0.0, 1.0);
                PlaybackPhase::Rollback(self.easing.apply(linear_u))
            }
        }
    }

    pub fn playback_state_at_frame(&self, frame_index: u32) -> PlaybackPhase {
        let time_sec = frame_index as f32 / self.fps as f32;
        self.playback_state_at_time(time_sec)
    }

    #[allow(dead_code)]
    pub fn progress_at_time(&self, time_sec: f32) -> f32 {
        match self.playback_state_at_time(time_sec) {
            PlaybackPhase::Solve(p) => p,
            PlaybackPhase::Hold => 1.0,
            PlaybackPhase::Rollback(p) => 1.0 - p,
        }
    }

    #[allow(dead_code)]
    pub fn progress_at_frame(&self, frame_index: u32) -> f32 {
        let time_sec = frame_index as f32 / self.fps as f32;
        self.progress_at_time(time_sec)
    }

    pub fn solve_rate_steps_per_sec(&self, total_steps: usize) -> f32 {
        let solve_sec = self.solve_duration_sec();
        total_steps as f32 / solve_sec
    }
}
