import { invoke } from '@tauri-apps/api/tauri';

export interface MxfMetadata {
  material_package_uid: string;
  file_package_uid?: string;
  physical_source_package_uid?: string;
  timecode?: string;
  duration?: number;
  tracks: MxfTrack[];
}

export interface MxfTrack {
  track_id: number;
  track_type: string; // "video" | "audio" | "data"
  codec?: string;
}

export interface UnifyMobIdOptions {
  inputFiles: string[];
  targetMobId?: string;
  referenceFile?: string;
  outputDir: string;
  outputType?: 'avid' | 'op1a' | 'as11op1a' | 'as11d10' | 'rdd9' | 'as10';
}

/**
 * Extract metadata from an MXF file including MOB ID
 */
export async function extractMxfMetadata(filePath: string): Promise<MxfMetadata> {
  return invoke<MxfMetadata>('extract_mxf_metadata', { filePath });
}

/**
 * Unify MOB IDs across multiple MXF files so they all belong to the same material package
 * This is essential for MXF OP-atom workflows where each file contains one track
 */
export async function unifyMxfMobIds(options: UnifyMobIdOptions): Promise<string[]> {
  return invoke<string[]>('unify_mxf_mob_ids', {
    inputFiles: options.inputFiles,
    targetMobId: options.targetMobId,
    referenceFile: options.referenceFile,
    outputDir: options.outputDir,
    outputType: options.outputType || 'avid',
  });
}

/**
 * Check if multiple MXF files have consistent MOB IDs (i.e., belong together)
 */
export async function checkMxfMobConsistency(filePaths: string[]): Promise<boolean> {
  return invoke<boolean>('check_mxf_mob_consistency', { filePaths });
}

/**
 * Helper: Extract MOB ID from the first file and use it to unify all files
 */
export async function unifyWithFirstFile(
  inputFiles: string[],
  outputDir: string,
  outputType?: string
): Promise<string[]> {
  if (inputFiles.length === 0) {
    throw new Error('No input files provided');
  }

  return unifyMxfMobIds({
    inputFiles,
    outputDir,
    outputType: (outputType as any) || 'avid',
  });
}

/**
 * Helper: Extract MOB ID from a reference file and apply it to other files
 */
export async function unifyWithReference(
  referenceFile: string,
  inputFiles: string[],
  outputDir: string,
  outputType?: string
): Promise<string[]> {
  return unifyMxfMobIds({
    inputFiles,
    referenceFile,
    outputDir,
    outputType: (outputType as any) || 'avid',
  });
}

/**
 * Helper: Use a specific MOB ID for all files
 */
export async function unifyWithCustomMobId(
  mobId: string,
  inputFiles: string[],
  outputDir: string,
  outputType?: string
): Promise<string[]> {
  return unifyMxfMobIds({
    inputFiles,
    targetMobId: mobId,
    outputDir,
    outputType: (outputType as any) || 'avid',
  });
}

/**
 * Batch process: Check consistency and unify if needed
 */
export async function ensureConsistentMobIds(
  inputFiles: string[],
  outputDir: string,
  outputType?: string
): Promise<{ consistent: boolean; outputFiles?: string[] }> {
  const consistent = await checkMxfMobConsistency(inputFiles);
  
  if (consistent) {
    return { consistent: true };
  }
  
  // Not consistent, unify them
  const outputFiles = await unifyWithFirstFile(inputFiles, outputDir, outputType);
  
  return {
    consistent: false,
    outputFiles,
  };
}

