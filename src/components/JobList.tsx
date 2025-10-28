/**
 * Job List Component
 * Displays all transcode jobs with progress and controls
 */

import React from 'react';
import { Job, cancelJob, getStatusColor, getPriorityColor } from '../api/transcoder-api';

interface JobListProps {
  jobs: Job[];
  onRefresh: () => void;
}

export const JobList: React.FC<JobListProps> = ({ jobs, onRefresh }) => {
  const handleCancelJob = async (jobId: string) => {
    try {
      await cancelJob(jobId);
      onRefresh();
    } catch (err) {
      console.error('Failed to cancel job:', err);
    }
  };

  if (jobs.length === 0) {
    return (
      <div className="bg-gray-800 rounded-lg p-12 border border-gray-700 text-center">
        <div className="text-gray-500 text-6xl mb-4">📁</div>
        <h3 className="text-xl font-semibold text-gray-300 mb-2">No Jobs Yet</h3>
        <p className="text-gray-500">Add a job to get started with transcoding</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold mb-4">Jobs</h2>
      {jobs.map((job) => (
        <div
          key={job.id}
          className="bg-gray-800 rounded-lg p-6 border border-gray-700 hover:border-gray-600 transition-colors"
        >
          <div className="flex items-start justify-between mb-4">
            <div className="flex-1">
              <div className="flex items-center gap-3 mb-2">
                <span className={`text-lg font-semibold ${getStatusColor(job.status)}`}>
                  {job.status.toUpperCase()}
                </span>
                <span className={`text-sm ${getPriorityColor(job.priority)}`}>
                  {job.priority}
                </span>
              </div>
              <div className="text-sm text-gray-400 space-y-1">
                <div className="flex items-center gap-2">
                  <span className="text-gray-500">Input:</span>
                  <span className="font-mono text-xs bg-gray-900 px-2 py-1 rounded">
                    {job.input_path}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-gray-500">Output:</span>
                  <span className="font-mono text-xs bg-gray-900 px-2 py-1 rounded">
                    {job.output_path}
                  </span>
                </div>
              </div>
            </div>

            {/* Actions */}
            {job.status === 'pending' || job.status === 'running' ? (
              <button
                onClick={() => handleCancelJob(job.id)}
                className="bg-red-600 hover:bg-red-700 px-4 py-2 rounded text-sm font-semibold transition-colors"
              >
                Cancel
              </button>
            ) : null}
          </div>

          {/* Progress Bar */}
          {job.status === 'running' && (
            <div className="mt-4">
              <div className="flex items-center justify-between text-sm mb-2">
                <span className="text-gray-400">Progress</span>
                <span className="text-blue-400 font-semibold">{job.progress.toFixed(1)}%</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-gradient-to-r from-blue-500 to-purple-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${job.progress}%` }}
                />
              </div>
            </div>
          )}

          {/* Error Message */}
          {job.error_message && (
            <div className="mt-4 bg-red-900 border border-red-600 rounded p-3 text-sm text-red-100">
              <span className="font-semibold">Error:</span> {job.error_message}
            </div>
          )}

          {/* Timestamps */}
          <div className="mt-4 flex gap-6 text-xs text-gray-500">
            <div>
              <span>Created:</span>{' '}
              <span className="font-mono">{new Date(job.created_at).toLocaleString()}</span>
            </div>
            {job.started_at && (
              <div>
                <span>Started:</span>{' '}
                <span className="font-mono">{new Date(job.started_at).toLocaleString()}</span>
              </div>
            )}
            {job.completed_at && (
              <div>
                <span>Completed:</span>{' '}
                <span className="font-mono">{new Date(job.completed_at).toLocaleString()}</span>
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};

