import { invokeCmd } from "$lib/shared/tauri";
import type { AnalyticsSummary } from "$lib/shared/types";

export const analyticsApi = {
  getSummary: (deckId?: string) =>
    invokeCmd<AnalyticsSummary>("get_analytics_summary", {
      deckId: deckId && deckId !== "__ALL__" ? deckId : null,
    }),
};
