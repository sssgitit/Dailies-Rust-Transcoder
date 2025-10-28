/**
 * MXF Rewrap Tool
 * Convert between clip-wrapped and frame-wrapped MXF files
 */

import React, { useState, useEffect } from 'react';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

type MxfWrapping = 'ClipWrapped' | 'FrameWrapped';

export const MxfRewrapTool: React.FC = () => {
  const [inputPath, setInputPath] = useState('');
  const [outputPath, setOutputPath] = useState('');
  const [currentWrapping, setCurrentWrapping] = useState<MxfWrapping | null>(null);
  const [targetWrapping, setTargetWrapping] = useState<MxfWrapping>('FrameWrapped');
  const [isAvailable, setIsAvailable] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    // Check if MXF rewrapping is available
    invoke<boolean>('is_mxf_rewrapping_available')
      .then(setIsAvailable)
      .catch(() => setIsAvailable(false));
  }, []);

  const handleSelectInput = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'MXF Files',
            extensions: ['mxf'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setInputPath(selected);
        setError(null);
        setSuccess(null);
        
        // Auto-detect wrapping
        setLoading(true);
        try {
          const wrapping = await invoke<MxfWrapping>('detect_mxf_wrapping', {
            inputPath: selected,
          });
          setCurrentWrapping(wrapping);
          
          // Set opposite as target
          setTargetWrapping(
            wrapping === 'ClipWrapped' ? 'FrameWrapped' : 'ClipWrapped'
          );
        } catch (err) {
          setError(`Could not detect wrapping: ${err}`);
        } finally {
          setLoading(false);
        }
        
        // Auto-generate output path
        if (!outputPath) {
          const newPath = selected.replace('.mxf', '_rewrapped.mxf');
          setOutputPath(newPath);
        }
      }
    } catch (err) {
      setError(`Failed to select file: ${err}`);
    }
  };

  const handleSelectOutput = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'MXF Files',
            extensions: ['mxf'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setOutputPath(selected);
      }
    } catch (err) {
      setError(`Failed to select output: ${err}`);
    }
  };

  const handleRewrap = async () => {
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      await invoke('rewrap_mxf', {
        request: {
          input_path: inputPath,
          output_path: outputPath,
          target_wrapping: targetWrapping,
        },
      });

      setSuccess(`✓ Successfully rewrapped to ${formatWrapping(targetWrapping)}`);
    } catch (err) {
      setError(`Rewrap failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const formatWrapping = (wrapping: MxfWrapping): string => {
    return wrapping === 'ClipWrapped' ? 'Clip-Wrapped' : 'Frame-Wrapped';
  };

  if (!isAvailable) {
    return (
      <div className="bg-gray-800 rounded-lg p-8 border border-gray-700">
        <div className="text-center">
          <div className="text-yellow-500 text-6xl mb-4">⚠️</div>
          <h3 className="text-xl font-bold mb-2">BMX Tools Not Found</h3>
          <p className="text-gray-400 mb-4">
            MXF rewrapping requires bmxtranswrap from BMX tools
          </p>
          <div className="bg-gray-900 rounded p-4 text-left text-sm font-mono">
            <div className="text-gray-500 mb-2"># Install on macOS:</div>
            <div className="text-blue-400">brew install bmx</div>
            <div className="text-gray-500 mt-3 mb-2"># Or build from source:</div>
            <div className="text-blue-400">git clone https://github.com/ebu/bmx.git</div>
            <div className="text-blue-400">cd bmx && mkdir build && cd build</div>
            <div className="text-blue-400">cmake .. && make && sudo make install</div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-8 border border-gray-700">
      <div className="mb-6">
        <h2 className="text-2xl font-bold mb-2">MXF Rewrap Tool</h2>
        <p className="text-gray-400">
          Convert between clip-wrapped and frame-wrapped MXF files
        </p>
      </div>

      {/* Info Box */}
      <div className="mb-6 bg-blue-900 border border-blue-600 rounded-lg p-4 text-sm">
        <div className="font-semibold text-blue-200 mb-2">📘 What's the difference?</div>
        <div className="text-blue-100 space-y-1">
          <div>
            <strong>Clip-Wrapped:</strong> Essence stored in contiguous chunks. More efficient
            for playback but harder to edit.
          </div>
          <div>
            <strong>Frame-Wrapped:</strong> Essence interleaved frame-by-frame. Better for
            editing systems (Avid, Premiere, etc.)
          </div>
        </div>
      </div>

      {error && (
        <div className="mb-6 bg-red-900 border border-red-600 rounded-lg p-4 text-red-100">
          {error}
        </div>
      )}

      {success && (
        <div className="mb-6 bg-green-900 border border-green-600 rounded-lg p-4 text-green-100">
          {success}
        </div>
      )}

      <div className="space-y-6">
        {/* Input File */}
        <div>
          <label className="block text-sm font-semibold mb-2">Input MXF File</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={inputPath}
              onChange={(e) => setInputPath(e.target.value)}
              placeholder="/path/to/input.mxf"
              className="flex-1 bg-gray-900 border border-gray-600 rounded px-4 py-2 text-white focus:outline-none focus:border-blue-500"
            />
            <button
              onClick={handleSelectInput}
              disabled={loading}
              className="bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 px-4 py-2 rounded font-semibold transition-colors"
            >
              Browse
            </button>
          </div>
          {currentWrapping && (
            <div className="mt-2 text-sm">
              <span className="text-gray-400">Current wrapping: </span>
              <span className="text-blue-400 font-semibold">
                {formatWrapping(currentWrapping)}
              </span>
            </div>
          )}
        </div>

        {/* Output File */}
        <div>
          <label className="block text-sm font-semibold mb-2">Output MXF File</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={outputPath}
              onChange={(e) => setOutputPath(e.target.value)}
              placeholder="/path/to/output.mxf"
              className="flex-1 bg-gray-900 border border-gray-600 rounded px-4 py-2 text-white focus:outline-none focus:border-blue-500"
            />
            <button
              onClick={handleSelectOutput}
              disabled={loading}
              className="bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 px-4 py-2 rounded font-semibold transition-colors"
            >
              Browse
            </button>
          </div>
        </div>

        {/* Target Wrapping */}
        <div>
          <label className="block text-sm font-semibold mb-2">Target Wrapping</label>
          <div className="grid grid-cols-2 gap-4">
            <button
              onClick={() => setTargetWrapping('ClipWrapped')}
              disabled={loading}
              className={`py-3 px-4 rounded font-semibold transition-colors ${
                targetWrapping === 'ClipWrapped'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
              }`}
            >
              <div className="text-lg mb-1">📦 Clip-Wrapped</div>
              <div className="text-xs opacity-75">Better for playback</div>
            </button>
            <button
              onClick={() => setTargetWrapping('FrameWrapped')}
              disabled={loading}
              className={`py-3 px-4 rounded font-semibold transition-colors ${
                targetWrapping === 'FrameWrapped'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
              }`}
            >
              <div className="text-lg mb-1">🎬 Frame-Wrapped</div>
              <div className="text-xs opacity-75">Better for editing</div>
            </button>
          </div>
        </div>

        {/* Rewrap Button */}
        <button
          onClick={handleRewrap}
          disabled={!inputPath || !outputPath || loading}
          className="w-full bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 disabled:from-gray-600 disabled:to-gray-600 disabled:cursor-not-allowed px-6 py-4 rounded-lg font-bold text-lg transition-colors"
        >
          {loading ? 'Rewrapping...' : '🔄 Rewrap MXF File'}
        </button>
      </div>

      {/* Quick Actions */}
      <div className="mt-6 pt-6 border-t border-gray-700">
        <div className="text-sm font-semibold text-gray-400 mb-3">Quick Actions:</div>
        <div className="grid grid-cols-2 gap-3">
          <button
            onClick={async () => {
              if (!inputPath || !outputPath) return;
              setLoading(true);
              setError(null);
              setSuccess(null);
              try {
                await invoke('clip_to_frame', { inputPath, outputPath });
                setSuccess('✓ Converted to frame-wrapped');
              } catch (err) {
                setError(`Error: ${err}`);
              } finally {
                setLoading(false);
              }
            }}
            disabled={!inputPath || !outputPath || loading}
            className="bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed px-4 py-2 rounded text-sm font-semibold transition-colors"
          >
            Clip → Frame
          </button>
          <button
            onClick={async () => {
              if (!inputPath || !outputPath) return;
              setLoading(true);
              setError(null);
              setSuccess(null);
              try {
                await invoke('frame_to_clip', { inputPath, outputPath });
                setSuccess('✓ Converted to clip-wrapped');
              } catch (err) {
                setError(`Error: ${err}`);
              } finally {
                setLoading(false);
              }
            }}
            disabled={!inputPath || !outputPath || loading}
            className="bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed px-4 py-2 rounded text-sm font-semibold transition-colors"
          >
            Frame → Clip
          </button>
        </div>
      </div>
    </div>
  );
};

