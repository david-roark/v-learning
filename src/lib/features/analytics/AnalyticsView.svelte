<script lang="ts">
  import { AlertTriangle, Award, BookOpen, Clock, Flame, Target, Volume2 } from "@lucide/svelte";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import type { AnalyticsSummary, DeckWithStats } from "$lib/shared/types";
  import { analyticsApi } from "./api";

  interface Props {
    decks?: DeckWithStats[];
  }

  let { decks = [] }: Props = $props();

  let selectedDeckId = $state("__ALL__");
  let summary = $state<AnalyticsSummary | null>(null);
  let loading = $state(true);

  const WEEKDAY_LABELS = ["T2", "T3", "T4", "T5", "T6", "T7", "CN"];
  const WEEKDAY_FULL = ["Thứ 2", "Thứ 3", "Thứ 4", "Thứ 5", "Thứ 6", "Thứ 7", "Chủ nhật"];

  const deckOptions = $derived([
    { value: "__ALL__", label: "Tất cả bộ thẻ" },
    ...decks.map((d) => ({ value: d.id, label: d.name })),
  ]);

  const maxForecast = $derived(
    summary ? Math.max(10, ...summary.forecast7d.map((d) => d.dueCount)) : 10,
  );
  const hasOverload = $derived(summary ? summary.forecast7d.some((d) => d.dueCount >= 100) : false);

  const maxHabitCell = $derived.by(() => {
    if (!summary) return 1;
    let m = 1;
    for (const row of summary.hourlyWeekdayHeatmap) {
      for (const c of row) {
        if (c > m) m = c;
      }
    }
    return m;
  });

  function heatColor(count: number): string {
    if (count === 0) return "bg-zinc-800";
    if (count < 5) return "bg-emerald-900/80";
    if (count < 15) return "bg-emerald-700";
    if (count < 30) return "bg-emerald-500";
    return "bg-emerald-400";
  }

  function habitColor(count: number, maxVal: number): string {
    if (count === 0) return "bg-zinc-800";
    const ratio = count / Math.max(1, maxVal);
    if (ratio < 0.25) return "bg-indigo-950/90";
    if (ratio < 0.5) return "bg-indigo-800/90";
    if (ratio < 0.75) return "bg-indigo-600";
    return "bg-indigo-400";
  }

  function formatDuration(ms: number): string {
    if (ms <= 0) return "0p";
    const totalSec = Math.round(ms / 1000);
    if (totalSec < 60) return `${totalSec}s`;
    const mins = Math.floor(totalSec / 60);
    if (mins < 60) return `${mins}p`;
    const hrs = Math.floor(mins / 60);
    const remMins = mins % 60;
    return remMins > 0 ? `${hrs}h ${remMins}p` : `${hrs}h`;
  }

  function pct(part: number, total: number): number {
    if (total <= 0) return 0;
    return Math.round((part / total) * 1000) / 10;
  }

  async function loadSummary(deckId: string) {
    loading = true;
    try {
      summary = await analyticsApi.getSummary(deckId);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void loadSummary(selectedDeckId);
  });
</script>

<div class="space-y-5">
  <!-- Filter Bar -->
  {#if decks.length > 0}
    <div class="flex flex-wrap items-center justify-between gap-2">
      <p class="text-xs text-zinc-400">
        Phân tích chi tiết quá trình ghi nhớ, phân bố đánh giá và thói quen học tập
      </p>
      <Select options={deckOptions} bind:value={selectedDeckId} prefix="Bộ thẻ: " class="w-60" />
    </div>
  {/if}

  {#if loading}
    <div class="py-12 text-center text-sm text-zinc-400">Đang tính toán thống kê...</div>
  {:else if summary}
    <!-- KPI Cards -->
    <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <BookOpen class="h-4 w-4 text-sky-400" />
          Tổng từ vựng
        </div>
        <p class="mt-2 text-2xl font-bold text-white">{summary.totalNotes}</p>
        <p class="mt-0.5 text-[11px] text-zinc-400">
          {summary.totalCards} thẻ · {summary.totalReviews} lượt ôn
        </p>
      </div>

      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Target class="h-4 w-4 text-emerald-400" />
          Tỷ lệ Nhớ / Quên
        </div>
        <p class="mt-2 text-2xl font-bold text-emerald-400">{summary.trueRetention}%</p>
        <p class="mt-0.5 text-[11px] text-rose-400">Quên: {summary.forgetRate}%</p>
      </div>

      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Award class="h-4 w-4 text-indigo-400" />
          Đã thuộc vững
        </div>
        <p class="mt-2 text-2xl font-bold text-indigo-400">{summary.matureCards}</p>
        <p class="mt-0.5 text-[11px] text-zinc-400">
          {pct(summary.matureCards, summary.totalCards)}% tổng số thẻ
        </p>
      </div>

      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Flame class="h-4 w-4 text-amber-400" />
          Chuỗi ngày (Streak)
        </div>
        <p class="mt-2 text-2xl font-bold text-white">{summary.streakDays} ngày</p>
        <p class="mt-0.5 text-[11px] text-zinc-400">Duy trì đều mỗi ngày</p>
      </div>

      <div class="col-span-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4 sm:col-span-1">
        <div class="flex items-center gap-2 text-xs text-zinc-400">
          <Clock class="h-4 w-4 text-sky-400" />
          Thời gian học
        </div>
        <p class="mt-2 text-2xl font-bold text-white">{formatDuration(summary.studyMsToday)}</p>
        <p class="mt-0.5 text-[11px] text-zinc-400">
          Tổng: {formatDuration(summary.totalStudyMs)} · TB {(summary.avgDurationMs / 1000).toFixed(
            1,
          )}s/thẻ
        </p>
      </div>
    </div>

    <!-- Card Maturity & Rating Distribution -->
    <div class="grid gap-4 lg:grid-cols-2">
      <!-- Card States Breakdown -->
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <div class="mb-3 flex items-center justify-between">
          <h3 class="text-sm font-semibold text-white">Phân bố độ thuộc của thẻ</h3>
          <span class="text-xs text-zinc-400">{summary.totalCards} thẻ</span>
        </div>

        <div class="flex h-3 w-full overflow-hidden rounded-full bg-zinc-800">
          {#if summary.totalCards > 0}
            <div
              class="bg-emerald-500"
              style={`width: ${(summary.matureCards / summary.totalCards) * 100}%`}
              title="Thuộc vững"
            ></div>
            <div
              class="bg-indigo-500"
              style={`width: ${(summary.youngCards / summary.totalCards) * 100}%`}
              title="Đang ôn (< 21 ngày)"
            ></div>
            <div
              class="bg-amber-500"
              style={`width: ${(summary.learningCards / summary.totalCards) * 100}%`}
              title="Đang học / học lại"
            ></div>
            <div
              class="bg-sky-500"
              style={`width: ${(summary.newCards / summary.totalCards) * 100}%`}
              title="Thẻ mới"
            ></div>
          {/if}
        </div>

        <div class="mt-4 grid grid-cols-2 gap-2.5 text-xs sm:grid-cols-4">
          <div class="rounded-xl bg-zinc-950/60 p-2.5">
            <div class="flex items-center gap-1.5 text-zinc-400">
              <span class="h-2 w-2 rounded-full bg-emerald-500"></span>
              Thuộc vững
            </div>
            <p class="mt-1 font-mono text-base font-bold text-emerald-400">
              {summary.matureCards}
            </p>
          </div>
          <div class="rounded-xl bg-zinc-950/60 p-2.5">
            <div class="flex items-center gap-1.5 text-zinc-400">
              <span class="h-2 w-2 rounded-full bg-indigo-500"></span>
              Đang ôn
            </div>
            <p class="mt-1 font-mono text-base font-bold text-indigo-400">
              {summary.youngCards}
            </p>
          </div>
          <div class="rounded-xl bg-zinc-950/60 p-2.5">
            <div class="flex items-center gap-1.5 text-zinc-400">
              <span class="h-2 w-2 rounded-full bg-amber-500"></span>
              Đang học
            </div>
            <p class="mt-1 font-mono text-base font-bold text-amber-400">
              {summary.learningCards}
            </p>
          </div>
          <div class="rounded-xl bg-zinc-950/60 p-2.5">
            <div class="flex items-center gap-1.5 text-zinc-400">
              <span class="h-2 w-2 rounded-full bg-sky-500"></span>
              Thẻ mới
            </div>
            <p class="mt-1 font-mono text-base font-bold text-sky-400">
              {summary.newCards}
            </p>
          </div>
        </div>
      </div>

      <!-- Rating Breakdown -->
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <div class="mb-3 flex items-center justify-between">
          <h3 class="text-sm font-semibold text-white">Tỷ lệ Nhớ / Quên theo đánh giá</h3>
          <span class="text-xs text-zinc-400">{summary.totalReviews} lượt</span>
        </div>

        <div class="space-y-2.5 text-xs">
          {#each [{ label: "1 · Quên (Again)", count: summary.ratingCounts.again, bar: "bg-rose-500", text: "text-rose-400" }, { label: "2 · Khó (Hard)", count: summary.ratingCounts.hard, bar: "bg-amber-500", text: "text-amber-400" }, { label: "3 · Tốt (Good)", count: summary.ratingCounts.good, bar: "bg-emerald-500", text: "text-emerald-400" }, { label: "4 · Dễ (Easy)", count: summary.ratingCounts.easy, bar: "bg-sky-500", text: "text-sky-400" }] as row (row.label)}
            {@const rowPct = pct(row.count, summary.totalReviews)}
            <div class="space-y-1">
              <div class="flex items-center justify-between">
                <span class="text-zinc-300">{row.label}</span>
                <span class="font-mono {row.text}">{row.count} ({rowPct}%)</span>
              </div>
              <div class="h-2 w-full overflow-hidden rounded-full bg-zinc-800">
                <div class="h-full rounded-full {row.bar}" style={`width: ${rowPct}%`}></div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- 7x24 Study Habit Heatmap (Weekday x Hour) -->
    <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
      <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div>
          <h3 class="text-sm font-semibold text-white">
            Thói quen học theo khung giờ & ngày trong tuần
          </h3>
          {#if summary.peakHour !== null && summary.peakWeekday !== null}
            <p class="mt-0.5 text-xs text-zinc-400">
              Giờ học nhiều nhất:
              <strong class="text-indigo-400">
                {String(summary.peakHour).padStart(2, "0")}:00–{String(summary.peakHour).padStart(
                  2,
                  "0",
                )}:59
              </strong>
              · Ngày chăm nhất:
              <strong class="text-indigo-400">{WEEKDAY_FULL[summary.peakWeekday]}</strong>
            </p>
          {:else}
            <p class="mt-0.5 text-xs text-zinc-400">
              Chưa có dữ liệu ôn tập để phân tích khung giờ.
            </p>
          {/if}
        </div>

        <div class="flex items-center gap-1.5 text-[11px] text-zinc-400">
          <span>Ít</span>
          <span class="h-2.5 w-2.5 rounded-xs bg-zinc-800"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-indigo-950/90"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-indigo-800/90"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-indigo-600"></span>
          <span class="h-2.5 w-2.5 rounded-xs bg-indigo-400"></span>
          <span>Nhiều</span>
        </div>
      </div>

      <div class="overflow-x-auto">
        <div class="min-w-[540px] space-y-1">
          <!-- Hour Labels -->
          <div
            class="grid grid-cols-[2rem_repeat(24,minmax(0,1fr))] gap-1 text-center text-[10px] text-zinc-400"
          >
            <span></span>
            {#each Array.from({ length: 24 }, (_, h) => h) as hr (hr)}
              <span class="font-mono">{hr % 3 === 0 ? `${hr}h` : ""}</span>
            {/each}
          </div>

          {#each summary.hourlyWeekdayHeatmap as row, wdIdx (wdIdx)}
            <div class="grid grid-cols-[2rem_repeat(24,minmax(0,1fr))] items-center gap-1">
              <span class="text-[11px] font-medium text-zinc-400">{WEEKDAY_LABELS[wdIdx]}</span>
              {#each row as count, hr (hr)}
                <div
                  title={`${WEEKDAY_FULL[wdIdx]}, ${String(hr).padStart(2, "0")}:00–${String(hr).padStart(2, "0")}:59: ${count} lượt ôn`}
                  class="aspect-square rounded-xs transition hover:ring-1 hover:ring-white/50 {habitColor(
                    count,
                    maxHabitCell,
                  )}"
                ></div>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- 180-day Heatmap -->
    <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-semibold text-white">Hoạt động 180 ngày gần nhất</h3>
        <div class="flex items-center gap-1.5 text-[11px] text-zinc-400">
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
            class="aspect-square rounded-xs transition hover:ring-1 hover:ring-white/50 {heatColor(
              day.count,
            )}"
          ></div>
        {/each}
      </div>
    </div>

    <!-- 14-day Workload Forecast + Leech Words -->
    <div class="grid gap-4 lg:grid-cols-3">
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5 lg:col-span-2">
        <div class="mb-4 flex flex-wrap items-center justify-between gap-2">
          <h3 class="text-sm font-semibold text-white">Dự báo khối lượng ôn 14 ngày tới</h3>
          {#if hasOverload}
            <span
              class="inline-flex items-center gap-1.5 rounded-lg bg-amber-500/15 px-2.5 py-1 text-xs text-amber-300"
            >
              <AlertTriangle class="h-3.5 w-3.5" />
              Có ngày vượt 100 thẻ — nên giảm thẻ mới
            </span>
          {/if}
        </div>

        <div class="grid grid-cols-7 items-end gap-2 pt-2 sm:grid-cols-14">
          {#each summary.forecast7d as f, idx (f.date)}
            {@const heightPct = Math.max(6, Math.round((f.dueCount / maxForecast) * 100))}
            <div class="flex flex-col items-center gap-1.5">
              <span class="text-[11px] font-medium text-zinc-300">{f.dueCount}</span>
              <div class="flex h-24 w-full items-end justify-center rounded-xl bg-zinc-950/80 p-1">
                <div
                  class="w-full rounded-md transition-all {idx === 0
                    ? 'bg-indigo-600'
                    : 'bg-indigo-500/50'}"
                  style={`height: ${f.dueCount === 0 ? 4 : heightPct}%`}
                ></div>
              </div>
              <span class="text-[10px] text-zinc-400">{idx === 0 ? "Nay" : f.date}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Leech Words -->
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <h3 class="mb-3 text-sm font-semibold text-white">Top từ hay quên nhất</h3>
        {#if summary.leechWords.length === 0}
          <p class="py-8 text-center text-xs text-zinc-400">
            Tuyệt vời! Chưa có từ nào bị quên nhiều lần.
          </p>
        {:else}
          <div class="space-y-2">
            {#each summary.leechWords as lw, idx (idx)}
              <div
                class="flex items-center justify-between gap-2 rounded-xl border border-zinc-800 bg-zinc-950/60 px-3 py-2 text-xs"
              >
                <div class="min-w-0">
                  <p class="truncate font-semibold text-white">{lw.word}</p>
                  <p class="truncate text-[11px] text-zinc-400">{lw.meaning}</p>
                </div>
                <div class="flex shrink-0 items-center gap-1.5">
                  <span
                    class="rounded-md bg-rose-500/15 px-1.5 py-0.5 font-mono text-[10px] text-rose-400"
                  >
                    {lw.lapses} quên
                  </span>
                  <button
                    type="button"
                    onclick={() => playWordAudio(lw.word)}
                    title="Nghe phát âm"
                    class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-indigo-400"
                  >
                    <Volume2 class="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
