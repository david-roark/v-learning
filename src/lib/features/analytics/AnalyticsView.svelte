<script lang="ts">
  import { AlertTriangle, Award, BarChart3, Flame, Target } from "@lucide/svelte";
  import { onMount } from "svelte";
  import type { AnalyticsSummary } from "$lib/shared/types";
  import { analyticsApi } from "./api";

  let summary = $state<AnalyticsSummary | null>(null);
  let loading = $state(true);

  const maxForecast = $derived(
    summary ? Math.max(10, ...summary.forecast7d.map((d) => d.dueCount)) : 10,
  );
  const hasOverload = $derived(
    summary ? summary.forecast7d.some((d) => d.dueCount >= 100) : false,
  );

  function heatColor(count: number): string {
    if (count === 0) return "bg-zinc-800/70";
    if (count < 5) return "bg-emerald-900/80";
    if (count < 15) return "bg-emerald-700";
    if (count < 30) return "bg-emerald-500";
    return "bg-emerald-400";
  }

  onMount(async () => {
    try {
      summary = await analyticsApi.getSummary();
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div class="py-12 text-center text-sm text-zinc-500">Đang tính toán thống kê...</div>
{:else if summary}
  <div class="space-y-6">
    <!-- KPI Cards -->
    <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Flame class="h-4 w-4 text-amber-400" />
          Chuỗi ngày (Streak)
        </div>
        <p class="mt-2 text-2xl font-bold text-white">{summary.streakDays} ngày</p>
      </div>
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Target class="h-4 w-4 text-emerald-400" />
          True Retention
        </div>
        <p class="mt-2 text-2xl font-bold text-emerald-400">{summary.trueRetention}%</p>
      </div>
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Award class="h-4 w-4 text-indigo-400" />
          Từ đã thuộc vững
        </div>
        <p class="mt-2 text-2xl font-bold text-indigo-400">{summary.matureCards}</p>
      </div>
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <BarChart3 class="h-4 w-4 text-sky-400" />
          Tổng lượt ôn
        </div>
        <p class="mt-2 text-2xl font-bold text-white">{summary.totalReviews}</p>
      </div>
    </div>

    <!-- 90-day Heatmap -->
    <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-semibold text-white">Hoạt động 90 ngày gần nhất</h3>
        <div class="flex items-center gap-1.5 text-[11px] text-zinc-500">
          <span>Ít</span>
          <span class="h-2.5 w-2.5 rounded-xs bg-zinc-800"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-emerald-900"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-emerald-700"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-emerald-500"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-emerald-400"></span>
          <span>Nhiều</span>
        </div>
      </div>

      <div class="grid grid-cols-15 gap-1.5 sm:grid-cols-30">
        {#each summary.heatmap as day (day.date)}
          <div
            title={`${day.date}: ${day.count} lượt ôn`}
            class="aspect-square rounded-xs transition hover:ring-1 hover:ring-white/50 {heatColor(day.count)}"
          ></div>
        {/each}
      </div>
    </div>

    <!-- 7-day Workload Forecast -->
    <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-sm font-semibold text-white">Dự báo khối lượng ôn 7 ngày tới</h3>
        {#if hasOverload}
          <span class="inline-flex items-center gap-1.5 rounded-lg bg-amber-500/15 px-2.5 py-1 text-xs text-amber-300">
            <AlertTriangle class="h-3.5 w-3.5" />
            Có ngày vượt 100 thẻ — nên giảm thẻ mới
          </span>
        {/if}
      </div>

      <div class="grid grid-cols-7 items-end gap-3 pt-4">
        {#each summary.forecast7d as f, idx}
          {@const heightPct = Math.max(6, Math.round((f.dueCount / maxForecast) * 100))}
          <div class="flex flex-col items-center gap-2">
            <span class="text-xs font-medium text-zinc-300">{f.dueCount}</span>
            <div class="flex h-28 w-full items-end justify-center rounded-xl bg-zinc-950/80 p-1.5">
              <div
                class="w-full rounded-lg transition-all {idx === 0 ? 'bg-indigo-500' : 'bg-indigo-500/50'}"
                style={`height: ${f.dueCount === 0 ? 4 : heightPct}%`}
              ></div>
            </div>
            <span class="text-[11px] text-zinc-400">{idx === 0 ? "Hôm nay" : f.date}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
