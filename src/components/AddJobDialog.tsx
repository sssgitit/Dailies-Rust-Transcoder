/**
 * Add Job Dialog Component
 * Dialog for creating new transcode jobs
 */

import React, { useState, useEffect } from 'react';
import { open } from '@tauri-apps/api/dialog';
import { addJob, getPresets, CodecPreset, Priority } from '../api/transcoder-api';

interface AddJobDialogProps {
  onClose: () => void;
  onJobAdded: () => void;
}

export const AddJobDialog: React.FC<AddJobDialogProps> = ({ onClose, onJobAdded }) => {
  const [inputPath, setInputPath] = useState('');
  const [outputDir, setOutputDir] = useState('');
  const [outputFilename, setOutputFilename] = useState('');
  const [presetName, setPresetName] = useState('DNxHR LB (Fast)');  // Changed to fast preset
  const [priority, setPriority] = useState<Priority>('Normal');
  const [createBwf, setCreateBwf] = useState(false);
  const [presets, setPresets] = useState<Record<string, CodecPreset>>({});
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    getPresets()
      .then(setPresets)
      .catch((err) => setError(`Failed to load presets: ${err}`));
  }, []);

  const handleSelectInput = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Media Files',
            extensions: ['mxf', 'mov', 'mp4', 'avi', 'mkv', 'wav', 'aiff'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setInputPath(selected);
        
        // Auto-generate output filename and directory
        if (!outputFilename) {
          const basename = selected.split('/').pop() || 'output';
          const ext = presets[presetName]?.config.container || 'mov';
          const newFilename = basename.replace(/\.[^/.]+$/, `_transcoded.${ext}`);
          setOutputFilename(newFilename);
        }
        
        // Auto-set output directory to input directory
        if (!outputDir) {
          const dir = selected.substring(0, selected.lastIndexOf('/'));
          setOutputDir(dir);
        }
      }
    } catch (err) {
      setError(`Failed to select file: ${err}`);
    }
  };

  const handleSelectOutputDir = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === 'string') {
        setOutputDir(selected);
      }
    } catch (err) {
      setError(`Failed to select directory: ${err}`);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      // Construct full output path
      const outputPath = `${outputDir}/${outputFilename}`;
      
      await addJob({
        input_path: inputPath,
        output_path: outputPath,
        preset_name: presetName,
        priority,
        create_bwf: createBwf,
      });

      onJobAdded();
    } catch (err) {
      setError(`Failed to add job: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const selectedPreset = presets[presetName];

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-8 max-w-2xl w-full max-h-[90vh] overflow-y-auto border border-gray-700">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold">Add Transcode Job</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white text-2xl"
          >
            ✕
          </button>
        </div>

        {error && (
          <div className="mb-6 bg-red-900 border border-red-600 rounded-lg p-4 text-red-100">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Input File */}
          <div>
            <label className="block text-sm font-semibold mb-2">Input File</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={inputPath}
                onChange={(e) => setInputPath(e.target.value)}
                placeholder="/path/to/input.mxf"
                className="flex-1 bg-gray-900 border border-gray-600 rounded px-4 py-2 text-white focus:outline-none focus:border-blue-500"
                required
              />
              <button
                type="button"
                onClick={handleSelectInput}
                className="bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded font-semibold transition-colors"
              >
                Browse
              </button>
            </div>
          </div>

          {/* Output Directory */}
          <div>
            <label className="block text-sm font-semibold mb-2">Output Directory</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={outputDir}
                onChange={(e) => setOutputDir(e.target.value)}
                placeholder="/path/to/output/directory"
                className="flex-1 bg-gray-900 border border-gray-600 rounded px-4 py-2 text-white focus:outline-none focus:border-blue-500"
                required
              />
              <button
                type="button"
                onClick={handleSelectOutputDir}
                className="bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded font-semibold transition-colors"
              >
                Browse
              </button>
            </div>
          </div>

          {/* Output Filename */}
          <div>
            <label className="block text-sm font-semibold mb-2">Output Filename</label>
            <input
              type="text"
              value={outputFilename}
              onChange={(e) => setOutputFilename(e.target.value)}
              placeholder="output.mov"
              className="w-full bg-gray-900 border border-gray-600 rounded px-4 py-2 text-white focus:outline-none focus:border-blue-500"
              required
            />
            <p className="mt-1 text-xs text-gray-500">
              Full path: {outputDir}/{outputFilename}
            </p>
          </div>

          {/* Preset Selection */}
          <div>
            <label className="block text-sm font-semibold mb-2">Codec Preset</label>
            <select
              value={presetName}
              onChange={(e) => setPresetName(e.target.value)}
              className="w-full bg-gray-900 border border-gray-600 rounded px-4 py-2 text-white focus:outline-none focus:border-blue-500"
            >
              {Object.keys(presets).map((name) => (
                <option key={name} value={name}>
                  {name}
                </option>
              ))}
            </select>
            {selectedPreset && (
              <div className="mt-2 p-4 bg-gray-900 rounded border border-gray-700 text-sm">
                <p className="text-gray-400 mb-2">{selectedPreset.description}</p>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-gray-500">Video:</span>{' '}
                    <span className="text-blue-400">{selectedPreset.config.video_codec}</span>
                  </div>
                  <div>
                    <span className="text-gray-500">Audio:</span>{' '}
                    <span className="text-blue-400">{selectedPreset.config.audio_codec}</span>
                  </div>
                  <div>
                    <span className="text-gray-500">Container:</span>{' '}
                    <span className="text-blue-400">{selectedPreset.config.container}</span>
                  </div>
                  {selectedPreset.config.audio_sample_rate && (
                    <div>
                      <span className="text-gray-500">Sample Rate:</span>{' '}
                      <span className="text-blue-400">
                        {selectedPreset.config.audio_sample_rate} Hz
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* BWF Audio Option */}
          <div>
            <label className="flex items-center cursor-pointer p-4 bg-gray-900 rounded-lg border-2 border-gray-700 hover:border-purple-500 transition-colors">
              <input
                type="checkbox"
                checked={createBwf}
                onChange={(e) => setCreateBwf(e.target.checked)}
                className="w-5 h-5 rounded bg-gray-900 border-gray-600 text-purple-600 focus:ring-2 focus:ring-purple-500 cursor-pointer"
              />
              <div className="ml-4 flex-1">
                <div className="font-semibold">Also Create BWF Audio (WAV)</div>
                <div className="text-sm text-gray-400 mt-1">
                  Frame-accurate BEXT timecode • 48kHz 24-bit • Stereo mixdown • Auto-extracted from video
                </div>
              </div>
            </label>
          </div>

          {/* Priority */}
          <div>
            <label className="block text-sm font-semibold mb-2">Priority</label>
            <div className="grid grid-cols-4 gap-2">
              {(['Low', 'Normal', 'High', 'Urgent'] as Priority[]).map((p) => (
                <button
                  key={p}
                  type="button"
                  onClick={() => setPriority(p)}
                  className={`py-2 px-4 rounded font-semibold transition-colors ${
                    priority === p
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                  }`}
                >
                  {p}
                </button>
              ))}
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-4 pt-4">
            <button
              type="submit"
              disabled={loading}
              className="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-6 py-3 rounded-lg font-semibold transition-colors"
            >
              {loading ? 'Adding Job...' : 'Add Job'}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="px-6 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg font-semibold transition-colors"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

