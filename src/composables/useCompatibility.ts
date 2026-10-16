import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ReaderRequest {
  projectId: string;
  projectName: string;
  viewerId: string;
  cheatsEnabled: boolean;
  versionId?: string;
}
interface CompatibilityWarning {
  report: { editor: string; scripts: number; variable_operations: number };
  sourceUrl: string | null;
  request: ReaderRequest;
}
export const compatibilityWarning = ref<CompatibilityWarning | null>(null);
export async function openReader(request: ReaderRequest, allowIncompatible = false): Promise<boolean> {
  try {
    await invoke("open_viewer_window", { ...request, allowIncompatible });
    return true;
  } catch (error) {
    const message = String(error);
    const prefix = "VIEWER_COMPATIBILITY|";
    if (!message.startsWith(prefix)) throw error;
    compatibilityWarning.value = { ...JSON.parse(message.slice(prefix.length)), request };
    return false;
  }
}
