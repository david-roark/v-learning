import { invokeCmd } from "./tauri";
import type { TtsSettings } from "./types";

export type AudioSpeed = 0.8 | 1.0 | 1.2;

let cachedTtsSettings: TtsSettings | null = null;

export const ttsApi = {
  getSettings: async () => {
    const s = await invokeCmd<TtsSettings>("get_tts_settings");
    cachedTtsSettings = s;
    return s;
  },
  saveSettings: async (settings: TtsSettings) => {
    await invokeCmd<void>("save_tts_settings", { settings });
    cachedTtsSettings = settings;
  },
  previewProvider: (provider: string, voice: string, text: string) =>
    invokeCmd<number[]>("preview_tts_provider", { provider, voice, text }),
  speakSystem: (text: string, speed: AudioSpeed = 1.0) =>
    invokeCmd<void>("speak_system_tts", { text, speed }),
};

function speakBrowserTts(text: string, speed: AudioSpeed = 1.0): boolean {
  if (!text || typeof window === "undefined" || !("speechSynthesis" in window)) return false;
  if (window.speechSynthesis.getVoices().length === 0) return false;
  window.speechSynthesis.cancel();
  const utter = new SpeechSynthesisUtterance(text);
  utter.lang = "en-US";
  utter.rate = speed;
  window.speechSynthesis.speak(utter);
  return true;
}

export async function speakSystemTts(text: string, speed: AudioSpeed = 1.0): Promise<void> {
  const target = text.trim();
  if (!target) return;
  if (speakBrowserTts(target, speed)) return;
  await ttsApi.speakSystem(target, speed);
}

export async function playAudioBytes(bytes: number[], speed: AudioSpeed = 1.0): Promise<void> {
  const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
  const url = URL.createObjectURL(blob);
  const audio = new Audio(url);
  audio.playbackRate = speed;
  audio.onended = () => URL.revokeObjectURL(url);
  audio.onerror = () => URL.revokeObjectURL(url);
  await audio.play();
}

export function speakWithTts(text: string, speed: AudioSpeed = 1.0): void {
  if (!text.trim()) return;
  void playWordAudio(text, undefined, speed);
}

export async function playWordAudio(
  word: string,
  fallbackText?: string,
  speed: AudioSpeed = 1.0,
): Promise<void> {
  const target = word.trim();
  if (!target) return;

  const settings = cachedTtsSettings ?? (await ttsApi.getSettings().catch(() => null));
  const systemFirst = settings?.fallbackOrder?.[0] === "system_tts";

  if (systemFirst) {
    try {
      await speakSystemTts(fallbackText ?? target, speed);
      return;
    } catch {
      // Fall through to online TTS if OS TTS is unavailable
    }
  }

  try {
    const bytes = await invokeCmd<number[]>("fetch_audio", { word: target });
    await playAudioBytes(bytes, speed);
  } catch {
    await speakSystemTts(fallbackText ?? target, speed).catch(() => {});
  }
}
