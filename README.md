# 🎬 Image-to-Puzzle Video Solver Studio (`video-solver`)

A native Rust desktop application and CLI tool that converts any input image into one of **22 distinct puzzles** across 4 categories and solves it over a user-specified video duration, exporting high-speed H.264 `.mp4` (via `ffmpeg`) or `.gif` animations.

---

## Features

- **Native Studio GUI (`eframe` / `egui`)**:
  - Live 60 FPS preview canvas with real-time playback scrubber (`0:00 .. 7:00`).
  - Play / Pause, restart, and variable playback speeds (`0.5x`, `1.0x`, `2.0x`).
  - Category filters and dropdown selector for all 22 puzzles.
  - Interactive sliders for Puzzle Complexity (`1..=20`) with "Auto-scale step density for long videos" toggle.
  - Random Seed generator.
  - Video Duration slider & exact text input up to **7 minutes (420.0s)** with unrounded precision (supports `M:SS`, `MM:SS`, `5:57`, `5:57.5`, raw seconds `357`), and quick presets: `[30s]`, `[1m]`, `[3m]`, `[5m]`, `[7m]`.
  - **3-Phase Seamless Roll-Back Loop Mode**:
    - Optional toggle that solves the puzzle (45% duration), holds the solved image (10% duration), and then rolls back to the initial scrambled state via a novel route (45% duration).
    - Novel roll-back routes: center-out disassembly, parabolic loft flight arcs, secondary centrifugal spins, and reverse-order un-sorting animations (not simple frame-reversed video).
    - Final frame lands exactly on the initial scrambled frame for a seamless infinite video loop.
  - Solved Hold Duration slider (`0.0s..10.0s`), FPS (`30` / `60`), and Easing Curves (`Linear`, `Ease-In-Out`, `Accelerate`, `Decelerate`).
  - Live Solve Rate readout calculating the exact rate in `moves / second` based on the specified duration and active loop mode.
  - Single-click video export for `.mp4` (H.264), animated `.gif` (for videos $\le 1 \text{ min}$), or **Batch Export All 22 Puzzles**.
  - Built-in procedural image generators (Vibrant Mandala, Cyber Grid, Sunset Waves, Bauhaus Geometric) plus native file picker dialog for custom images (`PNG`, `JPG`, `WEBP`, `BMP`).

- **CLI Support**:
  - Headless video generation without launching the GUI.
  - Automated verification suite testing all 22 puzzles and export pipelines.

---

## The 22 Puzzle Types

| # | Puzzle Name | Category | Description |
|---|---|---|---|
| 1 | **Classic Jigsaw** | Spatial & Tile | Interlocking jigsaw pieces scattered across the canvas flying and rotating into locked positions. |
| 2 | **Sliding 15-Puzzle** | Spatial & Tile | $N \times N$ grid tiles sliding through an empty slot along the exact shortest path reversal. |
| 3 | **Tile Permutation Swap** | Spatial & Tile | Scrambled image tiles arcing across the board to swap places until sorted. |
| 4 | **Rotating Tile Grid** | Spatial & Tile | Tiles oriented at $90^\circ, 180^\circ, 270^\circ$ spinning upright in place. |
| 5 | **Concentric Rotating Rings** | Spatial & Tile | Concentric radial rings counter-rotating into alignment from center outward. |
| 6 | **Toroidal Rubik Shift** | Spatial & Tile | Row and column cyclic wrap shifts shifting back into place. |
| 7 | **Hexagonal Honeycomb** | Spatial & Tile | Hexagonal tessellated tiles flying in from a dispersed ring. |
| 8 | **Triangular Mesh** | Spatial & Tile | Diagonal triangular mesh fragments rotating and locking together. |
| 9 | **Quicksort Image Strips** | Sorting & Algorithmic | Image strips sorting via algorithmic Quicksort swaps with highlighted pivots. |
| 10 | **Hilbert Curve Unfold** | Sorting & Algorithmic | Blocks ordered along a 1D space-filling Hilbert curve reordering into 2D. |
| 11 | **Voronoi Shattered Glass** | Sorting & Algorithmic | Polygonal shattered glass shards converging from an explosive scatter. |
| 12 | **Quadtree Recursive Split** | Sorting & Algorithmic | Hierarchical quadtree subdividing coarse color blocks down to pixel resolution. |
| 13 | **Swirl Vortex Unwind** | Transform & Signal | Radial swirling vortex whirlpool unwinding non-linear angular torsion. |
| 14 | **Sine Wave Phase Alignment** | Transform & Signal | Harmonic multi-frequency sinusoidal displacement waves locking in phase. |
| 15 | **Kaleidoscope Mirror Unfold** | Transform & Signal | Multi-fold radial kaleidoscope reflection unfolding to the true image. |
| 16 | **Chromatic & Scanline Glitch** | Transform & Signal | Separated RGB color planes and jittered horizontal scanlines realigning. |
| 17 | **Mosaic Block Coalesce** | Transform & Signal | Coarse pixelated mosaic blocks smoothly coalescing down to 1:1 pixels. |
| 18 | **Maze Spanning-Tree Reveal** | Path & Logic Reveal | Labyrinth drawn over the image solved by an illuminating BFS path search. |
| 19 | **Nonogram Logic Fill** | Path & Logic Reveal | Logic puzzle grid with constraint clues systematically filling cells. |
| 20 | **Delaunay Triangulation** | Path & Logic Reveal | Geometric wireframe mesh triangulating and shading into full artwork. |
| 21 | **Scratch-Off Foil Brush** | Path & Logic Reveal | Frosted metallic foil scratched away by a continuous sweeping brush. |
| 22 | **Color-by-Number Flood Fill** | Path & Logic Reveal | Quantized color palette regions numbered and painted sequentially. |

---

## Duration & Solve Rate Relationship

The specified video duration directly dictates the puzzle solving rate:

$$D_{\text{solve}} = \max\left(0.1, D_{\text{total}} - D_{\text{hold}}\right)$$
$$\text{Solve Rate} = \frac{N_{\text{steps}}}{D_{\text{solve}}} \quad (\text{moves / second})$$

- Shorter video lengths automatically increase the moves per second.
- Sub-step progress is smoothly interpolated per frame using the selected easing curve.

---

## Usage

### 1. Launching the GUI Studio
```bash
cargo run --release
```

### 2. Headless CLI Video Export
```bash
cargo run --release -- --cli --duration 7:00 --complexity 12 --puzzle "Classic Jigsaw" --output jigsaw.mp4
```
Options:
- `--input <path>`: Path to image file (optional, defaults to procedural Mandala).
- `--output <path>`: Target `.mp4` or `.gif` path (defaults to `puzzle_output.mp4`).
- `--duration <sec or M:SS>`: Total video duration up to 7 minutes (e.g. `5:57`, `420`, or `7:00`).
- `--complexity <1-20>`: Puzzle complexity scale (e.g. `12`).
- `--puzzle <name or 1-22>`: Puzzle name substring or index `1..=22`.
- `--rollback`: Enable 3-phase roll-back loop (solve -> hold -> novel roll-back route).

### 3. Automated Verification Suite
```bash
cargo run --release -- --verify-export
```
Runs unit tests, initializes and renders all 22 puzzles, exports test MP4 and GIF files, and verifies output dimensions and frame rates.
