import { invokeCmd } from "$lib/shared/tauri";
import type { DailyStats, Rating, StudyCard } from "$lib/shared/types";

export const studyApi = {
  getQueue: (deckId: string, limit = 50) =>
    invokeCmd<StudyCard[]>("get_study_queue", { deckId, limit }),
  submitReview: (cardId: string, rating: Rating, durationMs?: number) =>
    invokeCmd<void>("submit_review", { cardId, rating, durationMs }),
  getDailyStats: () => invokeCmd<DailyStats>("get_daily_stats"),
};
