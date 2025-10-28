import React, { useState } from 'react';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

interface TranscodeJob {
  inputFile: string;
  outputDir: string;
  createVideo: boolean;
  createBwf: boolean;
}

export const SimpleTranscoder: React.FC = () => {
  const [inputFiles, setInputFiles] = useState<string[]>([]);
  const [outputDir, setOutputDir] = useState<string>('');
  const [createVideo, setCreateVideo] = useState(true);
  const [createBwf, setCreateBwf] = useState(true);
  const [processing, setProcessing] = useState(false);
  const [currentFile, setCurrentFile] = useState<string>('');
  const [currentFileIndex, setCurrentFileIndex] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleSelectInput = async () => {
    try {
      const selected = await open({
        multiple: true, // Enable multiple file selection
        filters: [
          { name: 'Media Files', extensions: ['mxf', 'mov', 'mp4'] },
        ],
      });

      if (selected) {
        // Handle both single file (string) and multiple files (array)
        const files = Array.isArray(selected) ? selected : [selected];
        setInputFiles(files);
        
        // Auto-set output directory to first file's directory
        if (!outputDir && files.length > 0) {
          const dir = files[0].substring(0, files[0].lastIndexOf('/'));
          setOutputDir(dir);
        }
        
        setError(null);
      }
    } catch (err) {
      setError(`Failed to select files: ${err}`);
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
        setError(null);
      }
    } catch (err) {
      setError(`Failed to select directory: ${err}`);
    }
  };

  const handleProcess = async () => {
    if (inputFiles.length === 0 || !outputDir) {
      setError('Please select input files and output directory');
      return;
    }

    if (!createVideo && !createBwf) {
      setError('Please select at least one output type');
      return;
    }

    setProcessing(true);
    setError(null);
    setSuccess(null);

    try {
      const allResults: string[] = [];

      // Process each file sequentially
      for (let i = 0; i < inputFiles.length; i++) {
        const inputFile = inputFiles[i];
        setCurrentFile(inputFile);
        setCurrentFileIndex(i + 1);

        const basename = inputFile.split('/').pop()?.replace(/\.[^/.]+$/, '') || 'output';
        const fileResults: string[] = [];

        // Start both processes for this file
        const promises: Promise<void>[] = [];

        // Video transcode (DNxHR LB MOV)
        if (createVideo) {
          const videoPromise = (async () => {
            const videoOutput = `${outputDir}/${basename}_transcoded.mov`;
            
            await invoke('transcode_dnxhr_lb', {
              inputPath: inputFile,
              outputPath: videoOutput,
            });
            
            fileResults.push(`✅ Video: ${basename}_transcoded.mov`);
          })();
          
          promises.push(videoPromise);
        }

        // BWF audio extraction
        if (createBwf) {
          const bwfPromise = (async () => {
            const bwfOutput = `${outputDir}/${basename}_audio.wav`;
            
            await invoke('create_bwf_from_mxf', {
              mxfPath: inputFile,
              outputPath: bwfOutput,
              sampleRate: 48000,
            });
            
            fileResults.push(`✅ BWF Audio: ${basename}_audio.wav`);
          })();
          
          promises.push(bwfPromise);
        }

        // Wait for this file to complete
        await Promise.all(promises);
        
        allResults.push(`\n📁 ${basename}:\n${fileResults.join('\n')}`);
      }

      setSuccess(`✅ Processed ${inputFiles.length} file(s):\n${allResults.join('\n')}`);
    } catch (err) {
      setError(`Processing failed on file ${currentFileIndex}/${inputFiles.length}: ${err}`);
    } finally {
      setProcessing(false);
      setCurrentFile('');
      setCurrentFileIndex(0);
    }
  };

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold mb-2">Fast Transcoder</h1>
      <p className="text-gray-400 mb-8">
        Hardware-accelerated DNxHR LB + Frame-accurate BWF Audio
      </p>

      {/* File Selection */}
      <div className="mb-8 space-y-4">
        <div>
          <label className="block text-sm font-semibold mb-2">Input Files</label>
          <div className="flex gap-2">
            <button
              onClick={handleSelectInput}
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-semibold transition-colors"
            >
              Select Files
            </button>
            {inputFiles.length > 0 && (
              <div className="flex-1 px-4 py-3 bg-gray-800 rounded-lg border border-gray-700">
                <div className="text-sm font-semibold text-blue-400 mb-2">
                  {inputFiles.length} file(s) selected
                </div>
                <div className="max-h-32 overflow-y-auto space-y-1">
                  {inputFiles.map((file, idx) => (
                    <div key={idx} className="text-xs text-gray-400 truncate flex items-center gap-2">
                      <span className="text-gray-500">{idx + 1}.</span>
                      <span className="truncate">{file.split('/').pop()}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        <div>
          <label className="block text-sm font-semibold mb-2">Output Directory</label>
          <div className="flex gap-2">
            <button
              onClick={handleSelectOutputDir}
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-semibold transition-colors"
            >
              Select Directory
            </button>
            {outputDir && (
              <div className="flex-1 px-4 py-3 bg-gray-800 rounded-lg border border-gray-700 truncate text-sm">
                {outputDir}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Output Options */}
      <div className="mb-8">
        <label className="block text-sm font-semibold mb-4">Output Files</label>
        <div className="space-y-3">
          <label className="flex items-center cursor-pointer p-4 bg-gray-800 rounded-lg border-2 border-gray-700 hover:border-blue-500 transition-colors">
            <input
              type="checkbox"
              checked={createVideo}
              onChange={(e) => setCreateVideo(e.target.checked)}
              className="w-5 h-5 rounded bg-gray-900 border-gray-600 text-blue-600 focus:ring-2 focus:ring-blue-500 cursor-pointer"
            />
            <div className="ml-4 flex-1">
              <div className="font-semibold">DNxHR LB QuickTime (MOV)</div>
              <div className="text-sm text-gray-400 mt-1">
                Hardware-accelerated • 7-8x realtime • Avid-ready • All 8 audio channels
              </div>
            </div>
          </label>

          <label className="flex items-center cursor-pointer p-4 bg-gray-800 rounded-lg border-2 border-gray-700 hover:border-blue-500 transition-colors">
            <input
              type="checkbox"
              checked={createBwf}
              onChange={(e) => setCreateBwf(e.target.checked)}
              className="w-5 h-5 rounded bg-gray-900 border-gray-600 text-blue-600 focus:ring-2 focus:ring-blue-500 cursor-pointer"
            />
            <div className="ml-4 flex-1">
              <div className="font-semibold">BWF Audio (WAV)</div>
              <div className="text-sm text-gray-400 mt-1">
                Frame-accurate BEXT timecode • 48kHz 24-bit • Stereo mixdown
              </div>
            </div>
          </label>
        </div>
      </div>

      {/* Process Button */}
      <div className="mb-8">
        <button
          onClick={handleProcess}
          disabled={processing || inputFiles.length === 0 || !outputDir || (!createVideo && !createBwf)}
          className="w-full py-4 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:bg-gray-700 disabled:cursor-not-allowed font-bold text-lg transition-colors"
        >
          {processing ? `🔄 Processing ${currentFileIndex}/${inputFiles.length}...` : `▶ Start Transcode (${inputFiles.length} file${inputFiles.length !== 1 ? 's' : ''})`}
        </button>
      </div>

      {/* Progress */}
      {processing && (
        <div className="mb-8 p-6 bg-gray-800 rounded-lg border border-gray-700">
          <h3 className="font-semibold mb-2">Processing File {currentFileIndex} of {inputFiles.length}</h3>
          <p className="text-sm text-gray-400 mb-4 truncate">
            {currentFile.split('/').pop()}
          </p>
          
          <div className="mb-4">
            <div className="flex justify-between text-sm mb-1">
              <span>Overall Progress</span>
              <span>{Math.round((currentFileIndex / inputFiles.length) * 100)}%</span>
            </div>
            <div className="w-full bg-gray-700 rounded-full h-3">
              <div 
                className="bg-gradient-to-r from-blue-500 to-purple-600 h-3 rounded-full transition-all"
                style={{ width: `${(currentFileIndex / inputFiles.length) * 100}%` }}
              />
            </div>
          </div>

          <div className="text-sm text-gray-400 space-y-1">
            {createVideo && <div>✓ Creating DNxHR LB video...</div>}
            {createBwf && <div>✓ Extracting BWF audio with timecode...</div>}
          </div>
        </div>
      )}

      {/* Success Message */}
      {success && (
        <div className="mb-8 p-6 bg-green-900/30 rounded-lg border border-green-600">
          <h3 className="font-semibold text-green-400 mb-2">✅ Success!</h3>
          <pre className="text-sm text-green-100 whitespace-pre-wrap">{success}</pre>
        </div>
      )}

      {/* Error Message */}
      {error && (
        <div className="mb-8 p-6 bg-red-900/30 rounded-lg border border-red-600">
          <h3 className="font-semibold text-red-400 mb-2">❌ Error</h3>
          <p className="text-red-100">{error}</p>
        </div>
      )}

      {/* Info */}
      <div className="p-6 bg-gray-800/50 rounded-lg border border-gray-700">
        <h4 className="font-semibold mb-3">System Info</h4>
        <div className="space-y-2 text-sm text-gray-300">
          <div className="flex justify-between">
            <span>Video Codec:</span>
            <span className="text-blue-400">DNxHR LB (45 Mbps)</span>
          </div>
          <div className="flex justify-between">
            <span>Audio Format:</span>
            <span className="text-purple-400">PCM 24-bit 48kHz</span>
          </div>
          <div className="flex justify-between">
            <span>Hardware Accel:</span>
            <span className="text-green-400">VideoToolbox (M2 Max)</span>
          </div>
          <div className="flex justify-between">
            <span>Expected Speed:</span>
            <span className="text-yellow-400">7-8x realtime</span>
          </div>
        </div>
      </div>
    </div>
  );
};

