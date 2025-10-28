/**
 * A012 Car - TypeScript/Tauri Batch Processing Example
 * 
 * This example shows how to process multiple A012 clips (A012_car_01.mxf, A012_car_02.mxf, etc.)
 * using the Industrial Transcoder API
 */

import { 
  addJob, 
  startWorkers, 
  getAllJobs,
  subscribeProgress,
  getQueueStats,
  invoke 
} from '../src/api/transcoder-api';

// Configuration
const INPUT_DIR = './footage/A012_car';
const OUTPUT_DIR = './processed/A012_car';

// Sample files (replace with actual file discovery)
const A012_FILES = [
  'A012_car_01.mxf',
  'A012_car_02.mxf',
  'A012_car_03.mxf',
  'A012_car_04.mxf',
  'A012_car_05.mxf',
];

/**
 * Step 1: Rewrap all MXF files to frame-wrapped
 */
async function rewrapA012Files() {
  console.log('🔄 Step 1: Rewrapping MXF files to frame-wrapped...\n');

  const rewrapJobs = A012_FILES.map(filename => ({
    input: `${INPUT_DIR}/${filename}`,
    output: `${OUTPUT_DIR}/frame_wrapped/${filename}`,
  }));

  // Batch rewrap
  const results = await invoke('batch_rewrap_mxf', {
    request: {
      files: rewrapJobs.map(j => [j.input, j.output]),
      target_wrapping: 'FrameWrapped',
    },
  });

  console.log(`✓ Rewrapped ${results.length} files\n`);
  return rewrapJobs.map(j => j.output);
}

/**
 * Step 2: Add transcode jobs for all files
 */
async function addTranscodeJobs(frameWrappedFiles: string[]) {
  console.log('📹 Step 2: Adding transcode jobs...\n');

  const jobIds = [];

  for (const [index, inputFile] of frameWrappedFiles.entries()) {
    const filename = inputFile.split('/').pop()?.replace('.mxf', '');
    
    // ProRes HQ for editing
    const proresJob = await addJob({
      input_path: inputFile,
      output_path: `${OUTPUT_DIR}/prores/${filename}_ProResHQ.mov`,
      preset_name: 'ProRes HQ',
      priority: 'High',
    });
    
    console.log(`  [${index + 1}/${frameWrappedFiles.length}] Added ProRes HQ job: ${proresJob}`);
    jobIds.push(proresJob);
    
    // ProRes LT proxy
    const proxyJob = await addJob({
      input_path: inputFile,
      output_path: `${OUTPUT_DIR}/proxy/${filename}_ProxyLT.mov`,
      preset_name: 'ProRes LT',
      priority: 'Normal',
    });
    
    console.log(`  [${index + 1}/${frameWrappedFiles.length}] Added ProRes LT proxy job: ${proxyJob}`);
    jobIds.push(proxyJob);
  }

  console.log(`\n✓ Added ${jobIds.length} transcode jobs\n`);
  return jobIds;
}

/**
 * Step 3: Start workers and monitor progress
 */
async function processJobs() {
  console.log('⚙️  Step 3: Starting workers and processing...\n');

  // Start worker pool with 4 workers
  await startWorkers(4);
  console.log('✓ Started 4 workers\n');

  // Subscribe to progress events
  const unlisten = await subscribeProgress((event) => {
    switch (event.type) {
      case 'job_started':
        const inputName = event.input_path.split('/').pop();
        console.log(`🎬 Started: ${inputName}`);
        break;
      
      case 'job_progress':
        // Show progress every 10%
        if (event.progress % 10 < 1) {
          console.log(`   Progress: ${event.progress.toFixed(1)}%${event.fps ? ` (${event.fps.toFixed(1)} fps)` : ''}`);
        }
        break;
      
      case 'job_completed':
        console.log(`✅ Completed in ${event.duration_seconds}s\n`);
        break;
      
      case 'job_failed':
        console.error(`❌ Failed: ${event.error}\n`);
        break;
      
      case 'queue_updated':
        // Only log significant updates
        if (event.running_count > 0) {
          console.log(`📊 Queue: ${event.pending_count} pending | ${event.running_count} running | ${event.completed_count} completed`);
        }
        break;
    }
  });

  // Poll until all jobs complete
  let lastCompleted = 0;
  while (true) {
    const stats = await getQueueStats();
    
    // Check if all done
    const totalFinished = stats.completed_count + stats.failed_count;
    if (totalFinished === stats.total_count && stats.total_count > 0) {
      console.log('\n✓ All jobs complete!\n');
      break;
    }
    
    // Show periodic updates
    if (stats.completed_count > lastCompleted) {
      lastCompleted = stats.completed_count;
    }
    
    // Wait before next poll
    await new Promise(resolve => setTimeout(resolve, 2000));
  }

  unlisten();
}

/**
 * Step 4: Show final summary
 */
async function showSummary() {
  console.log('📊 Final Summary\n');
  console.log('=' .repeat(50) + '\n');

  const jobs = await getAllJobs();
  
  const completed = jobs.filter(j => j.status === 'completed');
  const failed = jobs.filter(j => j.status === 'failed');
  
  console.log(`Total jobs: ${jobs.length}`);
  console.log(`✅ Completed: ${completed.length}`);
  console.log(`❌ Failed: ${failed.length}\n`);

  // Show completed files
  if (completed.length > 0) {
    console.log('Completed files:');
    completed.forEach(job => {
      const outputName = job.output_path.split('/').pop();
      console.log(`  ✓ ${outputName}`);
    });
    console.log();
  }

  // Show failed files
  if (failed.length > 0) {
    console.log('Failed files:');
    failed.forEach(job => {
      const inputName = job.input_path.split('/').pop();
      console.log(`  ✗ ${inputName}: ${job.error_message}`);
    });
    console.log();
  }

  // Calculate total processing time
  const totalSeconds = completed.reduce((sum, job) => {
    return sum + (job.duration_seconds?.() || 0);
  }, 0);
  
  console.log(`Total processing time: ${Math.floor(totalSeconds / 60)}m ${totalSeconds % 60}s`);
  console.log();
}

/**
 * Main workflow
 */
async function main() {
  console.log('🎬 A012 Car Clips - Batch Processing Pipeline\n');
  console.log('=' .repeat(50) + '\n');

  try {
    // Step 1: Rewrap MXF files
    const frameWrappedFiles = await rewrapA012Files();
    
    // Step 2: Add transcode jobs
    await addTranscodeJobs(frameWrappedFiles);
    
    // Step 3: Process all jobs
    await processJobs();
    
    // Step 4: Show summary
    await showSummary();
    
    console.log('✨ Pipeline complete!\n');
    console.log('Next steps:');
    console.log('  1. Import ProRes HQ files into your NLE');
    console.log('  2. Use ProRes LT proxies for faster editing');
    console.log('  3. Relink to ProRes HQ for final delivery\n');
    
  } catch (error) {
    console.error('❌ Pipeline failed:', error);
    process.exit(1);
  }
}

// Run if executed directly
if (require.main === module) {
  main();
}

export { main as processA012CarClips };

