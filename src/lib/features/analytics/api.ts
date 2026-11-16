import { invokeCmd } from "$lib/shared/tauri";
import type { AnalyticsSummary } from "$lib/shared/types";

export const analyticsApi = {
  getSummary: () => invokeCmd<AnalyticsSummary>("get_analytics_summary"),
};
