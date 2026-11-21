use image::RgbaImage;

#[path = "../src/puzzle/mod.rs"]
mod puzzle;

use puzzle::drawing::SampleImageType;
use puzzle::{EasingCurve, PuzzleCategory, PuzzleKind, TimingConfig};

#[test]
fn test_all_22_puzzles_initialize_and_render() {
    let img = SampleImageType::VibrantMandala.generate(128, 128);
    let mut frame_buf = RgbaImage::new(128, 128);

    assert_eq!(PuzzleKind::ALL.len(), 22);

    for kind in PuzzleKind::ALL {
        let sim = kind.create_simulation(&img, 5, 42);
        assert!(sim.total_steps() > 0, "Puzzle {:?} total_steps <= 0", kind);

        // Check start frame
        sim.render_frame(0.0, &mut frame_buf);
        let label_0 = sim.status_label(0.0);
        assert!(!label_0.is_empty());

        // Check mid frame
        sim.render_frame(0.5, &mut frame_buf);

        // Check solved frame
        sim.render_frame(1.0, &mut frame_buf);
        let label_1 = sim.status_label(1.0);
        assert!(!label_1.is_empty());
    }
}

#[test]
fn test_timing_duration_controls_solve_rate() {
    let steps = 100;

    // Fast video: 2 seconds total, 0.5s hold => 1.5s active solve => 100 / 1.5 = 66.67 moves/sec
    let fast_timing = TimingConfig {
        total_duration_sec: 2.0,
        hold_duration_sec: 0.5,
        fps: 30,
        easing: EasingCurve::Linear,
        complexity: 5,
        seed: 42,
    };
    let fast_rate = fast_timing.solve_rate_steps_per_sec(steps);
    assert!((fast_rate - 66.666).abs() < 0.1);

    // Slow video: 10 seconds total, 1.0s hold => 9.0s active solve => 100 / 9.0 = 11.11 moves/sec
    let slow_timing = TimingConfig {
        total_duration_sec: 10.0,
        hold_duration_sec: 1.0,
        fps: 30,
        easing: EasingCurve::Linear,
        complexity: 5,
        seed: 42,
    };
    let slow_rate = slow_timing.solve_rate_steps_per_sec(steps);
    assert!((slow_rate - 11.111).abs() < 0.1);

    // Solve rate must be strictly higher for shorter video
    assert!(fast_rate > slow_rate);
}

#[test]
fn test_easing_curves_bounds() {
    for curve in EasingCurve::all() {
        assert_eq!(curve.apply(0.0), 0.0);
        assert_eq!(curve.apply(1.0), 1.0);

        let mid = curve.apply(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }
}

#[test]
fn test_all_sample_images_generate() {
    for sample in SampleImageType::all() {
        let img = sample.generate(64, 64);
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
    }
}

#[test]
fn test_categories_coverage() {
    let mut spatial_count = 0;
    let mut sort_count = 0;
    let mut transform_count = 0;
    let mut path_count = 0;

    for kind in PuzzleKind::ALL {
        match kind.category() {
            PuzzleCategory::SpatialAndTile => spatial_count += 1,
            PuzzleCategory::SortingAndAlgorithmic => sort_count += 1,
            PuzzleCategory::TransformAndSignal => transform_count += 1,
            PuzzleCategory::PathAndLogic => path_count += 1,
        }
    }

    assert_eq!(spatial_count, 8);
    assert_eq!(sort_count, 4);
    assert_eq!(transform_count, 5);
    assert_eq!(path_count, 5);
    assert_eq!(spatial_count + sort_count + transform_count + path_count, 22);
}
