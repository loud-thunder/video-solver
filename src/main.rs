mod app;
mod puzzle;
mod video;

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use app::VideoSolverApp;
use eframe::egui;
use puzzle::drawing::SampleImageType;
use puzzle::{PuzzleKind, TimingConfig};
use video::{export_video, ExportJobConfig, ExportProgress, VideoFormat};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // CLI Verification Mode
    if args.iter().any(|a| a == "--verify-export") {
        println!("==> Running Automated Video Solver Verification Test...");
        run_automated_verification()?;
        println!("==> All verification tests passed successfully!");
        return Ok(());
    }

    // CLI Headless Export Mode
    if args.iter().any(|a| a == "--cli") {
        return run_cli_export(&args);
    }

    // Launch Native Desktop GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 840.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Image-to-Puzzle Video Solver Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "Image-to-Puzzle Video Solver",
        options,
        Box::new(|cc| Ok(Box::new(VideoSolverApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))
}

fn run_automated_verification() -> anyhow::Result<()> {
    // 1. Test image generation
    println!("Step 1: Generating sample procedural images...");
    let sample = SampleImageType::VibrantMandala;
    let img = sample.generate(320, 320);
    assert_eq!(img.width(), 320);
    assert_eq!(img.height(), 320);
    println!("✓ Procedural image generated successfully (320x320)");

    // 2. Test all 22 puzzles
    println!("Step 2: Initializing and rendering all 22 puzzle simulations...");
    let mut frame_buf = image::RgbaImage::new(320, 320);

    for (i, kind) in PuzzleKind::ALL.iter().enumerate() {
        let sim = kind.create_simulation(&img, 5, 42);
        assert!(sim.total_steps() > 0, "Puzzle {:?} has 0 steps", kind);

        // Test scrambled state (progress = 0.0)
        sim.render_frame(0.0, &mut frame_buf);
        let _status_0 = sim.status_label(0.0);

        // Test middle state (progress = 0.5)
        sim.render_frame(0.5, &mut frame_buf);

        // Test solved state (progress = 1.0)
        sim.render_frame(1.0, &mut frame_buf);
        let status_1 = sim.status_label(1.0);

        // Test roll-back state (progress = 0.0, 0.5, 1.0)
        sim.render_rollback_frame(0.0, &mut frame_buf);
        sim.render_rollback_frame(0.5, &mut frame_buf);
        sim.render_rollback_frame(1.0, &mut frame_buf);

        println!(
            "  [{:02}/22] {:<30} | Steps: {:<4} | Status: {}",
            i + 1,
            kind.name(),
            sim.total_steps(),
            status_1
        );
    }
    println!("✓ All 22 puzzle simulations initialized and rendered successfully (including roll-back)!");

    // 3. Test video export to MP4 via ffmpeg
    let test_mp4 = PathBuf::from("target/test_output.mp4");
    if let Some(parent) = test_mp4.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let timing = TimingConfig {
        total_duration_sec: 2.0,
        hold_duration_sec: 0.5,
        fps: 30,
        easing: puzzle::EasingCurve::EaseInOut,
        complexity: 4,
        auto_scale_steps: false,
        rollback: false,
        seed: 12345,
    };

    println!(
        "Step 3: Exporting 2.0s MP4 (60 frames) via ffmpeg (Solve Rate: {:.1} moves/sec)...",
        timing.solve_rate_steps_per_sec(16)
    );

    let config = ExportJobConfig {
        output_path: test_mp4.clone(),
        format: VideoFormat::Mp4H264,
        width: 320,
        height: 320,
        timing: timing.clone(),
        puzzle_kind: PuzzleKind::JigsawClassic,
    };

    let progress = ExportProgress::new(timing.total_frames());
    export_video(&img, &config, progress.clone())?;

    assert!(test_mp4.exists(), "MP4 file was not created");
    let metadata = std::fs::metadata(&test_mp4)?;
    assert!(metadata.len() > 1000, "MP4 file is too small ({} bytes)", metadata.len());
    println!("✓ MP4 exported successfully: {} (size: {} bytes)", test_mp4.display(), metadata.len());

    // 3b. Test 3-Phase Roll-back video export
    let test_rollback_mp4 = PathBuf::from("target/test_rollback.mp4");
    let rollback_timing = TimingConfig {
        total_duration_sec: 2.0,
        hold_duration_sec: 0.2,
        fps: 30,
        easing: puzzle::EasingCurve::EaseInOut,
        complexity: 4,
        auto_scale_steps: false,
        rollback: true,
        seed: 12345,
    };

    println!("Step 3b: Exporting 2.0s 3-Phase Roll-back MP4 via ffmpeg...");
    let rollback_config = ExportJobConfig {
        output_path: test_rollback_mp4.clone(),
        format: VideoFormat::Mp4H264,
        width: 320,
        height: 320,
        timing: rollback_timing.clone(),
        puzzle_kind: PuzzleKind::JigsawClassic,
    };
    let rollback_progress = ExportProgress::new(rollback_timing.total_frames());
    export_video(&img, &rollback_config, rollback_progress)?;
    assert!(test_rollback_mp4.exists(), "Rollback MP4 file was not created");
    println!("✓ Roll-back MP4 exported successfully: {} (size: {} bytes)", test_rollback_mp4.display(), std::fs::metadata(&test_rollback_mp4)?.len());

    // 4. Test pure-Rust GIF export
    let test_gif = PathBuf::from("target/test_output.gif");
    let gif_config = ExportJobConfig {
        output_path: test_gif.clone(),
        format: VideoFormat::Gif,
        width: 160,
        height: 160,
        timing: TimingConfig {
            total_duration_sec: 1.0,
            hold_duration_sec: 0.2,
            fps: 15,
            ..timing.clone()
        },
        puzzle_kind: PuzzleKind::RotatingTiles,
    };

    let gif_progress = ExportProgress::new(15);
    export_video(&img, &gif_config, gif_progress)?;
    assert!(test_gif.exists(), "GIF file was not created");
    println!("✓ GIF exported successfully: {} (size: {} bytes)", test_gif.display(), std::fs::metadata(&test_gif)?.len());

    Ok(())
}

fn run_cli_export(args: &[String]) -> anyhow::Result<()> {
    let mut input_path: Option<String> = None;
    let mut output_path = String::from("puzzle_output.mp4");
    let mut duration_sec = 5.0f32;
    let mut complexity = 5u32;
    let mut puzzle_index = 0usize;
    let mut rollback = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                if i < args.len() { input_path = Some(args[i].clone()); }
            }
            "--output" => {
                i += 1;
                if i < args.len() { output_path = args[i].clone(); }
            }
            "--duration" => {
                i += 1;
                if i < args.len() {
                    duration_sec = puzzle::parse_duration_string(&args[i]).unwrap_or(5.0);
                }
            }
            "--complexity" => {
                i += 1;
                if i < args.len() { complexity = args[i].parse().unwrap_or(5); }
            }
            "--rollback" => {
                rollback = true;
            }
            "--puzzle" => {
                i += 1;
                if i < args.len() {
                    let p_arg = &args[i];
                    if let Ok(idx) = p_arg.parse::<usize>() {
                        puzzle_index = idx.saturating_sub(1).min(PuzzleKind::ALL.len() - 1);
                    } else {
                        puzzle_index = PuzzleKind::ALL.iter().position(|k| {
                            k.name().to_lowercase().contains(&p_arg.to_lowercase())
                        }).unwrap_or(0);
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    let img = if let Some(path) = input_path {
        println!("Loading input image from: {}", path);
        image::open(path)?.to_rgba8()
    } else {
        println!("Using default Vibrant Mandala procedural image...");
        SampleImageType::VibrantMandala.generate(600, 600)
    };

    let kind = PuzzleKind::ALL[puzzle_index];
    let timing = TimingConfig {
        total_duration_sec: duration_sec,
        hold_duration_sec: (duration_sec * 0.2).min(2.0),
        fps: 60,
        complexity,
        rollback,
        ..Default::default()
    };

    let is_gif = output_path.ends_with(".gif");
    let format = if is_gif { VideoFormat::Gif } else { VideoFormat::Mp4H264 };

    println!("Puzzle: {}", kind.name());
    println!("Duration: {:.1}s (FPS: {})", timing.total_duration_sec, timing.fps);
    println!("Output: {}", output_path);

    let config = ExportJobConfig {
        output_path: PathBuf::from(output_path),
        format,
        width: img.width(),
        height: img.height(),
        timing,
        puzzle_kind: kind,
    };

    let progress = ExportProgress::new(config.timing.total_frames());
    let prog_clone = progress.clone();

    // Print progress in background
    let watcher = std::thread::spawn(move || {
        while !prog_clone.is_done.load(Ordering::Relaxed) && !prog_clone.is_cancelled.load(Ordering::Relaxed) {
            let cur = prog_clone.current_frame.load(Ordering::Relaxed);
            let tot = prog_clone.total_frames.load(Ordering::Relaxed);
            print!("\rRendering frame {} / {} ({:.1}%)...", cur, tot, prog_clone.fraction() * 100.0);
            let _ = std::io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    export_video(&img, &config, progress)?;
    let _ = watcher.join();
    println!("\nExport finished successfully!");
    Ok(())
}
