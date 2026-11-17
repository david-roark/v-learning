<script lang="ts">
  import {
    ArrowLeft,
    BookOpenCheck,
    Gamepad2,
    Headphones,
    Puzzle,
    Sparkles,
    Zap,
  } from "@lucide/svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import Select from "$lib/shared/components/Select.svelte";
  import type { DeckWithStats, Note } from "$lib/shared/types";
  import ClozeGame from "./ClozeGame.svelte";
  import { buildClozeQuestion, loadBestScores, type GameModeId } from "./gameEngine";
  import MatchPairsGame from "./MatchPairsGame.svelte";
  import SpeedQuizGame from "./SpeedQuizGame.svelte";
  import WordScrambleGame from "./WordScrambleGame.svelte";

  interface Props {
    decks: DeckWithStats[];
  }

  let { decks }: Props = $props();

  let selectedDeckId = $state("__ALL__");
  let notes = $state<Note[]>([]);
  let loadingNotes = $state(false);
  let activeMode = $state<GameModeId | null>(null);
  let bestScores = $state<Record<string, number>>(loadBestScores());

  const deckOptions = $derived([
    { value: "__ALL__", label: "Tất cả bộ thẻ" },
    { value: "__RECENT__", label: "20 từ mới thêm gần đây" },
    ...decks.map((d) => ({ value: d.id, label: `${d.name} (${d.totalCards} thẻ)` })),
  ]);

  const validNotes = $derived(notes.filter((n) => n.word.trim() && n.meaning.trim()));
  const clozeEligible = $derived(
    validNotes.filter((n) => buildClozeQuestion(n, validNotes) !== null),
  );
  const scrambleEligible = $derived(
    validNotes.filter((n) => {
      const w = n.word.trim();
      return w.length >= 3 && w.length <= 18 && !w.includes(" ");
    }),
  );

  async function loadNotes() {
    if (decks.length === 0) {
      notes = [];
      return;
    }
    loadingNotes = true;
    try {
      if (selectedDeckId === "__ALL__" || selectedDeckId === "__RECENT__") {
        const lists = await Promise.all(decks.map((d) => notesApi.listByDeck(d.id)));
        const all = lists.flat();
        if (selectedDeckId === "__RECENT__") {
          all.sort((a, b) => b.createdAt - a.createdAt);
          notes = all.slice(0, 20);
        } else {
          notes = all;
        }
      } else {
        notes = await notesApi.listByDeck(selectedDeckId);
      }
    } finally {
      loadingNotes = false;
    }
  }

  $effect(() => {
    void selectedDeckId;
    void decks.length;
    void loadNotes();
  });

  function refreshScores() {
    bestScores = loadBestScores();
  }
</script>

<div class="w-full space-y-5">
  {#if activeMode}
    <div class="flex items-center justify-between">
      <button
        type="button"
        onclick={() => (activeMode = null)}
        class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800 hover:text-white"
      >
        <ArrowLeft class="h-3.5 w-3.5" />
        Chọn mini-game khác
      </button>
      <span class="text-xs text-zinc-400">
        Nguồn: <strong class="text-zinc-200">{validNotes.length}</strong> từ
      </span>
    </div>

    {#if activeMode === "match"}
      <MatchPairsGame notes={validNotes} onScoreUpdated={refreshScores} />
    {:else if activeMode === "cloze"}
      <ClozeGame notes={validNotes} onScoreUpdated={refreshScores} />
    {:else if activeMode === "scramble"}
      <WordScrambleGame notes={scrambleEligible} onScoreUpdated={refreshScores} />
    {:else}
      <SpeedQuizGame notes={validNotes} mode={activeMode} onScoreUpdated={refreshScores} />
    {/if}
  {:else}
    <div
      class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4"
    >
      <div class="flex items-center gap-3">
        <div class="rounded-xl bg-indigo-500/15 p-2.5 text-indigo-400">
          <Gamepad2 class="h-5 w-5" />
        </div>
        <div>
          <h2 class="text-sm font-semibold text-white">Mini-game ôn từ vựng</h2>
          <p class="text-xs text-zinc-400">
            Luyện phản xạ nhanh từ bộ thẻ của bạn (không làm thay đổi lịch ôn FSRS).
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Select options={deckOptions} bind:value={selectedDeckId} class="w-60" />
      </div>
    </div>

    {#if loadingNotes}
      <p class="py-10 text-center text-xs text-zinc-400">Đang tải từ vựng...</p>
    {:else if validNotes.length < 4}
      <div
        class="rounded-2xl border border-dashed border-zinc-800 bg-zinc-900/40 py-12 text-center text-xs text-zinc-400"
      >
        Cần tối thiểu <strong>4 từ vựng</strong> trong nguồn đã chọn để bắt đầu chơi (hiện có {validNotes.length}
        từ).
      </div>
    {:else}
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <button
          type="button"
          onclick={() => (activeMode = "match")}
          class="flex flex-col justify-between rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5 text-left transition hover:border-indigo-500/60 hover:bg-zinc-900"
        >
          <div>
            <div class="mb-3 inline-flex rounded-xl bg-emerald-500/15 p-2.5 text-emerald-400">
              <Puzzle class="h-5 w-5" />
            </div>
            <h3 class="text-sm font-semibold text-white">Nối cặp (Match Pairs)</h3>
            <p class="mt-1 text-xs text-zinc-400">
              Nối nhanh từ tiếng Anh với nghĩa tiếng Việt tương ứng.
            </p>
          </div>
          <p class="mt-4 text-[11px] text-zinc-400">
            Kỷ lục:
            <strong class="text-amber-300">
              {bestScores.match !== undefined ? `${(bestScores.match / 1000).toFixed(1)}s` : "—"}
            </strong>
          </p>
        </button>

        <button
          type="button"
          onclick={() => (activeMode = "quiz")}
          class="flex flex-col justify-between rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5 text-left transition hover:border-indigo-500/60 hover:bg-zinc-900"
        >
          <div>
            <div class="mb-3 inline-flex rounded-xl bg-amber-500/15 p-2.5 text-amber-400">
              <Zap class="h-5 w-5" />
            </div>
            <h3 class="text-sm font-semibold text-white">Trắc nghiệm 45s</h3>
            <p class="mt-1 text-xs text-zinc-400">
              Chọn nhanh nghĩa đúng (Anh ↔ Việt) trong 45 giây, combo x4 điểm!
            </p>
          </div>
          <p class="mt-4 text-[11px] text-zinc-400">
            Kỷ lục:
            <strong class="text-amber-300">
              {bestScores.quiz !== undefined ? `${bestScores.quiz} điểm` : "—"}
            </strong>
          </p>
        </button>

        <button
          type="button"
          onclick={() => (activeMode = "audio")}
          class="flex flex-col justify-between rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5 text-left transition hover:border-indigo-500/60 hover:bg-zinc-900"
        >
          <div>
            <div class="mb-3 inline-flex rounded-xl bg-sky-500/15 p-2.5 text-sky-400">
              <Headphones class="h-5 w-5" />
            </div>
            <h3 class="text-sm font-semibold text-white">Nghe & Bắt từ</h3>
            <p class="mt-1 text-xs text-zinc-400">
              Nghe phát âm và chọn mặt chữ đúng trong 45 giây.
            </p>
          </div>
          <p class="mt-4 text-[11px] text-zinc-400">
            Kỷ lục:
            <strong class="text-amber-300">
              {bestScores.audio !== undefined ? `${bestScores.audio} điểm` : "—"}
            </strong>
          </p>
        </button>

        <button
          type="button"
          disabled={clozeEligible.length === 0}
          onclick={() => (activeMode = "cloze")}
          class="flex flex-col justify-between rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5 text-left transition hover:border-indigo-500/60 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:opacity-45"
        >
          <div>
            <div class="mb-3 inline-flex rounded-xl bg-violet-500/15 p-2.5 text-violet-400">
              <BookOpenCheck class="h-5 w-5" />
            </div>
            <h3 class="text-sm font-semibold text-white">Điền từ vào câu</h3>
            <p class="mt-1 text-xs text-zinc-400">
              Đọc câu ngữ cảnh và chọn từ phù hợp vào chỗ trống ({clozeEligible.length} câu sẵn sàng).
            </p>
          </div>
          <p class="mt-4 text-[11px] text-zinc-400">
            Kỷ lục:
            <strong class="text-amber-300">
              {bestScores.cloze !== undefined ? `${bestScores.cloze} điểm` : "—"}
            </strong>
          </p>
        </button>

        <button
          type="button"
          disabled={scrambleEligible.length === 0}
          onclick={() => (activeMode = "scramble")}
          class="flex flex-col justify-between rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5 text-left transition hover:border-indigo-500/60 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:opacity-45"
        >
          <div>
            <div class="mb-3 inline-flex rounded-xl bg-rose-500/15 p-2.5 text-rose-400">
              <Sparkles class="h-5 w-5" />
            </div>
            <h3 class="text-sm font-semibold text-white">Xếp chữ chính tả</h3>
            <p class="mt-1 text-xs text-zinc-400">
              Ghép lại các chữ cái bị xáo trộn để luyện chính tả ({scrambleEligible.length} từ đơn).
            </p>
          </div>
          <p class="mt-4 text-[11px] text-zinc-400">
            Kỷ lục:
            <strong class="text-amber-300">
              {bestScores.scramble !== undefined ? `${bestScores.scramble} điểm` : "—"}
            </strong>
          </p>
        </button>
      </div>
    {/if}
  {/if}
</div>
