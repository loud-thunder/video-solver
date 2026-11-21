use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Instant;

use eframe::egui::{self, Color32, ColorImage, RichText, TextureHandle, TextureOptions};
use image::RgbaImage;

use crate::puzzle::drawing::SampleImageType;
use crate::puzzle::{
    format_duration_mmss, format_duration_mmss_sub, parse_duration_string, EasingCurve,
    PlaybackPhase, PuzzleCategory, PuzzleKind, PuzzleSimulation, TimingConfig,
};
use crate::video::{
    check_ffmpeg_available, export_video_media, load_video_from_path, ExportJobConfig,
    ExportProgress, SourceMedia, VideoFormat, VideoLoopMode,
};

pub struct VideoSolverApp {
    // Current source media (image or video)
    source_media: SourceMedia,
    source_name: String,
    sample_type: Option<SampleImageType>,

    // Simulation
    selected_kind: PuzzleKind,
    category_filter: Option<PuzzleCategory>,
    current_sim: Option<Box<dyn PuzzleSimulation>>,

    // Timing & rate configuration
    timing: TimingConfig,
    duration_input_str: String,

    // Playback state
    playback_time: f32,
    is_playing: bool,
    playback_speed: f32,
    last_frame_instant: Instant,

    // Preview texture
    preview_texture: Option<TextureHandle>,
    last_rendered_progress: f32,
    last_rendered_media_idx: usize,

    // Export state
    ffmpeg_available: bool,
    active_export: Option<(ExportProgress, std::thread::JoinHandle<anyhow::Result<()>>)>,
    export_message: Option<(String, bool)>, // (message, is_error)
}

impl VideoSolverApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Dark theme default
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let sample = SampleImageType::VibrantMandala;
        let img = sample.generate(600, 600);
        let ffmpeg_available = check_ffmpeg_available();

        let mut app = Self {
            source_media: SourceMedia::from_image(img),
            source_name: sample.name().to_string(),
            sample_type: Some(sample),
            selected_kind: PuzzleKind::JigsawClassic,
            category_filter: None,
            current_sim: None,
            timing: TimingConfig::default(),
            duration_input_str: "05:00".to_string(),
            playback_time: 0.0,
            is_playing: true,
            playback_speed: 1.0,
            last_frame_instant: Instant::now(),
            preview_texture: None,
            last_rendered_progress: -1.0,
            last_rendered_media_idx: usize::MAX,
            ffmpeg_available,
            active_export: None,
            export_message: None,
        };

        app.rebuild_simulation();
        app
    }

    fn rebuild_simulation(&mut self) {
        let base_img = &self.source_media.frames[0];
        let sim = self.selected_kind.create_simulation(
            base_img,
            self.timing.effective_complexity(),
            self.timing.seed,
        );
        self.current_sim = Some(sim);
        self.last_rendered_progress = -1.0;
        self.last_rendered_media_idx = usize::MAX;
    }

    fn set_new_image(&mut self, img: RgbaImage, name: String) {
        self.source_media = SourceMedia::from_image(img);
        self.source_name = name;
        self.rebuild_simulation();
    }

    fn set_new_video(&mut self, media: SourceMedia, name: String) {
        self.source_media = media;
        self.source_name = name;
        self.sample_type = None;
        self.rebuild_simulation();
    }

    fn update_preview_texture(&mut self, ctx: &egui::Context, phase: PlaybackPhase) {
        let progress_key = match phase {
            PlaybackPhase::Solve(p) => p * 0.5,
            PlaybackPhase::Hold => 0.5,
            PlaybackPhase::Rollback(p) => 0.5 + p * 0.5,
        };

        let media_idx = self.source_media.frame_index_at_time(self.playback_time);

        // Avoid re-rendering if progress and video frame haven't noticeably changed
        if (progress_key - self.last_rendered_progress).abs() < 0.0005
            && media_idx == self.last_rendered_media_idx
            && self.preview_texture.is_some()
        {
            return;
        }

        if let Some(sim) = &mut self.current_sim {
            if media_idx != self.last_rendered_media_idx {
                sim.update_source(&self.source_media.frames[media_idx]);
                self.last_rendered_media_idx = media_idx;
            }

            let mut out = RgbaImage::new(self.source_media.width(), self.source_media.height());
            match phase {
                PlaybackPhase::Solve(p) => sim.render_frame(p, &mut out),
                PlaybackPhase::Hold => sim.render_frame(1.0, &mut out),
                PlaybackPhase::Rollback(p) => sim.render_rollback_frame(p, &mut out),
            }

            let color_img = ColorImage::from_rgba_unmultiplied(
                [out.width() as usize, out.height() as usize],
                out.as_raw(),
            );

            match &mut self.preview_texture {
                Some(handle) => handle.set(color_img, TextureOptions::LINEAR),
                None => {
                    self.preview_texture = Some(ctx.load_texture("puzzle_preview", color_img, TextureOptions::LINEAR));
                }
            }
            self.last_rendered_progress = progress_key;
        }
    }

    fn start_export(&mut self, path: PathBuf, format: VideoFormat) {
        let config = ExportJobConfig {
            output_path: path.clone(),
            format,
            width: self.source_media.width(),
            height: self.source_media.height(),
            timing: self.timing.clone(),
            puzzle_kind: self.selected_kind,
        };

        let progress = ExportProgress::new(self.timing.total_frames());
        let progress_clone = progress.clone();
        let media = self.source_media.clone();

        let handle = std::thread::spawn(move || {
            export_video_media(&media, &config, progress_clone)
        });

        self.active_export = Some((progress, handle));
        self.export_message = Some((format!("Exporting to {}...", path.display()), false));
    }

    fn start_batch_export(&mut self, folder: PathBuf) {
        let kinds = PuzzleKind::ALL;
        let timing = self.timing.clone();
        let frames_per_puzzle = timing.total_frames();
        let media = self.source_media.clone();

        let total_frames = frames_per_puzzle * kinds.len() as u32;
        let progress = ExportProgress::new(total_frames);
        let progress_clone = progress.clone();
        let folder_clone = folder.clone();

        let handle = std::thread::spawn(move || {
            for (idx, kind) in kinds.iter().enumerate() {
                if progress_clone.is_cancelled.load(Ordering::Relaxed) {
                    break;
                }
                let clean_name = kind.name().replace(['.', ' ', '/', '&'], "_");
                let out_file = folder_clone.join(format!("{}.mp4", clean_name));

                let config = ExportJobConfig {
                    output_path: out_file,
                    format: VideoFormat::Mp4H264,
                    width: media.width(),
                    height: media.height(),
                    timing: timing.clone(),
                    puzzle_kind: *kind,
                };

                let sub_progress = ExportProgress::new(frames_per_puzzle);
                let sub_clone = sub_progress.clone();
                let parent_prog = progress_clone.clone();

                // Sub-worker updater thread
                let is_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                let is_running_clone = is_running.clone();
                let sub_watcher = std::thread::spawn(move || {
                    while is_running_clone.load(Ordering::Relaxed) {
                        let cur_sub = sub_clone.current_frame.load(Ordering::Relaxed);
                        let global_f = (idx as u32 * frames_per_puzzle) + cur_sub;
                        parent_prog.current_frame.store(global_f, Ordering::Relaxed);
                        std::thread::sleep(std::time::Duration::from_millis(30));
                    }
                });

                let res = export_video_media(&media, &config, sub_progress);
                is_running.store(false, Ordering::Relaxed);
                let _ = sub_watcher.join();

                if let Err(e) = res {
                    return Err(e);
                }
            }
            progress_clone.is_done.store(true, Ordering::Relaxed);
            Ok(())
        });

        self.active_export = Some((progress, handle));
        self.export_message = Some((format!("Batch exporting all 52 puzzles to {}...", folder.display()), false));
    }
}

impl eframe::App for VideoSolverApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = (now - self.last_frame_instant).as_secs_f32();
        self.last_frame_instant = now;

        // Check active export status
        if let Some((_progress, handle)) = &mut self.active_export {
            if handle.is_finished() {
                // Take and inspect result
                let (_, h) = self.active_export.take().unwrap();
                match h.join() {
                    Ok(Ok(())) => {
                        self.export_message = Some(("Export completed successfully!".to_string(), false));
                    }
                    Ok(Err(e)) => {
                        self.export_message = Some((format!("Export error: {}", e), true));
                    }
                    Err(_) => {
                        self.export_message = Some(("Export thread panicked".to_string(), true));
                    }
                }
            }
        }

        // Advance playback if playing
        if self.is_playing && self.active_export.is_none() {
            self.playback_time += dt * self.playback_speed;
            if self.playback_time > self.timing.total_duration_sec {
                self.playback_time = 0.0; // Loop
            }
            ctx.request_repaint();
        }

        let current_phase = self.timing.playback_state_at_time(self.playback_time);
        self.update_preview_texture(ctx, current_phase);

        // Top Header
        egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🎬 Video Solver Studio").strong().color(Color32::from_rgb(90, 180, 255)));
                ui.label(RichText::new("• 52 Algorithmic & Spatial Image/Video Puzzles").italics());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.ffmpeg_available {
                        ui.label(RichText::new("✓ FFmpeg Ready (MP4 H.264 & Video Import)").color(Color32::from_rgb(100, 220, 120)));
                    } else {
                        ui.label(RichText::new("⚠ FFmpeg Not Found (GIF fallback)").color(Color32::from_rgb(255, 180, 70)));
                    }
                });
            });
            ui.add_space(4.0);
        });

        // Left Side Controls
        egui::SidePanel::left("controls_panel")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);

                    // SECTION 1: Image & Video Source
                    ui.group(|ui| {
                        ui.heading("🖼 Media Source (Image or Video)");
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("📂 Load Image...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
                                    .pick_file()
                                {
                                    if let Ok(dyn_img) = image::open(&path) {
                                        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                        self.sample_type = None;
                                        self.set_new_image(dyn_img.to_rgba8(), name);
                                    }
                                }
                            }

                            if ui.button("🎬 Load Video...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Videos", &["mp4", "mov", "webm", "mkv", "avi", "m4v", "gif"])
                                    .pick_file()
                                {
                                    match load_video_from_path(&path) {
                                        Ok(media) => {
                                            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                            self.set_new_video(media, name);
                                        }
                                        Err(e) => {
                                            self.export_message = Some((format!("Failed to load video: {}", e), true));
                                        }
                                    }
                                }
                            }

                            let mut current_sample = self.sample_type.unwrap_or(SampleImageType::VibrantMandala);
                            let sample_label = if self.sample_type.is_some() {
                                current_sample.name()
                            } else {
                                "Procedural..."
                            };

                            egui::ComboBox::from_id_salt("procedural_picker")
                                .selected_text(sample_label)
                                .show_ui(ui, |ui| {
                                    for sample in SampleImageType::all() {
                                        if ui.selectable_value(&mut current_sample, *sample, sample.name()).clicked() {
                                            let img = sample.generate(600, 600);
                                            self.sample_type = Some(*sample);
                                            self.set_new_image(img, sample.name().to_string());
                                        }
                                    }
                                });
                        });

                        ui.add_space(4.0);

                        if self.source_media.is_video() {
                            let cur_frame = self.source_media.frame_index_at_time(self.playback_time);
                            let total_frames = self.source_media.total_video_frames();
                            let pct = ((cur_frame + 1) as f32 / total_frames as f32) * 100.0;

                            ui.label(RichText::new(format!(
                                "🎬 Video: {} ({}x{}, {} frames @ {:.1} fps)",
                                self.source_name,
                                self.source_media.width(),
                                self.source_media.height(),
                                total_frames,
                                self.source_media.source_fps,
                            )).strong().color(Color32::from_rgb(120, 220, 255)));

                            ui.label(RichText::new(format!(
                                "• Active Video Frame: {} / {} ({:.0}%)",
                                cur_frame + 1,
                                total_frames,
                                pct,
                            )).small());

                            ui.add_space(4.0);
                            ui.label("Video Playback Mode:");
                            ui.horizontal(|ui| {
                                ui.selectable_value(
                                    &mut self.source_media.loop_mode,
                                    VideoLoopMode::ContinuousLoop,
                                    "🔁 Continuous (0 → 100, 0 → 100)",
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.selectable_value(
                                    &mut self.source_media.loop_mode,
                                    VideoLoopMode::Bounce,
                                    "↔ Bounce (0 → 100 → 0)",
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Video Speed:");
                                ui.selectable_value(&mut self.source_media.speed, 0.5, "0.5x");
                                ui.selectable_value(&mut self.source_media.speed, 1.0, "1x");
                                ui.selectable_value(&mut self.source_media.speed, 2.0, "2x");
                            });
                        } else {
                            ui.small(format!("Active Image: {} ({}x{})", self.source_name, self.source_media.width(), self.source_media.height()));
                        }
                    });

                    ui.add_space(8.0);

                    // SECTION 2: Puzzle Type & Configuration
                    ui.group(|ui| {
                        ui.heading("🧩 Puzzle Selector (52 Types)");

                        // Category filter buttons
                        ui.horizontal_wrapped(|ui| {
                            if ui.selectable_label(self.category_filter.is_none(), "All (52)").clicked() {
                                self.category_filter = None;
                            }
                            for cat in PuzzleCategory::all() {
                                if ui.selectable_label(self.category_filter == Some(*cat), cat.name()).clicked() {
                                    self.category_filter = Some(*cat);
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Dropdown of matching puzzles
                        egui::ComboBox::from_id_salt("puzzle_picker")
                            .selected_text(self.selected_kind.name())
                            .width(320.0)
                            .show_ui(ui, |ui| {
                                for kind in PuzzleKind::ALL {
                                    if let Some(cat) = self.category_filter {
                                        if kind.category() != cat { continue; }
                                    }
                                    if ui.selectable_value(&mut self.selected_kind, kind, kind.name()).clicked() {
                                        self.rebuild_simulation();
                                    }
                                }
                            });

                        ui.label(RichText::new(self.selected_kind.description()).small().italics());

                        ui.add_space(6.0);

                        // Complexity Slider
                        ui.horizontal(|ui| {
                            ui.label("Complexity:");
                            if ui.add(egui::Slider::new(&mut self.timing.complexity, 1..=20)).changed() {
                                self.rebuild_simulation();
                            }
                        });

                        if ui.checkbox(&mut self.timing.auto_scale_steps, "Auto-scale step density for long videos").changed() {
                            self.rebuild_simulation();
                        }

                        // Seed & Randomize
                        ui.horizontal(|ui| {
                            ui.label("Seed:");
                            if ui.add(egui::DragValue::new(&mut self.timing.seed)).changed() {
                                self.rebuild_simulation();
                            }
                            if ui.button("🎲 Random").clicked() {
                                self.timing.seed = rand::random();
                                self.rebuild_simulation();
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // SECTION 3: Video Duration & Solve Rate Controller
                    ui.group(|ui| {
                        ui.heading("⏱ Duration & Solve Rate");

                        // Duration presets
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Presets:");
                            if ui.button("30s").clicked() {
                                self.timing.total_duration_sec = 30.0;
                                self.duration_input_str = "00:30".to_string();
                            }
                            if ui.button("1m").clicked() {
                                self.timing.total_duration_sec = 60.0;
                                self.duration_input_str = "01:00".to_string();
                            }
                            if ui.button("3m").clicked() {
                                self.timing.total_duration_sec = 180.0;
                                self.duration_input_str = "03:00".to_string();
                            }
                            if ui.button("5m").clicked() {
                                self.timing.total_duration_sec = 300.0;
                                self.duration_input_str = "05:00".to_string();
                            }
                            if ui.button("7m (Max)").clicked() {
                                self.timing.total_duration_sec = 420.0;
                                self.duration_input_str = "07:00".to_string();
                            }
                        });

                        ui.add_space(3.0);

                        ui.horizontal(|ui| {
                            ui.label("Video Length:");

                            let slider = egui::Slider::new(&mut self.timing.total_duration_sec, 0.5..=420.0)
                                .custom_formatter(|val, _| format_duration_mmss(val as f32))
                                .custom_parser(|s| parse_duration_string(s).map(|v| v as f64));
                            let slider_resp = ui.add(slider);
                            if slider_resp.changed() {
                                self.duration_input_str = format_duration_mmss(self.timing.total_duration_sec);
                            }

                            let text_edit_resp = ui.add(
                                egui::TextEdit::singleline(&mut self.duration_input_str)
                                    .desired_width(50.0)
                                    .hint_text("5:57")
                            );
                            if text_edit_resp.changed() {
                                if let Some(parsed) = parse_duration_string(&self.duration_input_str) {
                                    self.timing.total_duration_sec = parsed;
                                }
                            } else if text_edit_resp.lost_focus() {
                                if let Some(parsed) = parse_duration_string(&self.duration_input_str) {
                                    self.timing.total_duration_sec = parsed;
                                    let mins = (parsed / 60.0).floor() as u32;
                                    let secs = parsed % 60.0;
                                    if secs.fract() < 0.001 {
                                        self.duration_input_str = format!("{:02}:{:02}", mins, secs.floor() as u32);
                                    } else {
                                        self.duration_input_str = format!("{:02}:{:04.1}", mins, secs);
                                    }
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Solved Hold:");
                            ui.add(egui::Slider::new(&mut self.timing.hold_duration_sec, 0.0..=10.0).suffix("s"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("FPS:");
                            ui.selectable_value(&mut self.timing.fps, 30, "30 FPS");
                            ui.selectable_value(&mut self.timing.fps, 60, "60 FPS");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Easing:");
                            egui::ComboBox::from_id_salt("easing_combo")
                                .selected_text(self.timing.easing.name())
                                .show_ui(ui, |ui| {
                                    for curve in EasingCurve::all() {
                                        ui.selectable_value(&mut self.timing.easing, *curve, curve.name());
                                    }
                                });
                        });

                        ui.add_space(2.0);
                        if ui.checkbox(&mut self.timing.rollback, "🔄 Roll-back to initial state (novel route)").changed() {
                            self.last_rendered_progress = -1.0;
                        }

                        ui.separator();

                        // Live Rate Readout
                        let total_steps = self.current_sim.as_ref().map(|s| s.total_steps()).unwrap_or(1);
                        let rate = self.timing.solve_rate_steps_per_sec(total_steps);
                        let total_frames = self.timing.total_frames();

                        ui.label(RichText::new(format!("⚡ Rate: {:.2} moves / second", rate)).strong().color(Color32::from_rgb(255, 215, 80)));
                        ui.label(format!("• Exact Duration: {} ({:.2}s)", format_duration_mmss(self.timing.total_duration_sec), self.timing.total_duration_sec));
                        if self.timing.rollback {
                            ui.label(RichText::new("• 3-Phase Loop Active:").color(Color32::from_rgb(140, 210, 255)));
                            ui.label(format!("   ↳ Phase 1 (Solve): {:.2}s (45%)", self.timing.total_duration_sec * 0.45));
                            ui.label(format!("   ↳ Phase 2 (Hold Solved): {:.2}s (10%)", self.timing.total_duration_sec * 0.10));
                            ui.label(format!("   ↳ Phase 3 (Roll-back): {:.2}s (45%)", self.timing.total_duration_sec * 0.45));
                        } else {
                            ui.label(format!("• Active Solve Window: {} (Hold: {:.1}s)", format_duration_mmss(self.timing.solve_duration_sec()), self.timing.hold_duration_sec));
                        }
                        ui.label(format!("• Total Discrete Steps: {}", total_steps));
                        ui.label(format!("• Total Frames: {} frames @ {} fps", total_frames, self.timing.fps));
                    });

                    ui.add_space(8.0);

                    // SECTION 4: Video Export
                    ui.group(|ui| {
                        ui.heading("💾 Video Export");

                        let is_exporting = self.active_export.is_some();
                        let gif_allowed = self.timing.total_duration_sec <= 60.0;

                        ui.horizontal(|ui| {
                            let mp4_btn = ui.add_enabled(
                                !is_exporting && self.ffmpeg_available,
                                egui::Button::new("🎥 Export MP4 (H.264)")
                            );
                            if mp4_btn.clicked() {
                                let default_name = format!("{}.mp4", self.selected_kind.name().replace(['.', ' ', '/', '&'], "_"));
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name(&default_name)
                                    .add_filter("MP4 Video", &["mp4"])
                                    .save_file()
                                {
                                    self.start_export(path, VideoFormat::Mp4H264);
                                }
                            }

                            let gif_btn = ui.add_enabled(
                                !is_exporting && gif_allowed,
                                egui::Button::new("🎞 Export GIF")
                            );
                            if gif_btn.clicked() {
                                let default_name = format!("{}.gif", self.selected_kind.name().replace(['.', ' ', '/', '&'], "_"));
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name(&default_name)
                                    .add_filter("GIF Animation", &["gif"])
                                    .save_file()
                                {
                                    self.start_export(path, VideoFormat::Gif);
                                }
                            }
                        });

                        if !gif_allowed {
                            ui.small(RichText::new("ℹ GIF disabled for videos > 1 min (use MP4 for up to 7 min)").color(Color32::from_rgb(255, 190, 80)));
                        }

                        ui.add_space(4.0);

                        let batch_btn = ui.add_enabled(
                            !is_exporting && self.ffmpeg_available,
                            egui::Button::new("📦 Batch Export All 52 Puzzles...")
                        );
                        if batch_btn.clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.start_batch_export(folder);
                            }
                        }

                        // Export Progress Banner
                        if let Some((progress, _)) = &self.active_export {
                            ui.add_space(6.0);
                            let frac = progress.fraction();
                            ui.add(egui::ProgressBar::new(frac).show_percentage());
                            ui.horizontal(|ui| {
                                ui.label(format!("Frame {} / {}", progress.current_frame.load(Ordering::Relaxed), progress.total_frames.load(Ordering::Relaxed)));
                                if ui.button("Cancel").clicked() {
                                    progress.cancel();
                                }
                            });
                        }

                        if let Some((msg, is_err)) = &self.export_message {
                            ui.add_space(4.0);
                            let color = if *is_err { Color32::from_rgb(255, 90, 90) } else { Color32::from_rgb(120, 230, 140) };
                            ui.label(RichText::new(msg).color(color).small());
                        }
                    });

                    ui.add_space(8.0);
                });
            });

        // Bottom Transport Bar
        egui::TopBottomPanel::bottom("transport_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Play / Pause
                let play_text = if self.is_playing { "⏸ Pause" } else { "▶ Play" };
                if ui.button(play_text).clicked() {
                    self.is_playing = !self.is_playing;
                }

                // Restart
                if ui.button("⏮ Restart").clicked() {
                    self.playback_time = 0.0;
                }

                // Scrubber Slider
                let mut t = self.playback_time;
                let slider = egui::Slider::new(&mut t, 0.0..=self.timing.total_duration_sec)
                    .custom_formatter(|val, _| format_duration_mmss_sub(val as f32))
                    .show_value(true);

                let response = ui.add_sized([ui.available_width() - 320.0, 20.0], slider);
                if response.changed() {
                    self.playback_time = t;
                    ctx.request_repaint();
                }

                ui.label(RichText::new(format!("{}/{}", format_duration_mmss(self.playback_time), format_duration_mmss(self.timing.total_duration_sec))).monospace());

                // Speed buttons
                ui.label("Speed:");
                ui.selectable_value(&mut self.playback_speed, 0.5, "0.5x");
                ui.selectable_value(&mut self.playback_speed, 1.0, "1x");
                ui.selectable_value(&mut self.playback_speed, 2.0, "2x");
            });
            ui.add_space(4.0);
        });

        // Central Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);

                // Simulation status banner
                if let Some(sim) = &self.current_sim {
                    let label = match current_phase {
                        PlaybackPhase::Solve(p) => sim.status_label(p),
                        PlaybackPhase::Hold => format!("✨ Solved: {} (Holding)", self.selected_kind.name()),
                        PlaybackPhase::Rollback(p) => format!("🔄 Roll-back: {:.0}% deconstructed (novel route)", p * 100.0),
                    };
                    ui.label(RichText::new(label).strong().size(15.0).color(Color32::from_rgb(220, 230, 255)));
                }

                ui.add_space(4.0);

                // Render Canvas
                if let Some(tex) = &self.preview_texture {
                    let avail_size = ui.available_size();
                    let aspect = tex.aspect_ratio();

                    let mut render_w = avail_size.x.min(avail_size.y * aspect) - 20.0;
                    let mut render_h = render_w / aspect;

                    if render_h > avail_size.y - 20.0 {
                        render_h = avail_size.y - 20.0;
                        render_w = render_h * aspect;
                    }

                    if render_w > 10.0 && render_h > 10.0 {
                        ui.image((tex.id(), egui::vec2(render_w, render_h)));
                    }
                }
            });
        });
    }
}
