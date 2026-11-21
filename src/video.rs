use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use image::RgbaImage;

use crate::puzzle::{PuzzleKind, PuzzleSimulation, TimingConfig};

pub const MAX_IMPORTED_VIDEO_FRAMES: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4H264,
    Gif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoLoopMode {
    /// Plays frames 0 -> N-1, then jumps back to 0 and repeats continuously (0 -> 100, 0 -> 100)
    ContinuousLoop,
    /// Plays frames 0 -> N-1, then reverses N-1 -> 0 and repeats (0 -> 100 -> 0)
    Bounce,
}

impl VideoLoopMode {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            VideoLoopMode::ContinuousLoop => "Continuous (0 → 100, 0 → 100)",
            VideoLoopMode::Bounce => "Bounce (0 → 100 → 0)",
        }
    }
}

#[derive(Clone)]
pub struct SourceMedia {
    pub frames: Vec<RgbaImage>,
    pub source_fps: f32,
    pub loop_mode: VideoLoopMode,
    pub speed: f32,
}

impl SourceMedia {
    pub fn from_image(image: RgbaImage) -> Self {
        Self {
            frames: vec![image],
            source_fps: 30.0,
            loop_mode: VideoLoopMode::ContinuousLoop,
            speed: 1.0,
        }
    }

    pub fn is_video(&self) -> bool {
        self.frames.len() > 1
    }

    pub fn width(&self) -> u32 {
        self.frames.first().map(|f| f.width()).unwrap_or(0)
    }

    pub fn height(&self) -> u32 {
        self.frames.first().map(|f| f.height()).unwrap_or(0)
    }

    pub fn total_video_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn frame_index_at_time(&self, time_sec: f32) -> usize {
        let n = self.frames.len();
        if n <= 1 {
            return 0;
        }

        let effective_fps = (self.source_fps * self.speed).max(0.1);
        let raw_idx = (time_sec.max(0.0) * effective_fps).floor() as usize;

        match self.loop_mode {
            VideoLoopMode::ContinuousLoop => raw_idx % n,
            VideoLoopMode::Bounce => {
                let cycle = 2 * (n - 1);
                let pos = raw_idx % cycle;
                if pos < n {
                    pos
                } else {
                    cycle - pos
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn frame_at_time(&self, time_sec: f32) -> &RgbaImage {
        let idx = self.frame_index_at_time(time_sec);
        &self.frames[idx]
    }
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

pub fn load_video_from_path(path: &Path) -> Result<SourceMedia> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext == "gif" {
        load_gif_video(path)
    } else {
        load_ffmpeg_video(path)
    }
}

fn load_gif_video(path: &Path) -> Result<SourceMedia> {
    let file = File::open(path)?;
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder.read_info(file)?;

    let mut raw_frames = Vec::new();
    let mut total_delay_cs: u32 = 0;
    let mut num_delays = 0;

    while let Some(frame) = reader.read_next_frame()? {
        let w = frame.width as u32;
        let h = frame.height as u32;
        if let Some(img) = RgbaImage::from_raw(w, h, frame.buffer.to_vec()) {
            raw_frames.push(img);
            total_delay_cs += frame.delay as u32;
            num_delays += 1;
        }
    }

    if raw_frames.is_empty() {
        bail!("GIF file contains no frames: {}", path.display());
    }

    let avg_delay_cs = if num_delays > 0 {
        (total_delay_cs as f32 / num_delays as f32).max(1.0)
    } else {
        10.0
    };
    let source_fps = (100.0 / avg_delay_cs).clamp(1.0, 60.0);

    // Subsample if frames exceed MAX_IMPORTED_VIDEO_FRAMES
    let final_frames = if raw_frames.len() > MAX_IMPORTED_VIDEO_FRAMES {
        let step = raw_frames.len() as f32 / MAX_IMPORTED_VIDEO_FRAMES as f32;
        (0..MAX_IMPORTED_VIDEO_FRAMES)
            .map(|i| {
                let idx = ((i as f32 * step).round() as usize).min(raw_frames.len() - 1);
                raw_frames[idx].clone()
            })
            .collect()
    } else {
        raw_frames
    };

    Ok(SourceMedia {
        frames: final_frames,
        source_fps,
        loop_mode: VideoLoopMode::ContinuousLoop,
        speed: 1.0,
    })
}

fn probe_video(path: &Path) -> Result<(u32, u32, f32, f32)> {
    // ffprobe -v error -select_streams v:0 -show_entries stream=width,height,r_frame_rate,duration -of csv=p=0:s=x <path>
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate,duration",
            "-of", "csv=p=0:s=x",
        ])
        .arg(path)
        .output()
        .context("Failed to run ffprobe. Is ffmpeg installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split('x').collect();
    if parts.len() >= 3 {
        let w: u32 = parts[0].trim().parse().unwrap_or(640);
        let h: u32 = parts[1].trim().parse().unwrap_or(480);

        // r_frame_rate can be "30/1" or "29.97" or "24/1"
        let rate_str = parts[2].trim();
        let fps = if let Some((num, den)) = rate_str.split_once('/') {
            let n: f32 = num.parse().unwrap_or(30.0);
            let d: f32 = den.parse().unwrap_or(1.0);
            (n / d.max(0.001)).clamp(1.0, 120.0)
        } else {
            rate_str.parse().unwrap_or(30.0)
        };

        let duration = if parts.len() >= 4 {
            parts[3].trim().parse().unwrap_or(5.0)
        } else {
            5.0
        };

        return Ok((w, h, fps, duration));
    }

    // Fallback: default probe
    Ok((640, 480, 30.0, 5.0))
}

fn load_ffmpeg_video(path: &Path) -> Result<SourceMedia> {
    let (w, h, native_fps, duration_sec) = probe_video(path).unwrap_or((640, 480, 30.0, 5.0));

    // Determine target extraction fps to cover 0% to 100% of video up to MAX_IMPORTED_VIDEO_FRAMES
    let estimated_total_frames = (duration_sec * native_fps).round() as usize;
    let target_fps = if estimated_total_frames > MAX_IMPORTED_VIDEO_FRAMES && duration_sec > 0.1 {
        (MAX_IMPORTED_VIDEO_FRAMES as f32 / duration_sec).clamp(1.0, native_fps)
    } else {
        native_fps.clamp(1.0, 60.0)
    };

    let mut child = Command::new("ffmpeg")
        .args([
            "-i",
        ])
        .arg(path)
        .args([
            "-vf", &format!("fps={}", target_fps),
            "-f", "rawvideo",
            "-pix_fmt", "rgba",
            "-s", &format!("{}x{}", w, h),
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn ffmpeg to decode video")?;

    let mut stdout = child.stdout.take().context("Failed to open ffmpeg stdout")?;
    let frame_bytes = (w * h * 4) as usize;
    let mut frames = Vec::new();

    loop {
        let mut buf = vec![0u8; frame_bytes];
        if stdout.read_exact(&mut buf).is_ok() {
            if let Some(img) = RgbaImage::from_raw(w, h, buf) {
                frames.push(img);
                if frames.len() >= MAX_IMPORTED_VIDEO_FRAMES {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    if frames.is_empty() {
        bail!("Failed to decode any video frames from {}", path.display());
    }

    Ok(SourceMedia {
        frames,
        source_fps: target_fps,
        loop_mode: VideoLoopMode::ContinuousLoop,
        speed: 1.0,
    })
}

pub fn export_video(
    image: &RgbaImage,
    config: &ExportJobConfig,
    progress: ExportProgress,
) -> Result<()> {
    let media = SourceMedia::from_image(image.clone());
    export_video_media(&media, config, progress)
}

pub fn export_video_media(
    media: &SourceMedia,
    config: &ExportJobConfig,
    progress: ExportProgress,
) -> Result<()> {
    if media.frames.is_empty() {
        bail!("No source frames provided for export");
    }

    // Resize/crop all source media frames if necessary

    let processed_frames: Vec<RgbaImage> = media.frames.iter().map(|frame| {
        let working_img = if frame.width() != config.width || frame.height() != config.height {
            image::imageops::resize(
                frame,
                config.width,
                config.height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            frame.clone()
        };

        let even_w = if working_img.width() % 2 != 0 { working_img.width() - 1 } else { working_img.width() };
        let even_h = if working_img.height() % 2 != 0 { working_img.height() - 1 } else { working_img.height() };
        if even_w != working_img.width() || even_h != working_img.height() {
            image::imageops::crop_imm(&working_img, 0, 0, even_w, even_h).to_image()
        } else {
            working_img
        }
    }).collect();

    let adjusted_media = SourceMedia {
        frames: processed_frames,
        source_fps: media.source_fps,
        loop_mode: media.loop_mode,
        speed: media.speed,
    };

    let first_img = &adjusted_media.frames[0];
    let mut sim = config.puzzle_kind.create_simulation(
        first_img,
        config.timing.effective_complexity(),
        config.timing.seed,
    );

    let total_frames = config.timing.total_frames();
    progress.total_frames.store(total_frames, Ordering::Relaxed);

    match config.format {
        VideoFormat::Mp4H264 => {
            export_mp4(&mut sim, &config.timing, &adjusted_media, &config.output_path, &progress)
        }
        VideoFormat::Gif => {
            export_gif(&mut sim, &config.timing, &adjusted_media, &config.output_path, &progress)
        }
    }
}

fn export_mp4(
    sim: &mut Box<dyn PuzzleSimulation>,
    timing: &TimingConfig,
    media: &SourceMedia,
    output_path: &Path,
    progress: &ExportProgress,
) -> Result<()> {
    let w = media.width();
    let h = media.height();
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
    let mut last_media_idx = usize::MAX;

    for f in 0..total_frames {
        if progress.is_cancelled.load(Ordering::Relaxed) {
            let _ = child.kill();
            bail!("Export cancelled by user");
        }

        let time_sec = f as f32 / timing.fps as f32;
        let media_idx = media.frame_index_at_time(time_sec);
        if media_idx != last_media_idx {
            sim.update_source(&media.frames[media_idx]);
            last_media_idx = media_idx;
        }

        match timing.playback_state_at_frame(f) {
            crate::puzzle::PlaybackPhase::Solve(p) => sim.render_frame(p, &mut frame_buf),
            crate::puzzle::PlaybackPhase::Hold => sim.render_frame(1.0, &mut frame_buf),
            crate::puzzle::PlaybackPhase::Rollback(p) => sim.render_rollback_frame(p, &mut frame_buf),
        }

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
    sim: &mut Box<dyn PuzzleSimulation>,
    timing: &TimingConfig,
    media: &SourceMedia,
    output_path: &Path,
    progress: &ExportProgress,
) -> Result<()> {
    let w = media.width() as u16;
    let h = media.height() as u16;
    let total_frames = timing.total_frames();

    let file = File::create(output_path)?;
    let mut encoder = gif::Encoder::new(file, w, h, &[])?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    let mut frame_buf = RgbaImage::new(w as u32, h as u32);
    let delay_centisec = ((100.0 / timing.fps as f32).round().max(1.0)) as u16;
    let mut last_media_idx = usize::MAX;

    for f in 0..total_frames {
        if progress.is_cancelled.load(Ordering::Relaxed) {
            bail!("Export cancelled by user");
        }

        let time_sec = f as f32 / timing.fps as f32;
        let media_idx = media.frame_index_at_time(time_sec);
        if media_idx != last_media_idx {
            sim.update_source(&media.frames[media_idx]);
            last_media_idx = media_idx;
        }

        match timing.playback_state_at_frame(f) {
            crate::puzzle::PlaybackPhase::Solve(p) => sim.render_frame(p, &mut frame_buf),
            crate::puzzle::PlaybackPhase::Hold => sim.render_frame(1.0, &mut frame_buf),
            crate::puzzle::PlaybackPhase::Rollback(p) => sim.render_rollback_frame(p, &mut frame_buf),
        }

        let mut pixels = frame_buf.as_raw().clone();
        let mut frame = gif::Frame::from_rgba_speed(w, h, &mut pixels, 10);
        frame.delay = delay_centisec;
        encoder.write_frame(&frame)?;

        progress.current_frame.store(f + 1, Ordering::Relaxed);
    }

    progress.is_done.store(true, Ordering::Relaxed);
    Ok(())
}
