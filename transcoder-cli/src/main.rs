//! Industrial Transcoder CLI
//!
//! Command-line interface for the multi-job transcoder

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::sync::Arc;
use transcoder_core::*;
use tracing::info;

#[derive(Parser)]
#[command(name = "transcoder")]
#[command(about = "Industrial multi-job transcoder for professional media workflows", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transcode a single file
    Transcode {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Codec preset (prores_hq, prores_422, prores_lt, h264_high)
        #[arg(short, long, default_value = "prores_hq")]
        preset: String,

        /// Worker threads (default: CPU count - 1)
        #[arg(short, long)]
        workers: Option<usize>,
    },

    /// List available codec presets
    Presets,

    /// Verify FFmpeg installation
    Verify,

    /// Show system information
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    transcoder_core::init()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Transcode {
            input,
            output,
            preset,
            workers,
        } => {
            transcode_file(input, output, preset, workers).await?;
        }
        Commands::Presets => {
            show_presets();
        }
        Commands::Verify => {
            verify_ffmpeg().await?;
        }
        Commands::Info => {
            show_info();
        }
    }

    Ok(())
}

async fn transcode_file(
    input: PathBuf,
    output: PathBuf,
    preset_name: String,
    worker_count: Option<usize>,
) -> Result<()> {
    println!("{}", "=== Industrial Transcoder ===".bright_cyan().bold());
    println!();

    // Get preset
    let presets = config::CodecPreset::all_presets();
    let preset = presets
        .get(&preset_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown preset: {}", preset_name))?;

    println!("{} {}", "Preset:".bright_yellow(), preset.name);
    println!("{} {:?}", "Input:".bright_yellow(), input);
    println!("{} {:?}", "Output:".bright_yellow(), output);
    println!();

    // Create job
    let config_json = serde_json::to_value(&preset.config)?;
    let job = job::Job::new(
        input,
        output,
        config_json,
        job::Priority::Normal,
    );

    // Create queue and add job
    let queue = Arc::new(JobQueue::new());
    let job_id = queue.add_job(job).await?;

    // Create progress reporter
    let progress_reporter = ProgressReporter::new(100);
    let mut receiver = progress_reporter.subscribe();

    // Spawn progress display task
    let progress_task = tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match event {
                ProgressEvent::JobStarted { .. } => {
                    println!("{}", "Starting transcode...".bright_green());
                }
                ProgressEvent::JobProgress { progress, fps, .. } => {
                    let bar_width = 40;
                    let filled = ((progress / 100.0) * bar_width as f32) as usize;
                    let empty = bar_width - filled;
                    
                    let bar = format!(
                        "[{}{}]",
                        "=".repeat(filled).bright_green(),
                        " ".repeat(empty)
                    );
                    
                    let fps_str = if let Some(fps) = fps {
                        format!(" {:.1} fps", fps)
                    } else {
                        String::new()
                    };
                    
                    print!("\r{} {:.1}%{}", bar, progress, fps_str);
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
                ProgressEvent::JobCompleted { duration_seconds, .. } => {
                    println!();
                    println!(
                        "{} Completed in {}s",
                        "✓".bright_green().bold(),
                        duration_seconds
                    );
                }
                ProgressEvent::JobFailed { error, .. } => {
                    println!();
                    println!("{} {}", "✗".bright_red().bold(), error.bright_red());
                }
                _ => {}
            }
        }
    });

    // Create and start worker pool
    let mut pool = WorkerPool::new(queue.clone(), progress_reporter, worker_count)?;
    pool.start().await?;

    // Wait for job to complete
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        if let Some(job) = queue.get_job(&job_id) {
            if job.is_finished() {
                break;
            }
        }
    }

    // Stop worker pool
    pool.stop().await;

    // Wait for progress task to finish
    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        progress_task
    ).await;

    println!();
    Ok(())
}

fn show_presets() {
    println!("{}", "=== Available Codec Presets ===".bright_cyan().bold());
    println!();

    let presets = config::CodecPreset::all_presets();
    let mut preset_list: Vec<_> = presets.values().collect();
    preset_list.sort_by_key(|p| &p.name);

    for preset in preset_list {
        println!("{}", preset.name.bright_green().bold());
        println!("  {}", preset.description.dimmed());
        println!("  Video: {:?}", preset.config.video_codec);
        println!("  Audio: {:?}", preset.config.audio_codec);
        println!("  Container: {:?}", preset.config.container);
        println!();
    }
}

async fn verify_ffmpeg() -> Result<()> {
    println!("{}", "=== Verifying FFmpeg ===".bright_cyan().bold());
    println!();

    let transcoder = Transcoder::new()?;
    let version = transcoder.verify().await?;

    println!("{} FFmpeg is installed and working", "✓".bright_green().bold());
    println!("{}", version.dimmed());
    println!();

    Ok(())
}

fn show_info() {
    println!("{}", "=== System Information ===".bright_cyan().bold());
    println!();

    let platform = Platform::current();
    println!("{} {}", "Platform:".bright_yellow(), platform.name());
    
    let cpu_count = crate::platform::cpu_count();
    println!("{} {}", "CPU Cores:".bright_yellow(), cpu_count);

    if let Some(mem_mb) = crate::platform::available_memory_mb() {
        println!("{} {} MB", "Available Memory:".bright_yellow(), mem_mb);
    }

    println!();

    // Check FFmpeg
    match platform::find_ffmpeg() {
        Ok(path) => {
            println!("{} {:?}", "FFmpeg:".bright_green(), path);
        }
        Err(_) => {
            println!("{} Not found", "FFmpeg:".bright_red());
        }
    }

    println!();
}

