import React, { useState } from 'react';
import { SimpleTranscoder } from './components/SimpleTranscoder';
import { TranscoderDashboard } from './components/TranscoderDashboard';
import { MxfRewrapTool } from './components/MxfRewrapTool';
import './App.css';

type Tab = 'simple' | 'dashboard' | 'mxf';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('simple');

  return (
    <div className="App min-h-screen bg-gray-900 text-white">
      {/* Tab Navigation */}
      <div className="bg-gray-800 border-b border-gray-700">
        <div className="flex gap-1 p-2">
          <button
            onClick={() => setActiveTab('simple')}
            className={`px-6 py-3 rounded-t-lg font-semibold transition-colors ${
              activeTab === 'simple'
                ? 'bg-gray-900 text-white border-b-2 border-blue-500'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-white'
            }`}
          >
            🎬 Quick Transcode & BWF
          </button>
          <button
            onClick={() => setActiveTab('dashboard')}
            className={`px-6 py-3 rounded-t-lg font-semibold transition-colors ${
              activeTab === 'dashboard'
                ? 'bg-gray-900 text-white border-b-2 border-blue-500'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-white'
            }`}
          >
            📊 Job Queue Dashboard
          </button>
          <button
            onClick={() => setActiveTab('mxf')}
            className={`px-6 py-3 rounded-t-lg font-semibold transition-colors ${
              activeTab === 'mxf'
                ? 'bg-gray-900 text-white border-b-2 border-blue-500'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600 hover:text-white'
            }`}
          >
            🎞️ MXF Tools
          </button>
        </div>
      </div>

      {/* Tab Content */}
      <div>
        {activeTab === 'simple' && <SimpleTranscoder />}
        {activeTab === 'dashboard' && <TranscoderDashboard />}
        {activeTab === 'mxf' && <MxfRewrapTool />}
      </div>
    </div>
  );
}

export default App;

