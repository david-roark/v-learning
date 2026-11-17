<script lang="ts">
  import { Flame, RotateCcw, Trophy, Volume2 } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { playWordAudio } from "$lib/shared/audio";
  import type { Note } from "$lib/shared/types";
  import {
    buildClozeQuestion,
    saveBestScore,
    shuffleArray,
    type ClozeQuestion,
    type QuizOption,
  } from "./gameEngine";

  interface Props {
    notes: Note[];
    onScoreUpdated: () => void;
  }

  let { notes, onScoreUpdated }: Props = $props();

  let questions = $state<ClozeQuestion[]>([]);
  let currentIndex = $state(0);
  let score = $state(0);
  let combo = $state(0);
  let correctCount = $state(0);
  let pickedId = $state<string | null>(null);
  let mistakes = $state<Note[]>([]);
  let finished = $state(false);
  let bestScore = $state<number | null>(null);
  let advanceTimer: ReturnType<typeof setTimeout> | null = null;

  const currentQuestion = $derived(questions[currentIndex] ?? null);

  function startRound(targetPool: Note[] = notes) {
    if (advanceTimer) {
      clearTimeout(advanceTimer);
      advanceTimer = null;
    }
    const built: ClozeQuestion[] = [];
    for (const n of shuffleArray(targetPool)) {
      const q = buildClozeQuestion(n, notes);
      if (q) built.push(q);
      if (built.length >= 10) break;
    }
    questions = built;
    currentIndex = 0;
    score = 0;
    combo = 0;
    correctCount = 0;
    pickedId = null;
    mistakes = [];
    finished = built.length === 0;
  }

  function chooseOption(opt: QuizOption) {
    if (finished || pickedId !== null || !currentQuestion) return;
    pickedId = opt.id;

    if (opt.isCorrect) {
      combo += 1;
      correctCount += 1;
      score += 15 * Math.min(3, combo);
    } else {
      combo = 0;
      const missed = currentQuestion.note;
      if (!mistakes.some((m) => m.id === missed.id)) {
        mistakes = [...mistakes, missed];
      }
    }

    if (advanceTimer) clearTimeout(advanceTimer);
    advanceTimer = setTimeout(() => {
      advanceTimer = null;
      if (currentIndex + 1 >= questions.length) {
        finished = true;
        bestScore = saveBestScore("cloze", score, false);
        onScoreUpdated();
      } else {
        currentIndex += 1;
        pickedId = null;
      }
    }, 550);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (finished || !currentQuestion || pickedId !== null) return;
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const idx = ["1", "2", "3", "4"].indexOf(e.key);
    if (idx >= 0 && currentQuestion.options[idx]) {
      e.preventDefault();
      chooseOption(currentQuestion.options[idx]);
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
        {Math.min(currentIndex + 1, questions.length)}/{questions.length}
      </span>
      <span class="text-zinc-400">
        Điểm: <strong class="text-white">{score}</strong>
      </span>
      {#if combo >= 2}
        <span
          class="inline-flex items-center gap-1 rounded-full bg-amber-500/15 px-2 py-0.5 font-semibold text-amber-300"
        >
          <Flame class="h-3.5 w-3.5" />
          x{Math.min(3, combo)}
        </span>
      {/if}
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
          Đúng {correctCount}/{questions.length} câu
          {#if bestScore !== null}
            · Kỷ lục: <strong class="text-amber-300">{bestScore} điểm</strong>
          {/if}
        </p>
      </div>

      {#if mistakes.length > 0}
        <div class="mx-auto max-w-md space-y-1.5 text-left">
          <p class="text-xs font-semibold text-zinc-300">Từ cần ôn lại:</p>
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
  {:else if currentQuestion}
    <div class="space-y-2 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-center">
      <p class="text-xs text-zinc-400">Chọn từ phù hợp điền vào chỗ trống (phím 1–4)</p>
      <p class="text-lg leading-relaxed font-medium text-white">
        {currentQuestion.maskedSentence}
      </p>
      <p class="text-xs text-emerald-400">Gợi ý: {currentQuestion.note.meaning}</p>
    </div>

    <div class="grid gap-2.5 sm:grid-cols-2">
      {#each currentQuestion.options as opt, idx (opt.id)}
        {@const showResult = pickedId !== null}
        {@const isPicked = pickedId === opt.id}
        <button
          type="button"
          disabled={showResult}
          onclick={() => chooseOption(opt)}
          class="flex items-center gap-3 rounded-xl border px-4 py-3.5 text-left text-sm transition {showResult
            ? opt.isCorrect
              ? 'border-emerald-500 bg-emerald-500/15 text-emerald-200'
              : isPicked
                ? 'border-rose-500 bg-rose-500/15 text-rose-200'
                : 'border-zinc-800 bg-zinc-900/40 text-zinc-500'
            : 'border-zinc-800 bg-zinc-900/70 text-zinc-100 hover:border-indigo-500/60 hover:bg-zinc-900'}"
        >
          <kbd class="rounded bg-zinc-800 px-1.5 py-0.5 font-mono text-[11px] text-zinc-400">
            {idx + 1}
          </kbd>
          <span class="flex-1">{opt.label}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>
