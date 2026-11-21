#![allow(dead_code)]

use image::RgbaImage;

#[path = "../src/puzzle/mod.rs"]
mod puzzle;

use puzzle::drawing::SampleImageType;
use puzzle::{
    format_duration_mmss, format_duration_mmss_sub, parse_duration_string, EasingCurve,
    PlaybackPhase, PuzzleCategory, PuzzleKind, TimingConfig,
};

#[test]
fn test_all_22_puzzles_initialize_and_render_complexity_20() {
    let img = SampleImageType::VibrantMandala.generate(128, 128);
    let mut frame_buf = RgbaImage::new(128, 128);

    assert_eq!(PuzzleKind::ALL.len(), 22);

    for kind in PuzzleKind::ALL {
        let sim = kind.create_simulation(&img, 20, 9999);
        assert!(sim.total_steps() > 0, "Puzzle {:?} total_steps <= 0 at complexity 20", kind);

        // Check start frame
        sim.render_frame(0.0, &mut frame_buf);
        let label_0 = sim.status_label(0.0);
        assert!(!label_0.is_empty());

        // Check solved frame
        sim.render_frame(1.0, &mut frame_buf);
        let label_1 = sim.status_label(1.0);
        assert!(!label_1.is_empty());
    }
}

#[test]
fn test_duration_mmss_formatting_and_parsing() {
    assert_eq!(format_duration_mmss(0.0), "00:00");
    assert_eq!(format_duration_mmss(5.0), "00:05");
    assert_eq!(format_duration_mmss(65.0), "01:05");
    assert_eq!(format_duration_mmss(420.0), "07:00");

    assert_eq!(format_duration_mmss_sub(125.4), "02:05.4");

    assert_eq!(parse_duration_string("7:00"), Some(420.0));
    assert_eq!(parse_duration_string("06:30"), Some(390.0));
    assert_eq!(parse_duration_string("5:57"), Some(357.0));
    assert_eq!(parse_duration_string("05:57"), Some(357.0));
    assert_eq!(parse_duration_string("5:57.5"), Some(357.5));
    assert_eq!(parse_duration_string("5m 57s"), Some(357.0));
    assert_eq!(parse_duration_string("5m57s"), Some(357.0));
    assert_eq!(parse_duration_string("5m"), Some(300.0));
    assert_eq!(parse_duration_string("57s"), Some(57.0));
    assert_eq!(parse_duration_string("357"), Some(357.0));
    assert_eq!(parse_duration_string("357.25"), Some(357.25));
    assert_eq!(parse_duration_string("30.5"), Some(30.5));
}

#[test]
fn test_seven_minute_duration_rate() {
    let timing = TimingConfig {
        total_duration_sec: 420.0, // 7 minutes
        hold_duration_sec: 10.0,
        fps: 60,
        easing: EasingCurve::Linear,
        complexity: 15,
        auto_scale_steps: true,
        rollback: false,
        seed: 42,
    };

    assert_eq!(timing.solve_duration_sec(), 410.0);
    assert_eq!(timing.total_frames(), 420 * 60);

    let total_steps = 820;
    let rate = timing.solve_rate_steps_per_sec(total_steps);
    // 820 / 410 = 2.0 steps / sec
    assert!((rate - 2.0).abs() < 1e-4);
}

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
        auto_scale_steps: false,
        rollback: false,
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
        auto_scale_steps: false,
        rollback: false,
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

#[test]
fn test_rollback_all_22_puzzles_seamless_loop() {
    let img = SampleImageType::VibrantMandala.generate(96, 96);
    let mut initial_solve_frame = RgbaImage::new(96, 96);
    let mut end_rollback_frame = RgbaImage::new(96, 96);
    let mut final_solve_frame = RgbaImage::new(96, 96);
    let mut start_rollback_frame = RgbaImage::new(96, 96);
    let mut mid_rollback_frame = RgbaImage::new(96, 96);

    for kind in PuzzleKind::ALL {
        let sim = kind.create_simulation(&img, 5, 42);

        // 1. Initial scramble (Solve @ 0.0) vs Rollback end (Rollback @ 1.0)
        sim.render_frame(0.0, &mut initial_solve_frame);
        sim.render_rollback_frame(1.0, &mut end_rollback_frame);

        // Verify pixel difference is zero or negligible (seamless loop to frame 0)
        let diff_initial_end: u64 = initial_solve_frame
            .pixels()
            .zip(end_rollback_frame.pixels())
            .map(|(p1, p2)| {
                (p1[0] as i32 - p2[0] as i32).abs() as u64
                    + (p1[1] as i32 - p2[1] as i32).abs() as u64
                    + (p1[2] as i32 - p2[2] as i32).abs() as u64
                    + (p1[3] as i32 - p2[3] as i32).abs() as u64
            })
            .sum();
        let avg_diff_initial = diff_initial_end as f64 / (96 * 96 * 4) as f64;
        assert!(
            avg_diff_initial < 1.0,
            "Puzzle {:?} rollback end (1.0) does not match initial scramble (0.0): avg channel diff {}",
            kind,
            avg_diff_initial
        );

        // 2. Solved frame (Solve @ 1.0) vs Rollback start (Rollback @ 0.0)
        sim.render_frame(1.0, &mut final_solve_frame);
        sim.render_rollback_frame(0.0, &mut start_rollback_frame);

        let diff_solved_start: u64 = final_solve_frame
            .pixels()
            .zip(start_rollback_frame.pixels())
            .map(|(p1, p2)| {
                (p1[0] as i32 - p2[0] as i32).abs() as u64
                    + (p1[1] as i32 - p2[1] as i32).abs() as u64
                    + (p1[2] as i32 - p2[2] as i32).abs() as u64
                    + (p1[3] as i32 - p2[3] as i32).abs() as u64
            })
            .sum();
        let avg_diff_solved = diff_solved_start as f64 / (96 * 96 * 4) as f64;
        assert!(
            avg_diff_solved < 1.0,
            "Puzzle {:?} rollback start (0.0) does not match solved frame (1.0): avg channel diff {}",
            kind,
            avg_diff_solved
        );

        // 3. Rollback mid progress renders cleanly without panic
        sim.render_rollback_frame(0.5, &mut mid_rollback_frame);
    }
}

#[test]
fn test_3_phase_playback_timing() {
    let timing = TimingConfig {
        total_duration_sec: 10.0,
        hold_duration_sec: 1.0,
        fps: 10, // 100 total frames
        easing: EasingCurve::Linear,
        complexity: 5,
        auto_scale_steps: false,
        rollback: true,
        seed: 42,
    };

    assert_eq!(timing.total_frames(), 100);

    // Frame 0 to 44 should be Solve
    match timing.playback_state_at_frame(0) {
        PlaybackPhase::Solve(p) => assert_eq!(p, 0.0),
        _ => panic!("Expected Solve at frame 0"),
    }
    match timing.playback_state_at_frame(22) {
        PlaybackPhase::Solve(p) => assert!((p - 0.4888).abs() < 0.02),
        _ => panic!("Expected Solve at frame 22"),
    }

    // Frame 45 to 54 should be Hold
    match timing.playback_state_at_frame(45) {
        PlaybackPhase::Hold => {}
        _ => panic!("Expected Hold at frame 45"),
    }
    match timing.playback_state_at_frame(54) {
        PlaybackPhase::Hold => {}
        _ => panic!("Expected Hold at frame 54"),
    }

    // Frame 55 to 99 should be Rollback
    match timing.playback_state_at_frame(55) {
        PlaybackPhase::Rollback(p) => assert_eq!(p, 0.0),
        _ => panic!("Expected Rollback at frame 55"),
    }
    match timing.playback_state_at_frame(99) {
        PlaybackPhase::Rollback(p) => assert!((p - 0.9777).abs() < 0.02),
        _ => panic!("Expected Rollback at frame 99"),
    }
    match timing.playback_state_at_time(10.0) {
        PlaybackPhase::Rollback(p) => assert_eq!(p, 1.0),
        _ => panic!("Expected Rollback at time 10.0"),
    }
}
