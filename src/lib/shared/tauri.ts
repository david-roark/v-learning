import { invoke } from "@tauri-apps/api/core";
import { MUTATING_COMMANDS, scheduleAutoBackup } from "$lib/features/sync/autoBackup.svelte";

export async function invokeCmd<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const result = await invoke<T>(cmd, args);
  if (MUTATING_COMMANDS[cmd]) {
    scheduleAutoBackup();
  }
  return result;
}
