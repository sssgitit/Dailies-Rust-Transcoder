import { invoke } from '@tauri-apps/api/tauri';

export interface BwfTimecode {
  hours: number;
  minutes: number;
  seconds: number;
  frames: number;
}

export interface BwfCreateOptions {
  inputFile: string;
  outputFile: string;
  timecode: BwfTimecode;
  sampleRate?: number;
  description?: string;
}

/**
 * Calculate BEXT TimeReference for 23.976fps
 * Uses the validated frame-based method
 */
export async function calculateBextTimecode(
  timecode: BwfTimecode
): Promise<number> {
  return invoke<number>('calculate_bext_timecode', {
    hours: timecode.hours,
    minutes: timecode.minutes,
    seconds: timecode.seconds,
    frames: timecode.frames,
  });
}

/**
 * Extract timecode from MXF file
 */
export async function extractTimecodeFromFile(
  filePath: string
): Promise<BwfTimecode> {
  return invoke<BwfTimecode>('extract_timecode_from_file', { filePath });
}

/**
 * Create BWF file with BEXT chunk containing timecode
 */
export async function createBwfFile(
  options: BwfCreateOptions
): Promise<void> {
  return invoke('create_bwf_file', {
    inputFile: options.inputFile,
    outputFile: options.outputFile,
    hours: options.timecode.hours,
    minutes: options.timecode.minutes,
    seconds: options.timecode.seconds,
    frames: options.timecode.frames,
    sampleRate: options.sampleRate || 48000,
    description: options.description,
  });
}

/**
 * Format timecode as string (HH:MM:SS:FF)
 */
export function formatTimecode(tc: BwfTimecode): string {
  return `${tc.hours.toString().padStart(2, '0')}:${tc.minutes
    .toString()
    .padStart(2, '0')}:${tc.seconds.toString().padStart(2, '0')}:${tc.frames
    .toString()
    .padStart(2, '0')}`;
}

/**
 * Parse timecode string to object
 */
export function parseTimecode(tcString: string): BwfTimecode {
  const parts = tcString.split(/[:;]/).map((p) => parseInt(p, 10));
  
  if (parts.length !== 4 || parts.some(isNaN)) {
    throw new Error('Invalid timecode format. Use HH:MM:SS:FF');
  }
  
  return {
    hours: parts[0],
    minutes: parts[1],
    seconds: parts[2],
    frames: parts[3],
  };
}

/**
 * Validate timecode values
 */
export function validateTimecode(tc: BwfTimecode): string | null {
  if (tc.hours < 0 || tc.hours > 23) {
    return 'Hours must be 0-23';
  }
  if (tc.minutes < 0 || tc.minutes > 59) {
    return 'Minutes must be 0-59';
  }
  if (tc.seconds < 0 || tc.seconds > 59) {
    return 'Seconds must be 0-59';
  }
  if (tc.frames < 0 || tc.frames > 23) {
    return 'Frames must be 0-23 for 23.976fps';
  }
  return null;
}

/**
 * Helper: Create BWF from MXF with auto-extracted timecode
 */
export async function createBwfFromMxf(
  mxfPath: string,
  outputPath: string,
  sampleRate: number = 48000
): Promise<void> {
  const timecode = await extractTimecodeFromFile(mxfPath);
  
  await createBwfFile({
    inputFile: mxfPath,
    outputFile: outputPath,
    timecode,
    sampleRate,
    description: `Converted from ${mxfPath.split('/').pop()}`,
  });
}

