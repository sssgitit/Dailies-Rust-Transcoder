import React, { useState } from 'react';
import { open } from '@tauri-apps/api/dialog';
import {
  extractMxfMetadata,
  unifyMxfMobIds,
  checkMxfMobConsistency,
  type MxfMetadata,
} from '../mxf-api';

export const MxfMobIdTool: React.FC = () => {
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [metadata, setMetadata] = useState<MxfMetadata | null>(null);
  const [consistent, setConsistent] = useState<boolean | null>(null);
  const [processing, setProcessing] = useState(false);
  const [outputFiles, setOutputFiles] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const handleSelectFiles = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'MXF Files', extensions: ['mxf', 'MXF'] }],
      });

      if (selected && Array.isArray(selected)) {
        setSelectedFiles(selected);
        setError(null);
        
        // Check consistency
        const isConsistent = await checkMxfMobConsistency(selected);
        setConsistent(isConsistent);
      }
    } catch (err) {
      setError(`Failed to select files: ${err}`);
    }
  };

  const handleExtractMetadata = async (filePath: string) => {
    try {
      setProcessing(true);
      setError(null);
      const meta = await extractMxfMetadata(filePath);
      setMetadata(meta);
    } catch (err) {
      setError(`Failed to extract metadata: ${err}`);
    } finally {
      setProcessing(false);
    }
  };

  const handleUnifyMobIds = async () => {
    if (selectedFiles.length === 0) {
      setError('Please select files first');
      return;
    }

    try {
      setProcessing(true);
      setError(null);

      const outputDir = '/tmp/mxf_unified'; // Or use file picker

      const outputs = await unifyMxfMobIds({
        inputFiles: selectedFiles,
        outputDir,
        outputType: 'avid',
      });

      setOutputFiles(outputs);
      setConsistent(true);
    } catch (err) {
      setError(`Failed to unify MOB IDs: ${err}`);
    } finally {
      setProcessing(false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold mb-6">MXF MOB ID Tool</h2>

      {/* File Selection */}
      <div className="mb-6">
        <button
          onClick={handleSelectFiles}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Select MXF Files
        </button>
        
        {selectedFiles.length > 0 && (
          <div className="mt-4">
            <p className="font-semibold mb-2">Selected Files ({selectedFiles.length}):</p>
            <ul className="space-y-1">
              {selectedFiles.map((file, idx) => (
                <li key={idx} className="flex items-center justify-between text-sm">
                  <span className="truncate flex-1">{file}</span>
                  <button
                    onClick={() => handleExtractMetadata(file)}
                    className="ml-4 px-2 py-1 text-xs bg-gray-200 rounded hover:bg-gray-300"
                  >
                    View Metadata
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* Consistency Check */}
      {consistent !== null && (
        <div className={`p-4 rounded mb-6 ${consistent ? 'bg-green-100 border-green-400' : 'bg-yellow-100 border-yellow-400'} border`}>
          {consistent ? (
            <div className="flex items-center">
              <svg className="w-5 h-5 text-green-600 mr-2" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
              </svg>
              <span className="font-semibold">All files have consistent MOB IDs</span>
            </div>
          ) : (
            <div>
              <div className="flex items-center mb-2">
                <svg className="w-5 h-5 text-yellow-600 mr-2" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                </svg>
                <span className="font-semibold">Files have different MOB IDs</span>
              </div>
              <p className="text-sm mb-3">
                These files need to be unified to work as a single clip in editing systems.
              </p>
              <button
                onClick={handleUnifyMobIds}
                disabled={processing}
                className="px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50"
              >
                {processing ? 'Processing...' : 'Unify MOB IDs'}
              </button>
            </div>
          )}
        </div>
      )}

      {/* Metadata Display */}
      {metadata && (
        <div className="bg-gray-50 p-4 rounded mb-6 border">
          <h3 className="font-bold mb-3">File Metadata</h3>
          <div className="space-y-2 text-sm font-mono">
            <div>
              <span className="font-semibold">Material Package UID:</span>
              <div className="break-all bg-white p-2 mt-1 rounded">
                {metadata.material_package_uid}
              </div>
            </div>
            {metadata.file_package_uid && (
              <div>
                <span className="font-semibold">File Package UID:</span>
                <div className="break-all bg-white p-2 mt-1 rounded">
                  {metadata.file_package_uid}
                </div>
              </div>
            )}
            {metadata.timecode && (
              <div>
                <span className="font-semibold">Timecode:</span> {metadata.timecode}
              </div>
            )}
            {metadata.duration && (
              <div>
                <span className="font-semibold">Duration:</span> {metadata.duration} frames
              </div>
            )}
            {metadata.tracks.length > 0 && (
              <div>
                <span className="font-semibold">Tracks:</span>
                <ul className="mt-1 pl-4">
                  {metadata.tracks.map((track, idx) => (
                    <li key={idx}>
                      Track {track.track_id}: {track.track_type}
                      {track.codec && ` (${track.codec})`}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Output Files */}
      {outputFiles.length > 0 && (
        <div className="bg-green-50 p-4 rounded mb-6 border border-green-400">
          <h3 className="font-bold mb-3 text-green-800">✓ Unified Files Created</h3>
          <ul className="space-y-1 text-sm">
            {outputFiles.map((file, idx) => (
              <li key={idx} className="truncate">{file}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div className="bg-red-50 p-4 rounded border border-red-400">
          <p className="text-red-800 font-semibold">Error:</p>
          <p className="text-red-700 text-sm mt-1">{error}</p>
        </div>
      )}

      {/* Info Box */}
      <div className="mt-8 p-4 bg-blue-50 rounded border border-blue-200">
        <h4 className="font-semibold text-blue-900 mb-2">About MOB IDs</h4>
        <p className="text-sm text-blue-800">
          In MXF OP-atom workflows (commonly used by Avid), each essence track (video, audio channels) 
          is stored in a separate MXF file. For editing systems to recognize these files as belonging 
          to the same clip, they must all share the same Material Package UID (MOB ID). This tool 
          helps you check and unify MOB IDs across multiple files.
        </p>
      </div>
    </div>
  );
};

