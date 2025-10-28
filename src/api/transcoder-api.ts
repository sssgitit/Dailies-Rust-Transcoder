/**
 * TypeScript API for Industrial Transcoder
 * Cross-platform multi-job transcoding interface
 */

import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

// ============================================================================
// Types
// ============================================================================

export interface SystemInfo {
  platform: string;
  cpu_cores: number;
  available_memory_mb: number | null;
  ffmpeg_available: boolean;
  ffmpeg_path: string | null;
}

export interface CodecPreset {
  name: string;
  description: string;
  config: TranscodeConfig;
}

export interface TranscodeConfig {
  video_codec: VideoCodec;
  audio_codec: AudioCodec;
  container: ContainerFormat;
  video_bitrate?: string;
  audio_bitrate?: string;
  audio_sample_rate?: number;
  resolution?: string;
  frame_rate?: number;
  prores_profile?: ProResProfile;
  extra_args: string[];
}

export type VideoCodec = 'prores' | 'prores_ks' | 'dnxhd' | 'h264' | 'h265' | 'copy';
export type AudioCodec = 'pcm_s16le' | 'pcm_s24le' | 'aac' | 'copy';
export type ContainerFormat = 'mov' | 'mp4' | 'mxf' | 'wav' | 'auto';
export type ProResProfile = 'proxy' | 'lt' | 'standard' | 'hq' | '4444' | '4444xq';

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
export type Priority = 'Low' | 'Normal' | 'High' | 'Urgent';

export interface Job {
  id: string;
  input_path: string;
  output_path: string;
  status: JobStatus;
  priority: Priority;
  progress: number;
  error_message: string | null;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  config: any;
}

export interface AddJobRequest {
  input_path: string;
  output_path: string;
  preset_name: string;
  priority?: Priority;
}

export interface QueueStats {
  total_count: number;
  pending_count: number;
  running_count: number;
  completed_count: number;
  failed_count: number;
  cancelled_count: number;
}

export interface WorkerStatus {
  is_running: boolean;
  active_workers: number;
  total_workers: number;
}

export type ProgressEvent =
  | { type: 'job_started'; job_id: string; input_path: string; output_path: string }
  | { type: 'job_progress'; job_id: string; progress: number; fps?: number; eta_seconds?: number }
  | { type: 'job_completed'; job_id: string; duration_seconds: number }
  | { type: 'job_failed'; job_id: string; error: string }
  | { type: 'job_cancelled'; job_id: string }
  | { type: 'queue_updated'; pending_count: number; running_count: number; completed_count: number };

// ============================================================================
// System Commands
// ============================================================================

export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke('get_system_info');
}

export async function verifyFfmpeg(): Promise<string> {
  return await invoke('verify_ffmpeg');
}

// ============================================================================
// Preset Commands
// ============================================================================

export async function getPresets(): Promise<Record<string, CodecPreset>> {
  return await invoke('get_presets');
}

// ============================================================================
// Job Commands
// ============================================================================

export async function addJob(request: AddJobRequest): Promise<string> {
  return await invoke('add_job', { request });
}

export async function getJob(jobId: string): Promise<Job> {
  return await invoke('get_job', { jobId });
}

export async function getAllJobs(): Promise<Job[]> {
  return await invoke('get_all_jobs');
}

export async function cancelJob(jobId: string): Promise<void> {
  return await invoke('cancel_job', { jobId });
}

export async function clearCompletedJobs(): Promise<number> {
  return await invoke('clear_completed_jobs');
}

export async function getQueueStats(): Promise<QueueStats> {
  return await invoke('get_queue_stats');
}

// ============================================================================
// Worker Pool Commands
// ============================================================================

export async function startWorkers(workerCount?: number): Promise<void> {
  return await invoke('start_workers', { workerCount: workerCount ?? null });
}

export async function stopWorkers(): Promise<void> {
  return await invoke('stop_workers');
}

export async function getWorkerStatus(): Promise<WorkerStatus> {
  return await invoke('get_worker_status');
}

// ============================================================================
// Progress Subscription
// ============================================================================

export async function subscribeProgress(
  callback: (event: ProgressEvent) => void
): Promise<UnlistenFn> {
  // Initialize subscription
  await invoke('subscribe_progress');
  
  // Listen to progress events
  return await listen<ProgressEvent>('transcode_progress', (event) => {
    callback(event.payload);
  });
}

// ============================================================================
// Utility Functions
// ============================================================================

export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  
  if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  
  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

export function getStatusColor(status: JobStatus): string {
  switch (status) {
    case 'pending':
      return 'text-yellow-500';
    case 'running':
      return 'text-blue-500';
    case 'completed':
      return 'text-green-500';
    case 'failed':
      return 'text-red-500';
    case 'cancelled':
      return 'text-gray-500';
    default:
      return 'text-gray-400';
  }
}

export function getPriorityColor(priority: Priority): string {
  switch (priority) {
    case 'Urgent':
      return 'text-red-600';
    case 'High':
      return 'text-orange-500';
    case 'Normal':
      return 'text-blue-500';
    case 'Low':
      return 'text-gray-500';
    default:
      return 'text-gray-400';
  }
}

