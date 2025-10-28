/**
 * Industrial Transcoder Dashboard
 * Main UI component for multi-job transcoding
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  getSystemInfo,
  getQueueStats,
  getWorkerStatus,
  startWorkers,
  stopWorkers,
  getAllJobs,
  subscribeProgress,
  clearCompletedJobs,
  SystemInfo,
  QueueStats,
  WorkerStatus,
  Job,
  ProgressEvent,
  getStatusColor,
  getPriorityColor,
  formatDuration,
} from '../api/transcoder-api';
import { AddJobDialog } from './AddJobDialog';
import { JobList } from './JobList';

export const TranscoderDashboard: React.FC = () => {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [queueStats, setQueueStats] = useState<QueueStats | null>(null);
  const [workerStatus, setWorkerStatus] = useState<WorkerStatus | null>(null);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load system info
  useEffect(() => {
    getSystemInfo()
      .then(setSystemInfo)
      .catch((err) => setError(`Failed to load system info: ${err}`));
  }, []);

  // Refresh data periodically
  const refreshData = useCallback(async () => {
    try {
      const [stats, status, jobList] = await Promise.all([
        getQueueStats(),
        getWorkerStatus(),
        getAllJobs(),
      ]);
      
      setQueueStats(stats);
      setWorkerStatus(status);
      setJobs(jobList);
    } catch (err) {
      console.error('Failed to refresh data:', err);
    }
  }, []);

  useEffect(() => {
    refreshData();
    const interval = setInterval(refreshData, 2000);
    return () => clearInterval(interval);
  }, [refreshData]);

  // Subscribe to progress events
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    subscribeProgress((event: ProgressEvent) => {
      // Refresh jobs on any progress event
      refreshData();
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((err) => {
        console.error('Failed to subscribe to progress:', err);
      });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [refreshData]);

  // Start/stop workers
  const handleToggleWorkers = async () => {
    try {
      if (workerStatus?.is_running) {
        await stopWorkers();
      } else {
        await startWorkers();
      }
      await refreshData();
    } catch (err) {
      setError(`Failed to toggle workers: ${err}`);
    }
  };

  // Clear completed jobs
  const handleClearCompleted = async () => {
    try {
      const count = await clearCompletedJobs();
      alert(`Cleared ${count} completed job(s). Logs saved to ~/.industrial-transcoder/logs/`);
      await refreshData();
    } catch (err) {
      setError(`Failed to clear completed jobs: ${err}`);
    }
  };

  if (!systemInfo) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p>Loading transcoder...</p>
        </div>
      </div>
    );
  }

  if (!systemInfo.ffmpeg_available) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center max-w-md">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <h2 className="text-2xl font-bold mb-2">FFmpeg Not Found</h2>
          <p className="text-gray-400">
            FFmpeg is required for transcoding. Please install it and restart the application.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-4xl font-bold mb-2 bg-gradient-to-r from-blue-500 to-purple-600 bg-clip-text text-transparent">
          Industrial Transcoder
        </h1>
        <p className="text-gray-400">Cross-platform multi-job media transcoding</p>
      </div>

      {/* Error Message */}
      {error && (
        <div className="mb-6 bg-red-900 border border-red-600 rounded-lg p-4">
          <div className="flex items-center justify-between">
            <span className="text-red-100">{error}</span>
            <button
              onClick={() => setError(null)}
              className="text-red-300 hover:text-red-100"
            >
              ✕
            </button>
          </div>
        </div>
      )}

      {/* System Info & Controls */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        {/* System Info */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">System Info</h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Platform:</span>
              <span className="font-mono">{systemInfo.platform}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">CPU Cores:</span>
              <span className="font-mono">{systemInfo.cpu_cores}</span>
            </div>
            {systemInfo.available_memory_mb && (
              <div className="flex justify-between">
                <span className="text-gray-400">Memory:</span>
                <span className="font-mono">{systemInfo.available_memory_mb} MB</span>
              </div>
            )}
          </div>
        </div>

        {/* Queue Stats */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">Queue Stats</h3>
          {queueStats && (
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-400">Total:</span>
                <span className="font-mono">{queueStats.total_count}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-yellow-500">Pending:</span>
                <span className="font-mono">{queueStats.pending_count}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-blue-500">Running:</span>
                <span className="font-mono">{queueStats.running_count}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-green-500">Completed:</span>
                <span className="font-mono">{queueStats.completed_count}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-red-500">Failed:</span>
                <span className="font-mono">{queueStats.failed_count}</span>
              </div>
            </div>
          )}
        </div>

        {/* Worker Status */}
        <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h3 className="text-lg font-semibold mb-4">Workers</h3>
          {workerStatus && (
            <>
              <div className="space-y-2 text-sm mb-4">
                <div className="flex justify-between">
                  <span className="text-gray-400">Status:</span>
                  <span className={workerStatus.is_running ? 'text-green-500' : 'text-gray-500'}>
                    {workerStatus.is_running ? '● Running' : '○ Stopped'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Active:</span>
                  <span className="font-mono">
                    {workerStatus.active_workers} / {workerStatus.total_workers}
                  </span>
                </div>
              </div>
              <button
                onClick={handleToggleWorkers}
                className={`w-full py-2 px-4 rounded-lg font-semibold transition-colors ${
                  workerStatus.is_running
                    ? 'bg-red-600 hover:bg-red-700'
                    : 'bg-green-600 hover:bg-green-700'
                }`}
              >
                {workerStatus.is_running ? 'Stop Workers' : 'Start Workers'}
              </button>
            </>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="mb-6 flex gap-4">
        <button
          onClick={() => setShowAddDialog(true)}
          className="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-lg font-semibold transition-colors"
        >
          + Add Job
        </button>
        <button
          onClick={refreshData}
          className="bg-gray-700 hover:bg-gray-600 px-6 py-3 rounded-lg font-semibold transition-colors"
        >
          ↻ Refresh
        </button>
        <button
          onClick={handleClearCompleted}
          disabled={!queueStats || (queueStats.completed_count + queueStats.failed_count + queueStats.cancelled_count) === 0}
          className="bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-6 py-3 rounded-lg font-semibold transition-colors"
          title="Clear completed jobs and save logs"
        >
          🗑️ Clear Completed
        </button>
      </div>

      {/* Job List */}
      <JobList jobs={jobs} onRefresh={refreshData} />

      {/* Add Job Dialog */}
      {showAddDialog && (
        <AddJobDialog
          onClose={() => setShowAddDialog(false)}
          onJobAdded={() => {
            setShowAddDialog(false);
            refreshData();
          }}
        />
      )}
    </div>
  );
};

