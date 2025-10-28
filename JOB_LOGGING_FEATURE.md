# Job Logging & Clear Completed Feature

## Overview

Added a comprehensive job logging system that creates detailed logs of all transcoding jobs before clearing them from the queue.

## Features Implemented

### 1. **Detailed Job Logging System** (`transcoder-core/src/logger.rs`)

Creates detailed logs for each transcode job including:

#### File Information
- Input/output file paths
- File sizes (in MB)

#### Codec Configuration
- Full transcode configuration (codec, bitrate, resolution, etc.)
- Preset details

#### Timing Information
- Created timestamp
- Started timestamp
- Completed timestamp
- Total duration (in seconds and minutes)

#### System Information
- Platform (macOS/Windows/Linux)
- CPU core count
- Available memory
- FFmpeg version

#### Performance Metrics
- Read speed (MB/s)
- Write speed (MB/s)
- Average FPS (when available)
- Processing speed (realtime multiplier)

#### Error Information
- Error messages for failed jobs

### 2. **Log Storage**

Logs are saved in two formats:

1. **Human-readable text**: `~/.industrial-transcoder/logs/YYYY-MM-DD_transcoder_log.txt`
2. **JSON format**: `~/.industrial-transcoder/logs/YYYY-MM-DD_transcoder_log.json`

Each day's logs are appended to a single daily log file.

### 3. **Enhanced Clear Command**

The `clear_completed_jobs` command now:
1. Retrieves all completed/failed/cancelled jobs
2. Logs each job with full details
3. Saves logs to disk
4. Clears the jobs from the queue
5. Returns the count of cleared jobs

### 4. **UI Integration**

Added a **"🗑️ Clear Completed"** button to the TranscoderDashboard that:
- Is disabled when no completed jobs exist
- Shows a confirmation with the count of cleared jobs
- Displays the log location
- Automatically refreshes the job list

## Usage

### From the UI

1. Complete some transcode jobs
2. Click the **"🗑️ Clear Completed"** button
3. View the success message showing how many jobs were cleared
4. Logs are automatically saved

### Log Location

```
~/.industrial-transcoder/logs/
├── 2025-10-28_transcoder_log.txt    # Human-readable
└── 2025-10-28_transcoder_log.json   # Machine-readable
```

### Example Log Entry

```
================================================================================
JOB LOG ENTRY
================================================================================
Job ID: 123e4567-e89b-12d3-a456-426614174000
Logged: 2025-10-28 14:30:45 UTC
Status: Completed

--------------------------------------------------------------------------------
FILE INFORMATION
--------------------------------------------------------------------------------
Input:  /path/to/source.mxf
        1024.50 MB
Output: /path/to/output.mov
        850.25 MB

--------------------------------------------------------------------------------
CODEC CONFIGURATION
--------------------------------------------------------------------------------
{
  "video_codec": "prores",
  "audio_codec": "pcm_s24le",
  "container": "mov",
  ...
}

--------------------------------------------------------------------------------
TIMING INFORMATION
--------------------------------------------------------------------------------
Created:   2025-10-28 14:25:00 UTC
Started:   2025-10-28 14:25:05 UTC
Completed: 2025-10-28 14:30:45 UTC
Duration:  340 seconds (5.67 minutes)

--------------------------------------------------------------------------------
SYSTEM INFORMATION
--------------------------------------------------------------------------------
Platform: macOS
CPU Cores: 8
Available Memory: 16384 MB (16.00 GB)
FFmpeg: ffmpeg version 6.0

--------------------------------------------------------------------------------
PERFORMANCE METRICS
--------------------------------------------------------------------------------
Read Speed:  3.01 MB/s
Write Speed: 2.50 MB/s

================================================================================
```

## API Reference

### Rust Backend

```rust
// Log a single job
let logger = JobLogger::default();
logger.log_job(&job)?;

// Log multiple jobs
logger.log_jobs(&jobs)?;
```

### TypeScript Frontend

```typescript
import { clearCompletedJobs } from './api/transcoder-api';

// Clear completed jobs (automatically logs them)
const count = await clearCompletedJobs();
console.log(`Cleared ${count} jobs`);
```

## Benefits

1. **Historical Record**: Keep a permanent record of all transcode operations
2. **Performance Analysis**: Compare read/write speeds across different systems
3. **Debugging**: Review error messages and configurations for failed jobs
4. **Auditing**: Track what was transcoded, when, and on what hardware
5. **Optimization**: Identify bottlenecks and performance patterns

## Git Commit

Changes committed and pushed to GitHub:
- Commit: `cd9c38f`
- Message: "Add job logging system with clear completed jobs feature"

## Next Steps (Optional Enhancements)

1. Add a "View Logs" button in the UI
2. Include video duration and media metadata in logs
3. Export logs to CSV for spreadsheet analysis
4. Add log rotation (keep last N days)
5. Include GPU usage metrics (when available)
6. Add network transfer speeds for remote files

