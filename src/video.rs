use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use image::RgbaImage;

use crate::puzzle::{PuzzleKind, PuzzleSimulation, TimingConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4H264,
    Gif,
}

#[derive(Debug, Clone)]
pub struct ExportJobConfig {
    pub output_path: PathBuf,
    pub format: VideoFormat,
    pub width: u32,
    pub height: u32,
    pub timing: TimingConfig,
    pub puzzle_kind: PuzzleKind,
}

#[derive(Clone)]
pub struct ExportProgress {
    pub current_frame: Arc<AtomicU32>,
    pub total_frames: Arc<AtomicU32>,
    pub is_cancelled: Arc<AtomicBool>,
    pub is_done: Arc<AtomicBool>,
}

impl ExportProgress {
    pub fn new(total_frames: u32) -> Self {
        Self {
            current_frame: Arc::new(AtomicU32::new(0)),
            total_frames: Arc::new(AtomicU32::new(total_frames)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
            is_done: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn fraction(&self) -> f32 {
        let cur = self.current_frame.load(Ordering::Relaxed) as f32;
        let tot = self.total_frames.load(Ordering::Relaxed).max(1) as f32;
        (cur / tot).clamp(0.0, 1.0)
    }

    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::Relaxed);
    }
}

pub fn check_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn export_video(
    image: &RgbaImage,
    config: &ExportJobConfig,
    progress: ExportProgress,
) -> Result<()> {
    // Resize image if target resolution differs
    let working_img = if image.width() != config.width || image.height() != config.height {
        image::imageops::resize(
            image,
            config.width,
            config.height,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        image.clone()
    };

    // Make sure dimensions are even for H.264
    let even_w = if working_img.width() % 2 != 0 { working_img.width() - 1 } else { working_img.width() };
    let even_h = if working_img.height() % 2 != 0 { working_img.height() - 1 } else { working_img.height() };
    let final_img = if even_w != working_img.width() || even_h != working_img.height() {
        image::imageops::crop_imm(&working_img, 0, 0, even_w, even_h).to_image()
    } else {
        working_img
    };

    let sim = config.puzzle_kind.create_simulation(
        &final_img,
        config.timing.effective_complexity(),
        config.timing.seed,
    );

    let total_frames = config.timing.total_frames();
    progress.total_frames.store(total_frames, Ordering::Relaxed);

    match config.format {
        VideoFormat::Mp4H264 => {
            export_mp4(&sim, &config.timing, &final_img, &config.output_path, &progress)
        }
        VideoFormat::Gif => {
            export_gif(&sim, &config.timing, &final_img, &config.output_path, &progress)
        }
    }
}

fn export_mp4(
    sim: &Box<dyn PuzzleSimulation>,
    timing: &TimingConfig,
    base_img: &RgbaImage,
    output_path: &Path,
    progress: &ExportProgress,
) -> Result<()> {
    let w = base_img.width();
    let h = base_img.height();
    let fps = timing.fps;
    let total_frames = timing.total_frames();

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pixel_format", "rgba",
            "-video_size", &format!("{}x{}", w, h),
            "-framerate", &fps.to_string(),
            "-i", "-",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-preset", "fast",
            "-crf", "18",
            "-movflags", "+faststart",
        ])
        .arg(output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg. Is ffmpeg installed and in PATH?")?;

    let mut stdin = child.stdin.take().context("Failed to open ffmpeg stdin")?;
    let mut frame_buf = RgbaImage::new(w, h);

    for f in 0..total_frames {
        if progress.is_cancelled.load(Ordering::Relaxed) {
            let _ = child.kill();
            bail!("Export cancelled by user");
        }

        let p = timing.progress_at_frame(f);
        sim.render_frame(p, &mut frame_buf);

        if let Err(e) = stdin.write_all(frame_buf.as_raw()) {
            bail!("Failed to write frame to ffmpeg: {}", e);
        }

        progress.current_frame.store(f + 1, Ordering::Relaxed);
    }

    drop(stdin);
    let status = child.wait()?;
    if !status.success() {
        bail!("ffmpeg exited with non-zero status: {}", status);
    }

    progress.is_done.store(true, Ordering::Relaxed);
    Ok(())
}

fn export_gif(
    sim: &Box<dyn PuzzleSimulation>,
    timing: &TimingConfig,
    base_img: &RgbaImage,
    output_path: &Path,
    progress: &ExportProgress,
) -> Result<()> {
    let w = base_img.width() as u16;
    let h = base_img.height() as u16;
    let total_frames = timing.total_frames();

    let file = File::create(output_path)?;
    let mut encoder = gif::Encoder::new(file, w, h, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    let mut frame_buf = RgbaImage::new(w as u32, h as u32);
    let delay_centisec = ((100.0 / timing.fps as f32).round().max(1.0)) as u16;

    for f in 0..total_frames {
        if progress.is_cancelled.load(Ordering::Relaxed) {
            bail!("Export cancelled by user");
        }

        let p = timing.progress_at_frame(f);
        sim.render_frame(p, &mut frame_buf);

        // Convert RGBA to GIF frame with palette
        let mut pixels = frame_buf.as_raw().clone();
        let mut frame = gif::Frame::from_rgba_speed(w, h, &mut pixels, 10);
        frame.delay = delay_centisec;
        encoder.write_frame(&frame)?;

        progress.current_frame.store(f + 1, Ordering::Relaxed);
    }

    progress.is_done.store(true, Ordering::Relaxed);
    Ok(())
}
