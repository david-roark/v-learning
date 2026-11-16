import { invokeCmd } from "./tauri";

export type AudioSpeed = 0.8 | 1.0 | 1.2;

export function speakWithTts(text: string, speed: AudioSpeed = 1.0): void {
  if (!text || typeof window === "undefined" || !("speechSynthesis" in window)) return;
  window.speechSynthesis.cancel();
  const utter = new SpeechSynthesisUtterance(text);
  utter.lang = "en-US";
  utter.rate = speed;
  window.speechSynthesis.speak(utter);
}

export async function playWordAudio(
  word: string,
  fallbackText?: string,
  speed: AudioSpeed = 1.0,
): Promise<void> {
  try {
    const bytes = await invokeCmd<number[]>("fetch_audio", { word });
    const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);
    audio.playbackRate = speed;
    audio.onended = () => URL.revokeObjectURL(url);
    audio.onerror = () => {
      URL.revokeObjectURL(url);
      speakWithTts(fallbackText ?? word, speed);
    };
    await audio.play();
  } catch {
    speakWithTts(fallbackText ?? word, speed);
  }
}
