import { invokeCmd } from "./tauri";

export function speakWithTts(text: string): void {
  if (!text || typeof window === "undefined" || !("speechSynthesis" in window)) return;
  window.speechSynthesis.cancel();
  const utter = new SpeechSynthesisUtterance(text);
  utter.lang = "en-US";
  utter.rate = 0.95;
  window.speechSynthesis.speak(utter);
}

export async function playWordAudio(word: string, fallbackText?: string): Promise<void> {
  try {
    const bytes = await invokeCmd<number[]>("fetch_audio", { word });
    const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);
    audio.onended = () => URL.revokeObjectURL(url);
    audio.onerror = () => {
      URL.revokeObjectURL(url);
      speakWithTts(fallbackText ?? word);
    };
    await audio.play();
  } catch {
    speakWithTts(fallbackText ?? word);
  }
}
