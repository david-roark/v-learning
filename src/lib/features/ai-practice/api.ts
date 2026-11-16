import { invokeCmd } from "$lib/shared/tauri";
import type { AiPracticeFeedback, AiSettings } from "$lib/shared/types";

export const aiApi = {
  getSettings: () => invokeCmd<AiSettings>("get_ai_settings"),
  saveSettings: (settings: AiSettings) => invokeCmd<void>("save_ai_settings", { settings }),
  listModels: (baseUrl: string, apiKey: string) =>
    invokeCmd<string[]>("list_ai_models", { baseUrl, apiKey }),
  evaluatePractice: (targetWords: string[], userText: string) =>
    invokeCmd<AiPracticeFeedback>("evaluate_ai_practice", { targetWords, userText }),
};
