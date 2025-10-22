# MXF MOB ID Integration Guide

## Overview

This integration adds MXF OP-atom support to Transkoder, allowing you to extract metadata and unify MOB IDs across multiple MXF files. This is crucial for professional broadcast workflows where multiple MXF files (video + audio tracks) need to share the same Material Package UID to be recognized as a single clip.

## What Was Built

### 1. **mxf2raw & bmx Tools** (✓ Installed)
- Location: `/Users/Editor/Downloads/bmx-ebu/build/`
- Tools available:
  - `mxf2raw` - Extract metadata and raw essence
  - `bmxtranswrap` - Rewrap MXF files
  - `raw2bmx` - Create MXF from raw essence
  - `MXFDump` - Text dump of MXF files

### 2. **Rust Backend Module** (`src-tauri/src/mxf.rs`)
Cross-platform Rust module with:
- `extract_mxf_metadata()` - Extract MOB ID and metadata from MXF files
- `unify_mob_ids()` - Rewrap multiple files with same MOB ID
- `check_mob_id_consistency()` - Check if files belong together

### 3. **Tauri Commands** (exposed to frontend)
- `extract_mxf_metadata` - Get metadata from a file
- `unify_mxf_mob_ids` - Unify MOB IDs across files
- `check_mxf_mob_consistency` - Check consistency

### 4. **TypeScript API** (`src/mxf-api.ts`)
Type-safe frontend API with helper functions:
- `extractMxfMetadata()`
- `unifyMxfMobIds()`
- `checkMxfMobConsistency()`
- `ensureConsistentMobIds()` - Auto-check and unify

### 5. **React Component** (`src/components/MxfMobIdTool.tsx`)
Ready-to-use UI component with:
- File selection
- Metadata viewing
- Consistency checking
- One-click MOB ID unification

### 6. **Shell Script** (`unify_mob_id.sh`)
Standalone command-line tool for batch processing

## Cross-Platform Compatibility

### Current Platform Support
- ✅ **macOS** - Fully working (tested)
- ⚠️ **Windows** - Needs bmx compiled for Windows
- ⚠️ **Linux** - Needs bmx compiled for Linux

### Making It Cross-Platform

#### Option A: Bundle Pre-compiled Binaries (Recommended)
1. Compile bmx for each platform:
   ```bash
   # macOS (ARM)
   cd bmx-ebu/build && cmake --build .
   
   # macOS (Intel) - use appropriate build flags
   # Windows - use Visual Studio or cross-compile
   # Linux - use standard gcc/cmake build
   ```

2. Update `src-tauri/src/mxf.rs` to detect and use correct binary:
   ```rust
   fn get_mxf_tool_path(tool_name: &str) -> Result<PathBuf, String> {
       #[cfg(target_os = "macos")]
       let base_path = std::env::current_exe()?.parent()
           .unwrap().join("../Resources/bmx");
       
       #[cfg(target_os = "windows")]
       let base_path = std::env::current_exe()?.parent()
           .unwrap().join("bmx");
       
       #[cfg(target_os = "linux")]
       let base_path = std::env::current_exe()?.parent()
           .unwrap().join("bmx");
       
       // ... rest of function
   }
   ```

3. Configure Tauri to bundle binaries in `tauri.conf.json`:
   ```json
   {
     "tauri": {
       "bundle": {
         "resources": {
           "macos": ["resources/bmx/macos/*"],
           "windows": ["resources/bmx/windows/*"],
           "linux": ["resources/bmx/linux/*"]
         }
       }
     }
   }
   ```

#### Option B: Use Rust MXF Library (Future Enhancement)
Replace shell commands with pure Rust implementation using an MXF library (more complex but fully integrated).

## Usage Examples

### From TypeScript/React

```typescript
import { extractMxfMetadata, unifyMxfMobIds, ensureConsistentMobIds } from './mxf-api';

// Extract metadata from a file
const metadata = await extractMxfMetadata('/path/to/file.mxf');
console.log('MOB ID:', metadata.material_package_uid);

// Unify multiple files
const outputFiles = await unifyMxfMobIds({
  inputFiles: [
    '/path/to/video.mxf',
    '/path/to/audio1.mxf',
    '/path/to/audio2.mxf'
  ],
  outputDir: '/output/path',
  outputType: 'avid'
});

// Auto-check and unify if needed
const result = await ensureConsistentMobIds(
  ['/path/to/video.mxf', '/path/to/audio1.mxf'],
  '/output/path'
);

if (result.consistent) {
  console.log('Files already have same MOB ID');
} else {
  console.log('Files unified:', result.outputFiles);
}
```

### From Rust

```rust
use crate::mxf::{extract_mxf_metadata, unify_mob_ids, UnifyMobIdOptions};

// Extract metadata
let metadata = extract_mxf_metadata("/path/to/file.mxf")?;
println!("MOB ID: {}", metadata.material_package_uid);

// Unify files
let options = UnifyMobIdOptions {
    input_files: vec![
        "/path/to/video.mxf".into(),
        "/path/to/audio1.mxf".into(),
    ],
    target_mob_id: None, // Uses first file's MOB ID
    reference_file: None,
    output_dir: "/output/path".into(),
    output_type: "avid".to_string(),
};

let output_files = unify_mob_ids(options)?;
```

### From Command Line

```bash
# Using the shell script
/Users/Editor/Downloads/bmx-ebu/build/unify_mob_id.sh \
  video.mxf audio1.mxf audio2.mxf

# With options
./unify_mob_id.sh \
  -r reference.mxf \
  -o /output/dir \
  -t avid \
  video.mxf audio1.mxf audio2.mxf
```

## Integration into Your Workflow

### Add to Sidebar Navigation
```typescript
// In src/components/Sidebar.tsx
import { MxfMobIdTool } from './components/MxfMobIdTool';

// Add to your routing or tabs
<Tab onClick={() => setView('mxf-tools')}>
  MXF Tools
</Tab>
```

### Add as Pre-Processing Step
```typescript
// Before transcoding, ensure MOB IDs are consistent
async function preprocessMxfFiles(files: string[]) {
  const result = await ensureConsistentMobIds(files, '/tmp/preprocessed');
  
  if (!result.consistent && result.outputFiles) {
    // Use the unified files for transcoding
    return result.outputFiles;
  }
  
  return files; // Already consistent
}
```

## Testing

### Test with Sample MXF Files
```typescript
// Create test in src/__tests__/mxf.test.ts
import { extractMxfMetadata, checkMxfMobConsistency } from '../mxf-api';

test('extract metadata from MXF file', async () => {
  const metadata = await extractMxfMetadata('/path/to/test.mxf');
  expect(metadata.material_package_uid).toBeDefined();
  expect(metadata.material_package_uid.length).toBe(64);
});
```

## Deployment Checklist

- [ ] Compile bmx for target platforms (macOS, Windows, Linux)
- [ ] Bundle binaries with Tauri app
- [ ] Update `mxf.rs` with correct binary paths for each platform
- [ ] Test on each target platform
- [ ] Add error handling for missing binaries
- [ ] Document MXF workflow for users
- [ ] Add UI component to main app
- [ ] Create user documentation

## Why This Is Cross-Platform

1. **Rust Backend**: Compiles to native code for all platforms
2. **BMX Tools**: Open source, can be compiled for Windows/Linux/macOS
3. **Tauri**: Bundles everything into platform-specific installers
4. **TypeScript Frontend**: Runs in Tauri's webview on all platforms
5. **No Runtime Dependencies**: Everything is bundled in the app

## Next Steps

1. **Compile bmx for Windows and Linux** to complete cross-platform support
2. **Bundle binaries** in your Tauri app resources
3. **Test on all target platforms**
4. **Add to main UI** by importing `MxfMobIdTool` component
5. **Add workflow automation** - auto-detect MXF files and suggest unification

## Support

For issues with:
- **BMX tools**: See https://github.com/ebu/bmx
- **MXF format**: See SMPTE specifications
- **Integration**: Check the code comments in `mxf.rs` and `mxf-api.ts`

