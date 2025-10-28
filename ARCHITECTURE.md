# Industrial Transcoder Architecture

**Cross-platform multi-job transcoding system**

## Overview

Industrial Transcoder is built with a **multi-layered architecture** designed for cross-platform compatibility, high performance, and maintainability.

```
┌─────────────────────────────────────────────────┐
│              React UI (TypeScript)              │
│         Modern dashboard with real-time         │
│            progress and job management          │
└─────────────────┬───────────────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────────────┐
│          Tauri Backend (Rust)                   │
│       Command handlers & state management       │
└─────────────────┬───────────────────────────────┘
                  │ Library API
┌─────────────────▼───────────────────────────────┐
│       Transcoder Core (Rust Library)            │
│   • Job Queue (Priority-based)                  │
│   • Worker Pool (Multi-threaded)                │
│   • Platform Abstraction Layer                  │
│   • Progress Reporter (Event Broadcasting)      │
└─────────────────┬───────────────────────────────┘
                  │ Process Spawning
┌─────────────────▼───────────────────────────────┐
│            External Tools                       │
│       FFmpeg (transcoding engine)               │
│       FFprobe (media analysis)                  │
└─────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Transcoder Core (`transcoder-core/`)

The heart of the system - a cross-platform Rust library.

#### **Job Queue** (`queue.rs`)
- **Priority-based queue** using `BinaryHeap`
- **Thread-safe** with `DashMap` and `RwLock`
- **Job lifecycle**: Pending → Running → Completed/Failed/Cancelled
- **Statistics tracking**: counts by status

**Key Features:**
- Concurrent access from multiple workers
- Priority levels: Low, Normal, High, Urgent
- FIFO within same priority
- Automatic job validation

#### **Worker Pool** (`worker.rs`)
- **Multi-threaded** job processing
- **Configurable worker count** (default: CPU cores - 1)
- **Automatic job distribution**
- **Graceful shutdown**

**Architecture:**
```rust
WorkerPool {
    workers: Vec<Worker>,      // N worker threads
    queue: Arc<JobQueue>,       // Shared job queue
    running: AtomicBool,        // Control flag
    active_workers: AtomicUsize // Busy worker count
}
```

**Worker Loop:**
1. Poll queue for next job
2. Mark job as running
3. Execute transcode
4. Report progress
5. Update job status
6. Repeat

#### **Transcoder** (`transcode.rs`)
- **FFmpeg wrapper** for media processing
- **Progress parsing** from FFmpeg output
- **Duration calculation** via FFprobe
- **Error handling** with detailed messages

**Progress Tracking:**
- Parses FFmpeg stderr for time and FPS
- Calculates percentage based on duration
- Real-time callbacks to UI

#### **Platform Abstraction** (`platform.rs`)
- **Cross-platform utilities**
- **FFmpeg detection** in system PATH
- **Path normalization** (Windows vs Unix)
- **CPU and memory detection**

**Platform Detection:**
```rust
Platform::current() -> Platform {
    MacOS | Windows | Linux
}
```

#### **Configuration** (`config.rs`)
- **Codec presets** (ProRes, H.264, etc.)
- **FFmpeg argument generation**
- **Extensible preset system**

**Built-in Presets:**
- ProRes HQ (broadcast quality)
- ProRes 422 (standard)
- ProRes LT (offline editing)
- H.264 High (delivery)

#### **Progress Reporter** (`progress.rs`)
- **Event broadcasting** via `tokio::broadcast`
- **Multiple subscribers** support
- **Event types**: JobStarted, JobProgress, JobCompleted, etc.

---

### 2. Tauri Integration (`src-tauri/`)

Desktop application backend using Tauri framework.

#### **Main Application** (`main.rs`)
- **State management**: JobQueue, WorkerPool, ProgressReporter
- **Event emitter**: Broadcasts progress to frontend
- **Command registration**: All Tauri commands

#### **Commands** (`transcoder_commands.rs`)
- **System commands**: get_system_info, verify_ffmpeg
- **Job commands**: add_job, get_job, cancel_job
- **Worker commands**: start_workers, stop_workers
- **Statistics**: get_queue_stats, get_worker_status

**Command Flow:**
```
Frontend → invoke() → Tauri Command → Core Library → FFmpeg
                                    ↓
                         Progress Events → Event Emitter
                                    ↓
                         Frontend (listen)
```

---

### 3. React UI (`src/`)

Modern web-based interface built with React and TypeScript.

#### **Components**
- **TranscoderDashboard**: Main dashboard with stats and controls
- **JobList**: Display all jobs with progress bars
- **AddJobDialog**: Form for creating new jobs

#### **API Layer** (`api/transcoder-api.ts`)
- **Type-safe** TypeScript API
- **Async/await** patterns
- **Event subscription** for progress
- **Helper functions** for formatting

**API Structure:**
```typescript
// System
getSystemInfo(): Promise<SystemInfo>
verifyFfmpeg(): Promise<string>

// Jobs
addJob(request): Promise<JobId>
getAllJobs(): Promise<Job[]>
cancelJob(jobId): Promise<void>

// Workers
startWorkers(count?): Promise<void>
stopWorkers(): Promise<void>

// Progress
subscribeProgress(callback): Promise<UnlistenFn>
```

---

### 4. CLI Tool (`transcoder-cli/`)

Command-line interface for scripting and automation.

**Commands:**
```bash
transcoder transcode --input FILE --output FILE --preset NAME
transcoder presets
transcoder verify
transcoder info
```

**Features:**
- **Progress bar** in terminal
- **Colored output** for better UX
- **Error handling** with exit codes

---

## Data Flow

### Job Submission Flow

```
1. User Action (UI or CLI)
   ↓
2. Create Job Object
   ↓
3. Add to Queue (with validation)
   ↓
4. Worker picks up job
   ↓
5. FFmpeg execution
   ↓
6. Progress parsing & reporting
   ↓
7. Job completion/failure
   ↓
8. UI update via events
```

### Progress Flow

```
FFmpeg stderr → Regex parsing → Progress calculation
                                       ↓
                         ProgressEvent creation
                                       ↓
                         Broadcast to subscribers
                                       ↓
                    UI updates (progress bars, stats)
```

---

## Concurrency Model

### Thread Safety

- **Job Queue**: `DashMap` + `RwLock` for thread-safe access
- **Worker Pool**: One Tokio task per worker
- **Progress Reporter**: `broadcast` channel for pub/sub

### Async/Await

- **Tokio runtime** for async operations
- **Non-blocking I/O** for FFmpeg process handling
- **Concurrent workers** process jobs in parallel

### Synchronization

```rust
// Job Queue uses DashMap for lock-free reads
jobs: Arc<DashMap<JobId, Job>>

// Pending queue uses RwLock for exclusive writes
pending_queue: Arc<RwLock<BinaryHeap<PriorityJob>>>

// Worker pool uses atomic for state
running: Arc<AtomicBool>
active_workers: Arc<AtomicUsize>
```

---

## Cross-Platform Strategy

### Platform Detection

Runtime detection with compile-time optimizations:
```rust
#[cfg(target_os = "macos")]
fn platform_specific() { ... }

#[cfg(target_os = "windows")]
fn platform_specific() { ... }

#[cfg(target_os = "linux")]
fn platform_specific() { ... }
```

### Path Handling

- **Normalization**: Convert paths to platform-appropriate format
- **Absolute paths**: Resolve relative paths automatically
- **Validation**: Check file existence before processing

### Executable Detection

- **which** crate for finding FFmpeg in PATH
- **Platform-specific extensions**: `.exe` on Windows
- **Fallback locations**: Common install paths

---

## Performance Optimizations

### Multi-Threading

- **Worker pool** processes multiple jobs simultaneously
- **CPU-based worker count**: Optimal parallelism
- **Job priority**: High-priority jobs jump the queue

### Memory Management

- **Streaming**: FFmpeg processes files without loading into RAM
- **Shared state**: `Arc` for minimal cloning
- **Efficient data structures**: `DashMap` for lock-free reads

### Progress Reporting

- **Sampling**: Don't report every line of FFmpeg output
- **Broadcast channel**: Single producer, multiple consumers
- **Bounded capacity**: Prevent memory growth

---

## Error Handling

### Error Types

```rust
pub enum TranscodeError {
    Io(std::io::Error),
    FfmpegNotFound,
    FfmpegFailed(String),
    InvalidInput(String),
    InvalidOutput(String),
    JobNotFound(String),
    Cancelled,
    // ...
}
```

### Recovery Strategies

- **Validation**: Check inputs before starting
- **Graceful degradation**: Continue processing other jobs if one fails
- **Error propagation**: Detailed error messages to UI
- **Logging**: Comprehensive tracing for debugging

---

## Testing Strategy

### Unit Tests

- **Per-module tests**: Each module has `#[cfg(test)]` section
- **Mock data**: Test without requiring FFmpeg
- **Edge cases**: Empty queues, invalid paths, etc.

### Integration Tests

- **Full workflow**: Add job → Process → Verify output
- **Cross-platform**: Test on macOS, Windows, Linux
- **Concurrency**: Multiple workers, multiple jobs

### Performance Tests

- **Benchmark**: Job throughput
- **Memory profiling**: Check for leaks
- **Load testing**: Many jobs, many workers

---

## Extensibility

### Adding New Codecs

1. Add codec enum variant to `config.rs`
2. Implement FFmpeg argument generation
3. Create preset in `CodecPreset::all_presets()`
4. Update UI preset selector

### Custom Workflows

```rust
// Example: Custom preprocessing
impl Transcoder {
    pub async fn custom_workflow(&self, ...) {
        // 1. Preprocess
        self.preprocess_file(...).await?;
        
        // 2. Transcode
        self.transcode(...).await?;
        
        // 3. Postprocess
        self.postprocess_file(...).await?;
    }
}
```

### Plugin Architecture (Future)

Potential for plugin system:
- **Dynamic loading**: Load plugins at runtime
- **Custom filters**: Add FFmpeg filters
- **Custom presets**: User-defined codec configs

---

## Security Considerations

### Path Validation

- **Canonicalize paths**: Prevent directory traversal
- **Check permissions**: Verify read/write access
- **Sandboxing**: Tauri restricts filesystem access

### Command Injection

- **No shell execution**: Direct process spawning
- **Argument validation**: Sanitize user inputs
- **Whitelist approach**: Only allow known FFmpeg options

### Resource Limits

- **Worker count limits**: Prevent CPU exhaustion
- **Queue size limits**: Prevent memory exhaustion
- **Timeout handling**: Kill hung processes

---

## Monitoring & Observability

### Logging

- **tracing** crate for structured logging
- **Log levels**: ERROR, WARN, INFO, DEBUG, TRACE
- **Environment control**: `RUST_LOG=info`

### Metrics

- **Queue statistics**: Real-time job counts
- **Worker utilization**: Active vs idle workers
- **Job duration**: Track completion times

### Events

- **Progress events**: Real-time updates
- **Job lifecycle events**: Started, completed, failed
- **System events**: Workers started/stopped

---

## Deployment

### Binary Distribution

**macOS:**
- `.app` bundle with code signing
- Notarization for Gatekeeper
- Universal binary (Intel + ARM)

**Windows:**
- `.msi` installer
- Code signing certificate
- Automatic FFmpeg bundling option

**Linux:**
- AppImage (portable)
- `.deb` / `.rpm` packages
- Flatpak (future)

### Dependencies

**Runtime:**
- FFmpeg (must be installed separately)
- System libraries (WebKit on Linux)

**Bundled:**
- All Rust dependencies compiled in
- No separate runtime needed

---

## Future Enhancements

### Planned Features

1. **GPU acceleration**: Use FFmpeg with NVENC/AMF
2. **Distributed processing**: Network-based worker pool
3. **Cloud storage**: S3, Azure Blob, GCS integration
4. **Watch folders**: Auto-process new files
5. **REST API**: HTTP server mode
6. **Batch templates**: Save/load job configurations
7. **Quality validation**: Automatic QC checks
8. **Format detection**: Auto-detect optimal settings

### Performance Improvements

- **Native BEXT writing**: Replace Python script
- **MXF parsing**: Native Rust implementation
- **Database**: Persist job history
- **Caching**: Cache media analysis results

---

## Contributing

See the main README for contribution guidelines.

**Architecture Questions?** Open a discussion on GitHub.

---

**Version:** 2.0.0  
**Last Updated:** October 2025  
**Cross-Platform:** ✅ macOS, Windows, Linux

