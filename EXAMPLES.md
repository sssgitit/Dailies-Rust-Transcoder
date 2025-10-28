# Usage Examples

**Practical examples for Industrial Transcoder**

## Table of Contents

- [CLI Examples](#cli-examples)
- [API Examples](#api-examples)
- [Preset Examples](#preset-examples)
- [Batch Processing](#batch-processing)
- [Advanced Workflows](#advanced-workflows)

---

## CLI Examples

### Basic Transcode

```bash
# Transcode MXF to ProRes HQ
transcoder transcode \
  --input footage/A001_C001.mxf \
  --output output/A001_C001.mov \
  --preset prores_hq

# Transcode with custom worker count
transcoder transcode \
  --input input.mxf \
  --output output.mov \
  --preset prores_422 \
  --workers 4
```

### List Available Presets

```bash
transcoder presets
```

**Output:**
```
=== Available Codec Presets ===

H.264 High
  H.264 high quality for delivery
  Video: H264
  Audio: AAC
  Container: MP4

ProRes 422
  Standard ProRes 422
  Video: ProResKS
  Audio: PCM24
  Container: MOV

ProRes HQ
  High Quality ProRes for broadcast
  Video: ProResKS
  Audio: PCM24
  Container: MOV

ProRes LT
  ProRes LT for offline editing
  Video: ProResKS
  Audio: PCM24
  Container: MOV
```

### System Information

```bash
transcoder info
```

**Output:**
```
=== System Information ===

Platform: macOS
CPU Cores: 8
Available Memory: 16384 MB

FFmpeg: /opt/homebrew/bin/ffmpeg
```

### Verify FFmpeg

```bash
transcoder verify
```

---

## API Examples

### TypeScript/React Integration

#### Basic Job Submission

```typescript
import { addJob, startWorkers, subscribeProgress } from './api/transcoder-api';

// Start worker pool
await startWorkers(4);

// Add a job
const jobId = await addJob({
  input_path: '/path/to/input.mxf',
  output_path: '/path/to/output.mov',
  preset_name: 'ProRes HQ',
  priority: 'High'
});

console.log('Job added:', jobId);
```

#### Progress Monitoring

```typescript
import { subscribeProgress } from './api/transcoder-api';

// Subscribe to progress events
const unlisten = await subscribeProgress((event) => {
  switch (event.type) {
    case 'job_started':
      console.log(`Job ${event.job_id} started`);
      console.log(`Input: ${event.input_path}`);
      break;
    
    case 'job_progress':
      console.log(`Job ${event.job_id}: ${event.progress.toFixed(1)}%`);
      if (event.fps) {
        console.log(`FPS: ${event.fps}`);
      }
      break;
    
    case 'job_completed':
      console.log(`Job ${event.job_id} completed in ${event.duration_seconds}s`);
      break;
    
    case 'job_failed':
      console.error(`Job ${event.job_id} failed: ${event.error}`);
      break;
    
    case 'queue_updated':
      console.log('Queue stats:');
      console.log(`  Pending: ${event.pending_count}`);
      console.log(`  Running: ${event.running_count}`);
      console.log(`  Completed: ${event.completed_count}`);
      break;
  }
});

// Later: unsubscribe
unlisten();
```

#### React Component Example

```typescript
import React, { useState } from 'react';
import { addJob, startWorkers } from './api/transcoder-api';

export const QuickTranscode: React.FC = () => {
  const [inputPath, setInputPath] = useState('');
  const [outputPath, setOutputPath] = useState('');
  const [status, setStatus] = useState('');

  const handleTranscode = async () => {
    try {
      setStatus('Starting workers...');
      await startWorkers();
      
      setStatus('Adding job...');
      const jobId = await addJob({
        input_path: inputPath,
        output_path: outputPath,
        preset_name: 'ProRes HQ',
        priority: 'Normal',
      });
      
      setStatus(`Job added: ${jobId}`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  };

  return (
    <div>
      <input
        value={inputPath}
        onChange={(e) => setInputPath(e.target.value)}
        placeholder="Input path"
      />
      <input
        value={outputPath}
        onChange={(e) => setOutputPath(e.target.value)}
        placeholder="Output path"
      />
      <button onClick={handleTranscode}>Transcode</button>
      <p>{status}</p>
    </div>
  );
};
```

---

## Preset Examples

### Using Built-in Presets

```typescript
import { getPresets, addJob } from './api/transcoder-api';

// Get all presets
const presets = await getPresets();

// List preset names
Object.keys(presets).forEach(name => {
  console.log(name, ':', presets[name].description);
});

// Use a preset
await addJob({
  input_path: '/input.mxf',
  output_path: '/output.mov',
  preset_name: 'ProRes HQ',
  priority: 'Normal'
});
```

### Custom Preset Configuration

If you need to customize beyond presets, you can build custom configs:

```rust
// In Rust
use transcoder_core::config::*;

let custom_config = TranscodeConfig {
    video_codec: VideoCodec::ProResKS,
    audio_codec: AudioCodec::PCM24,
    container: ContainerFormat::MOV,
    prores_profile: Some(ProResProfile::HQ),
    audio_sample_rate: Some(48000),
    resolution: Some("1920x1080".to_string()),
    frame_rate: Some(23.976),
    ..Default::default()
};
```

---

## Batch Processing

### Process Multiple Files (Shell Script)

```bash
#!/bin/bash
# batch_transcode.sh

INPUT_DIR="./footage"
OUTPUT_DIR="./transcoded"
PRESET="prores_hq"

mkdir -p "$OUTPUT_DIR"

for file in "$INPUT_DIR"/*.mxf; do
  basename=$(basename "$file" .mxf)
  output="$OUTPUT_DIR/${basename}.mov"
  
  echo "Processing: $file → $output"
  
  transcoder transcode \
    --input "$file" \
    --output "$output" \
    --preset "$PRESET"
done

echo "Batch processing complete!"
```

### Process Multiple Files (JavaScript)

```typescript
import { addJob, startWorkers, getQueueStats } from './api/transcoder-api';
import { readdir } from 'fs/promises';
import { join } from 'path';

async function batchTranscode(
  inputDir: string,
  outputDir: string,
  preset: string
) {
  // Start workers
  await startWorkers();
  
  // Get all MXF files
  const files = await readdir(inputDir);
  const mxfFiles = files.filter(f => f.endsWith('.mxf'));
  
  // Add all jobs
  const jobIds = [];
  for (const file of mxfFiles) {
    const inputPath = join(inputDir, file);
    const outputPath = join(outputDir, file.replace('.mxf', '.mov'));
    
    const jobId = await addJob({
      input_path: inputPath,
      output_path: outputPath,
      preset_name: preset,
      priority: 'Normal'
    });
    
    jobIds.push(jobId);
    console.log(`Added job ${jobId} for ${file}`);
  }
  
  console.log(`Added ${jobIds.length} jobs to queue`);
  
  // Poll until all complete
  while (true) {
    const stats = await getQueueStats();
    console.log(`Progress: ${stats.completed_count}/${stats.total_count}`);
    
    if (stats.completed_count + stats.failed_count === stats.total_count) {
      break;
    }
    
    await new Promise(resolve => setTimeout(resolve, 5000));
  }
  
  console.log('Batch complete!');
}

// Usage
batchTranscode('./footage', './transcoded', 'ProRes HQ');
```

---

## Advanced Workflows

### Priority Queue Management

```typescript
import { addJob } from './api/transcoder-api';

// High priority job (processed first)
await addJob({
  input_path: '/urgent/clip.mxf',
  output_path: '/urgent/clip.mov',
  preset_name: 'ProRes HQ',
  priority: 'Urgent'  // Urgent > High > Normal > Low
});

// Background jobs (lower priority)
await addJob({
  input_path: '/archive/old_clip.mxf',
  output_path: '/archive/old_clip.mov',
  preset_name: 'ProRes LT',
  priority: 'Low'
});
```

### Worker Pool Management

```typescript
import { startWorkers, stopWorkers, getWorkerStatus } from './api/transcoder-api';

// Start with specific worker count
await startWorkers(2);  // Use 2 workers

// Check status
const status = await getWorkerStatus();
console.log(`Workers: ${status.active_workers}/${status.total_workers}`);

// Stop workers (finish current jobs first)
await stopWorkers();
```

### MXF to BWF Workflow

Integration with existing BWF tools:

```typescript
import { addJob } from './api/transcoder-api';

// 1. Extract audio from MXF to WAV
await addJob({
  input_path: '/input/video.mxf',
  output_path: '/temp/audio.wav',
  preset_name: 'WAV 48kHz',  // Audio-only preset
  priority: 'High'
});

// 2. Then use BWF tools to add BEXT timecode
// (See bwf-tools/ for timecode insertion)
```

### Watch Folder Pattern

```typescript
import { watch } from 'fs/promises';
import { addJob, startWorkers } from './api/transcoder-api';

async function watchFolder(inputDir: string, outputDir: string) {
  // Start workers once
  await startWorkers();
  
  // Watch for new files
  const watcher = watch(inputDir);
  
  for await (const event of watcher) {
    if (event.filename?.endsWith('.mxf')) {
      const inputPath = join(inputDir, event.filename);
      const outputPath = join(
        outputDir,
        event.filename.replace('.mxf', '.mov')
      );
      
      console.log(`New file detected: ${event.filename}`);
      
      // Wait a bit to ensure file is fully written
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Add transcode job
      await addJob({
        input_path: inputPath,
        output_path: outputPath,
        preset_name: 'ProRes HQ',
        priority: 'Normal'
      });
    }
  }
}

// Usage
watchFolder('./input', './output');
```

---

## Real-World Scenarios

### Dailies Workflow

```typescript
// Transcode camera originals to ProRes for editing
async function dailiesWorkflow(
  cameraDir: string,
  dailiesDir: string
) {
  await startWorkers(4);  // Use 4 workers
  
  const files = await readdir(cameraDir);
  
  for (const file of files) {
    if (file.endsWith('.mxf')) {
      await addJob({
        input_path: join(cameraDir, file),
        output_path: join(dailiesDir, file.replace('.mxf', '.mov')),
        preset_name: 'ProRes HQ',
        priority: 'High'
      });
    }
  }
}
```

### Archive Workflow

```typescript
// Convert to H.264 for long-term storage
async function archiveWorkflow(
  sourceDir: string,
  archiveDir: string
) {
  await startWorkers(2);  // Lower priority, fewer workers
  
  const files = await readdir(sourceDir);
  
  for (const file of files) {
    if (file.endsWith('.mov')) {
      await addJob({
        input_path: join(sourceDir, file),
        output_path: join(archiveDir, file.replace('.mov', '.mp4')),
        preset_name: 'H.264 High',
        priority: 'Low'  // Background priority
      });
    }
  }
}
```

### Delivery Workflow

```typescript
// Create multiple deliverables from master
async function deliveryWorkflow(masterFile: string) {
  await startWorkers();
  
  const deliverables = [
    { preset: 'ProRes HQ', suffix: '_master' },
    { preset: 'H.264 High', suffix: '_web' },
    { preset: 'ProRes LT', suffix: '_proxy' },
  ];
  
  for (const { preset, suffix } of deliverables) {
    const outputPath = masterFile
      .replace('.mov', `${suffix}.mov`)
      .replace('.mxf', `${suffix}.mov`);
    
    await addJob({
      input_path: masterFile,
      output_path: outputPath,
      preset_name: preset,
      priority: 'Normal'
    });
  }
}
```

---

## Error Handling

```typescript
import { addJob } from './api/transcoder-api';

async function safeTranscode(inputPath: string, outputPath: string) {
  try {
    const jobId = await addJob({
      input_path: inputPath,
      output_path: outputPath,
      preset_name: 'ProRes HQ',
      priority: 'Normal'
    });
    
    console.log(`Job ${jobId} added successfully`);
    return jobId;
    
  } catch (err) {
    if (err.toString().includes('not found')) {
      console.error('Input file not found:', inputPath);
    } else if (err.toString().includes('FFmpeg')) {
      console.error('FFmpeg error - check installation');
    } else {
      console.error('Transcode error:', err);
    }
    
    return null;
  }
}
```

---

## Next Steps

- **Explore Presets**: See [CODEC_PRESETS.md](CODEC_PRESETS.md) (if available)
- **Read API Docs**: Full API reference
- **Check Workflows**: Integration with MXF and BWF tools

---

**Version:** 2.0.0  
**More Examples?** Open a discussion on GitHub!

