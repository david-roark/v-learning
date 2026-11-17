<script lang="ts">
  import { Delete, Lightbulb, RotateCcw, Trophy, Volume2 } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { playWordAudio } from "$lib/shared/audio";
  import type { Note } from "$lib/shared/types";
  import { saveBestScore, scrambleLetters, shuffleArray } from "./gameEngine";

  interface Props {
    notes: Note[];
    onScoreUpdated: () => void;
  }

  let { notes, onScoreUpdated }: Props = $props();

  let queue = $state<Note[]>([]);
  let currentIndex = $state(0);
  let tiles = $state<Array<{ idx: number; ch: string; used: boolean }>>([]);
  let pickedIndices = $state<number[]>([]);
  let score = $state(0);
  let correctCount = $state(0);
  let status = $state<"playing" | "correct" | "wrong">("playing");
  let mistakes = $state<Note[]>([]);
  let finished = $state(false);
  let bestScore = $state<number | null>(null);
  let advanceTimer: ReturnType<typeof setTimeout> | null = null;

  const currentNote = $derived(queue[currentIndex] ?? null);
  const assembled = $derived(pickedIndices.map((ti) => tiles[ti]?.ch ?? "").join(""));

  function loadWord(note: Note) {
    const scrambled = scrambleLetters(note.word.trim());
    tiles = scrambled.map((ch, idx) => ({ idx, ch, used: false }));
    pickedIndices = [];
    status = "playing";
  }

  function startRound(targetPool: Note[] = notes) {
    if (advanceTimer) {
      clearTimeout(advanceTimer);
      advanceTimer = null;
    }
    const chosen = shuffleArray(targetPool).slice(0, 10);
    queue = chosen;
    currentIndex = 0;
    score = 0;
    correctCount = 0;
    mistakes = [];
    finished = chosen.length === 0;
    if (chosen[0]) loadWord(chosen[0]);
  }

  function pickTile(tileIdx: number) {
    if (status !== "playing" || !currentNote || tiles[tileIdx]?.used) return;
    tiles = tiles.map((t, i) => (i === tileIdx ? { ...t, used: true } : t));
    pickedIndices = [...pickedIndices, tileIdx];

    if (pickedIndices.length === currentNote.word.trim().length) {
      const attempt = pickedIndices.map((ti) => tiles[ti]?.ch ?? "").join("");
      if (attempt.toLowerCase() === currentNote.word.trim().toLowerCase()) {
        status = "correct";
        score += 20;
        correctCount += 1;
        void playWordAudio(currentNote.word);
      } else {
        status = "wrong";
        if (!mistakes.some((m) => m.id === currentNote.id)) {
          mistakes = [...mistakes, currentNote];
        }
      }

      if (advanceTimer) clearTimeout(advanceTimer);
      advanceTimer = setTimeout(() => {
        advanceTimer = null;
        if (currentIndex + 1 >= queue.length) {
          finished = true;
          bestScore = saveBestScore("scramble", score, false);
          onScoreUpdated();
        } else {
          currentIndex += 1;
          loadWord(queue[currentIndex]);
        }
      }, 750);
    }
  }

  function undoLastTile() {
    if (status !== "playing" || pickedIndices.length === 0) return;
    const last = pickedIndices[pickedIndices.length - 1];
    pickedIndices = pickedIndices.slice(0, -1);
    tiles = tiles.map((t, i) => (i === last ? { ...t, used: false } : t));
  }

  function hintNextLetter() {
    if (status !== "playing" || !currentNote) return;
    const targetChar = currentNote.word.trim()[pickedIndices.length]?.toLowerCase();
    if (!targetChar) return;
    const match = tiles.find((t) => !t.used && t.ch.toLowerCase() === targetChar);
    if (match) pickTile(match.idx);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (finished || status !== "playing") return;
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if (e.key === "Backspace") {
      e.preventDefault();
      undoLastTile();
      return;
    }
    if (e.key.length === 1) {
      const ch = e.key.toLowerCase();
      const match = tiles.find((t) => !t.used && t.ch.toLowerCase() === ch);
      if (match) {
        e.preventDefault();
        pickTile(match.idx);
      }
    }
  }

  onMount(() => startRound());
  onDestroy(() => {
    if (advanceTimer) clearTimeout(advanceTimer);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-4">
  <div
    class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-xs"
  >
    <div class="flex items-center gap-4">
      <span class="font-mono text-sm font-bold text-indigo-300">
        {Math.min(currentIndex + 1, queue.length)}/{queue.length}
      </span>
      <span class="text-zinc-400">
        Điểm: <strong class="text-white">{score}</strong>
      </span>
    </div>
    <button
      type="button"
      onclick={() => startRound()}
      class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <RotateCcw class="h-3.5 w-3.5" />
      Vòng mới
    </button>
  </div>

  {#if finished}
    <div class="space-y-4 rounded-2xl border border-indigo-500/30 bg-indigo-500/5 p-6 text-center">
      <Trophy class="mx-auto h-10 w-10 text-amber-400" />
      <div>
        <h3 class="text-xl font-bold text-white">{score} điểm</h3>
        <p class="mt-1 text-xs text-zinc-400">
          Đúng {correctCount}/{queue.length} từ
          {#if bestScore !== null}
            · Kỷ lục: <strong class="text-amber-300">{bestScore} điểm</strong>
          {/if}
        </p>
      </div>

      {#if mistakes.length > 0}
        <div class="mx-auto max-w-md space-y-1.5 text-left">
          <p class="text-xs font-semibold text-zinc-300">Từ cần ôn chính tả:</p>
          {#each mistakes as m (m.id)}
            <div
              class="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-900/70 px-3 py-2 text-xs"
            >
              <div>
                <span class="font-semibold text-white">{m.word}</span>
                <span class="ml-2 text-emerald-400">{m.meaning}</span>
              </div>
              <button
                type="button"
                onclick={() => playWordAudio(m.word)}
                class="rounded-lg p-1 text-zinc-400 hover:bg-zinc-800 hover:text-indigo-300"
              >
                <Volume2 class="h-3.5 w-3.5" />
              </button>
            </div>
          {/each}
        </div>
      {/if}

      <div class="flex flex-wrap justify-center gap-2">
        {#if mistakes.length > 0}
          <button
            type="button"
            onclick={() => startRound(mistakes)}
            class="rounded-xl border border-amber-500/50 bg-amber-500/15 px-4 py-2.5 text-xs font-semibold text-amber-200 hover:bg-amber-500/25"
          >
            Ôn ngay {mistakes.length} từ vừa sai
          </button>
        {/if}
        <button
          type="button"
          onclick={() => startRound()}
          class="rounded-xl bg-indigo-600 px-5 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500"
        >
          Chơi vòng mới
        </button>
      </div>
    </div>
  {:else if currentNote}
    <div class="space-y-4 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-center">
      <div>
        <p class="text-xs text-zinc-400">Ghép hoặc gõ lại từ đúng chính tả</p>
        <h3 class="mt-2 text-lg font-semibold text-emerald-400">{currentNote.meaning}</h3>
        {#if currentNote.ipa}
          <p class="mt-1 font-mono text-xs text-indigo-300">{currentNote.ipa}</p>
        {/if}
      </div>

      <!-- Assembled Answer Slot -->
      <div
        class="mx-auto flex min-h-12 max-w-md items-center justify-center rounded-xl border px-4 py-2 font-mono text-xl font-bold tracking-widest {status ===
        'correct'
          ? 'border-emerald-500 bg-emerald-500/15 text-emerald-300'
          : status === 'wrong'
            ? 'border-rose-500 bg-rose-500/15 text-rose-300'
            : 'border-zinc-700 bg-zinc-950 text-white'}"
      >
        {assembled || "···"}
      </div>
      {#if status === "wrong"}
        <p class="text-xs text-rose-300">
          Đáp án đúng: <strong class="font-mono">{currentNote.word}</strong>
        </p>
      {/if}

      <!-- Scrambled Letter Tiles -->
      <div class="flex flex-wrap justify-center gap-2 pt-1">
        {#each tiles as tile (tile.idx)}
          <button
            type="button"
            disabled={tile.used || status !== "playing"}
            onclick={() => pickTile(tile.idx)}
            class="h-10 min-w-10 rounded-xl border px-3 font-mono text-sm font-bold transition {tile.used
              ? 'border-zinc-800 bg-zinc-950 text-zinc-600 opacity-35'
              : 'border-zinc-700 bg-zinc-800 text-white hover:border-indigo-500 hover:bg-indigo-600/30'}"
          >
            {tile.ch}
          </button>
        {/each}
      </div>

      <div class="flex flex-wrap items-center justify-center gap-2 pt-2 text-xs">
        <button
          type="button"
          onclick={undoLastTile}
          disabled={pickedIndices.length === 0 || status !== "playing"}
          class="inline-flex items-center gap-1 rounded-xl border border-zinc-700 px-3 py-1.5 text-zinc-300 hover:bg-zinc-800 disabled:opacity-40"
        >
          <Delete class="h-3.5 w-3.5" />
          Xóa chữ cuối
        </button>
        <button
          type="button"
          onclick={hintNextLetter}
          disabled={status !== "playing"}
          class="inline-flex items-center gap-1 rounded-xl border border-zinc-700 px-3 py-1.5 text-amber-300 hover:bg-zinc-800 disabled:opacity-40"
        >
          <Lightbulb class="h-3.5 w-3.5" />
          Gợi ý
        </button>
        <button
          type="button"
          onclick={() => playWordAudio(currentNote!.word)}
          class="inline-flex items-center gap-1 rounded-xl border border-zinc-700 px-3 py-1.5 text-indigo-300 hover:bg-zinc-800"
        >
          <Volume2 class="h-3.5 w-3.5" />
          Nghe từ
        </button>
      </div>
    </div>
  {/if}
</div>
