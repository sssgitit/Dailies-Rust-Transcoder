import React, { useState } from 'react';
import { open } from '@tauri-apps/api/dialog';
import {
  BwfTimecode,
  calculateBextTimecode,
  createBwfFile,
  extractTimecodeFromFile,
  formatTimecode,
  parseTimecode,
  validateTimecode,
  createBwfFromMxf,
} from '../bwf-api';

export const BwfTimecodeCreator: React.FC = () => {
  const [inputFile, setInputFile] = useState<string>('');
  const [outputFile, setOutputFile] = useState<string>('');
  const [timecode, setTimecode] = useState<BwfTimecode>({
    hours: 0,
    minutes: 0,
    seconds: 0,
    frames: 0,
  });
  const [sampleRate, setSampleRate] = useState(48000);
  const [timeReference, setTimeReference] = useState<number | null>(null);
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleSelectInputFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'Media Files', extensions: ['mxf', 'wav', 'mp4', 'mov'] },
        ],
      });

      if (selected && typeof selected === 'string') {
        setInputFile(selected);
        setError(null);
        
        // Try to auto-extract timecode if it's an MXF
        if (selected.toLowerCase().endsWith('.mxf')) {
          try {
            const extractedTc = await extractTimecodeFromFile(selected);
            setTimecode(extractedTc);
            setSuccess(`Auto-extracted timecode: ${formatTimecode(extractedTc)}`);
          } catch (err) {
            console.log('Could not auto-extract timecode:', err);
          }
        }
      }
    } catch (err) {
      setError(`Failed to select file: ${err}`);
    }
  };

  const handleSelectOutputFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });

      if (selected && typeof selected === 'string') {
        const baseName = inputFile.split('/').pop()?.replace(/\.[^/.]+$/, '') || 'output';
        setOutputFile(`${selected}/${baseName}_bwf.wav`);
        setError(null);
      }
    } catch (err) {
      setError(`Failed to select output: ${err}`);
    }
  };

  const handleCalculateTimeReference = async () => {
    const validationError = validateTimecode(timecode);
    if (validationError) {
      setError(validationError);
      return;
    }

    try {
      const ref = await calculateBextTimecode(timecode);
      setTimeReference(ref);
      setError(null);
    } catch (err) {
      setError(`Failed to calculate: ${err}`);
    }
  };

  const handleCreateBwf = async () => {
    if (!inputFile || !outputFile) {
      setError('Please select input and output files');
      return;
    }

    const validationError = validateTimecode(timecode);
    if (validationError) {
      setError(validationError);
      return;
    }

    setProcessing(true);
    setError(null);
    setSuccess(null);

    try {
      await createBwfFile({
        inputFile,
        outputFile,
        timecode,
        sampleRate,
        description: `BWF with timecode ${formatTimecode(timecode)}`,
      });

      setSuccess(`✅ BWF file created successfully at:\n${outputFile}`);
    } catch (err) {
      setError(`Failed to create BWF: ${err}`);
    } finally {
      setProcessing(false);
    }
  };

  const handleQuickConvert = async () => {
    if (!inputFile) {
      setError('Please select an input file');
      return;
    }

    if (!inputFile.toLowerCase().endsWith('.mxf')) {
      setError('Quick convert only works with MXF files');
      return;
    }

    setProcessing(true);
    setError(null);
    setSuccess(null);

    try {
      const outputPath = inputFile.replace(/\.mxf$/i, '_bwf.wav');
      await createBwfFromMxf(inputFile, outputPath, sampleRate);
      setOutputFile(outputPath);
      setSuccess(`✅ BWF file created successfully at:\n${outputPath}`);
    } catch (err) {
      setError(`Quick convert failed: ${err}`);
    } finally {
      setProcessing(false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold mb-6">BWF BEXT Timecode Creator</h2>

      {/* File Selection */}
      <div className="mb-6 space-y-4">
        <div>
          <label className="block text-sm font-medium mb-2">Input File</label>
          <div className="flex gap-2">
            <button
              onClick={handleSelectInputFile}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Select Input
            </button>
            {inputFile && (
              <span className="flex-1 px-3 py-2 bg-gray-100 rounded truncate text-sm">
                {inputFile}
              </span>
            )}
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Output File</label>
          <div className="flex gap-2">
            <button
              onClick={handleSelectOutputFile}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Select Output
            </button>
            {outputFile && (
              <span className="flex-1 px-3 py-2 bg-gray-100 rounded truncate text-sm">
                {outputFile}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Timecode Input */}
      <div className="mb-6">
        <label className="block text-sm font-medium mb-2">
          Timecode (23.976fps)
        </label>
        <div className="grid grid-cols-4 gap-4">
          <div>
            <input
              type="number"
              placeholder="HH"
              min="0"
              max="23"
              value={timecode.hours}
              onChange={(e) =>
                setTimecode({ ...timecode, hours: parseInt(e.target.value) || 0 })
              }
              className="w-full px-3 py-2 border rounded"
            />
            <span className="text-xs text-gray-500">Hours</span>
          </div>
          <div>
            <input
              type="number"
              placeholder="MM"
              min="0"
              max="59"
              value={timecode.minutes}
              onChange={(e) =>
                setTimecode({ ...timecode, minutes: parseInt(e.target.value) || 0 })
              }
              className="w-full px-3 py-2 border rounded"
            />
            <span className="text-xs text-gray-500">Minutes</span>
          </div>
          <div>
            <input
              type="number"
              placeholder="SS"
              min="0"
              max="59"
              value={timecode.seconds}
              onChange={(e) =>
                setTimecode({ ...timecode, seconds: parseInt(e.target.value) || 0 })
              }
              className="w-full px-3 py-2 border rounded"
            />
            <span className="text-xs text-gray-500">Seconds</span>
          </div>
          <div>
            <input
              type="number"
              placeholder="FF"
              min="0"
              max="23"
              value={timecode.frames}
              onChange={(e) =>
                setTimecode({ ...timecode, frames: parseInt(e.target.value) || 0 })
              }
              className="w-full px-3 py-2 border rounded"
            />
            <span className="text-xs text-gray-500">Frames</span>
          </div>
        </div>
        <div className="mt-2 text-sm text-gray-600">
          Current: {formatTimecode(timecode)}
        </div>
      </div>

      {/* Sample Rate */}
      <div className="mb-6">
        <label className="block text-sm font-medium mb-2">Sample Rate</label>
        <select
          value={sampleRate}
          onChange={(e) => setSampleRate(parseInt(e.target.value))}
          className="w-full px-3 py-2 border rounded"
        >
          <option value="48000">48000 Hz (Standard)</option>
          <option value="48048">48048 Hz (0.1% Pull-up)</option>
        </select>
      </div>

      {/* Actions */}
      <div className="mb-6 flex gap-4">
        <button
          onClick={handleCalculateTimeReference}
          className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
        >
          Calculate TimeReference
        </button>

        <button
          onClick={handleCreateBwf}
          disabled={processing || !inputFile || !outputFile}
          className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {processing ? 'Creating...' : 'Create BWF File'}
        </button>

        {inputFile.toLowerCase().endsWith('.mxf') && (
          <button
            onClick={handleQuickConvert}
            disabled={processing}
            className="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50"
          >
            Quick Convert (MXF → BWF)
          </button>
        )}
      </div>

      {/* TimeReference Display */}
      {timeReference !== null && (
        <div className="mb-6 p-4 bg-blue-50 rounded border border-blue-200">
          <h3 className="font-semibold mb-2">Calculated TimeReference</h3>
          <div className="font-mono text-lg">{timeReference.toLocaleString()} samples</div>
          <div className="text-sm text-gray-600 mt-1">
            For timecode: {formatTimecode(timecode)} @ {sampleRate} Hz
          </div>
        </div>
      )}

      {/* Success Message */}
      {success && (
        <div className="mb-6 p-4 bg-green-50 rounded border border-green-400">
          <p className="text-green-800 whitespace-pre-wrap">{success}</p>
        </div>
      )}

      {/* Error Message */}
      {error && (
        <div className="mb-6 p-4 bg-red-50 rounded border border-red-400">
          <p className="text-red-800">{error}</p>
        </div>
      )}

      {/* Info */}
      <div className="mt-8 p-4 bg-gray-50 rounded border">
        <h4 className="font-semibold mb-2">About BWF BEXT Timecode</h4>
        <p className="text-sm text-gray-700 mb-2">
          This tool creates Broadcast Wave Format (BWF) files with BEXT chunks containing
          frame-accurate timecodes for 23.976fps content.
        </p>
        <p className="text-sm text-gray-700">
          <strong>Method:</strong> Frame-based calculation (validated on production footage)<br/>
          <strong>Formula:</strong> TimeReference = total_frames × 2004.005263<br/>
          <strong>Output:</strong> 48000 Hz BWF files<br/>
          <strong>Accuracy:</strong> Frame-perfect for 23.976fps workflows
        </p>
      </div>
    </div>
  );
};

