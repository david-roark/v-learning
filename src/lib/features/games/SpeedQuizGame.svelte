<script lang="ts">
  import { Flame, RotateCcw, Trophy, Volume2 } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { playWordAudio } from "$lib/shared/audio";
  import type { Note } from "$lib/shared/types";
  import {
    buildQuizQuestion,
    saveBestScore,
    shuffleArray,
    type QuizOption,
    type QuizQuestion,
  } from "./gameEngine";

  interface Props {
    notes: Note[];
    mode: "quiz" | "audio";
    onScoreUpdated: () => void;
  }

  let { notes, mode, onScoreUpdated }: Props = $props();

  const DURATION_SEC = 45;

  let timeLeft = $state(DURATION_SEC);
  let score = $state(0);
  let combo = $state(0);
  let maxCombo = $state(0);
  let answered = $state(0);
  let correctCount = $state(0);
  let question = $state<QuizQuestion | null>(null);
  let pickedId = $state<string | null>(null);
  let direction = $state<"en-to-vi" | "vi-to-en">("en-to-vi");
  let mistakes = $state<Note[]>([]);
  let finished = $state(false);
  let bestScore = $state<number | null>(null);
  let targetSource: Note[] = [];
  let queue: Note[] = [];
  let timer: ReturnType<typeof setInterval> | null = null;

  function nextQuestion() {
    pickedId = null;
    if (queue.length === 0) {
      queue = shuffleArray(targetSource.length > 0 ? targetSource : notes);
    }
    const target = queue.pop();
    if (!target) return;
    const effectiveDir = mode === "audio" ? "vi-to-en" : direction;
    question = buildQuizQuestion(target, notes, effectiveDir);
    if (mode === "audio") {
      void playWordAudio(target.word);
    }
  }

  function startGame(customTargets?: Note[]) {
    if (timer) clearInterval(timer);
    targetSource = customTargets && customTargets.length > 0 ? customTargets : notes;
    timeLeft = DURATION_SEC;
    score = 0;
    combo = 0;
    maxCombo = 0;
    answered = 0;
    correctCount = 0;
    mistakes = [];
    finished = false;
    queue = shuffleArray(targetSource);
    nextQuestion();

    timer = setInterval(() => {
      if (timeLeft <= 1) {
        timeLeft = 0;
        if (timer) clearInterval(timer);
        finished = true;
        bestScore = saveBestScore(mode, score, false);
        onScoreUpdated();
      } else {
        timeLeft -= 1;
      }
    }, 1000);
  }

  function chooseOption(opt: QuizOption) {
    if (finished || pickedId !== null || !question) return;
    pickedId = opt.id;
    answered += 1;

    if (opt.isCorrect) {
      combo += 1;
      maxCombo = Math.max(maxCombo, combo);
      correctCount += 1;
      score += 10 * Math.min(4, combo);
    } else {
      combo = 0;
      const missed = question.note;
      if (!mistakes.some((m) => m.id === missed.id)) {
        mistakes = [...mistakes, missed];
      }
    }

    setTimeout(() => {
      if (!finished) nextQuestion();
    }, 450);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (finished || !question || pickedId !== null) return;
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const idx = ["1", "2", "3", "4"].indexOf(e.key);
    if (idx >= 0 && question.options[idx]) {
      e.preventDefault();
      chooseOption(question.options[idx]);
    }
  }

  onMount(() => startGame());
  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-4">
  <div
    class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-xs"
  >
    <div class="flex items-center gap-4">
      <span
        class="font-mono text-sm font-bold {timeLeft <= 10 ? 'text-rose-400' : 'text-indigo-300'}"
      >
        {timeLeft}s
      </span>
      <span class="text-zinc-400">
        Điểm: <strong class="text-white">{score}</strong>
      </span>
      {#if combo >= 2}
        <span
          class="inline-flex items-center gap-1 rounded-full bg-amber-500/15 px-2 py-0.5 font-semibold text-amber-300"
        >
          <Flame class="h-3.5 w-3.5" />
          x{Math.min(4, combo)}
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if mode === "quiz"}
        <button
          type="button"
          onclick={() => {
            direction = direction === "en-to-vi" ? "vi-to-en" : "en-to-vi";
            startGame();
          }}
          class="rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-indigo-300 hover:bg-zinc-800"
        >
          {direction === "en-to-vi" ? "Anh → Việt" : "Việt → Anh"}
        </button>
      {/if}
      <button
        type="button"
        onclick={() => startGame()}
        class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-zinc-200 hover:bg-zinc-800"
      >
        <RotateCcw class="h-3.5 w-3.5" />
        Chơi lại
      </button>
    </div>
  </div>

  <div class="h-1.5 w-full overflow-hidden rounded-full bg-zinc-800">
    <div
      class="h-full rounded-full transition-all duration-500 {timeLeft <= 10
        ? 'bg-rose-500'
        : 'bg-indigo-500'}"
      style="width: {(timeLeft / DURATION_SEC) * 100}%"
    ></div>
  </div>

  {#if finished}
    <div class="space-y-4 rounded-2xl border border-indigo-500/30 bg-indigo-500/5 p-6 text-center">
      <Trophy class="mx-auto h-10 w-10 text-amber-400" />
      <div>
        <h3 class="text-xl font-bold text-white">{score} điểm</h3>
        <p class="mt-1 text-xs text-zinc-400">
          Đúng {correctCount}/{answered} câu · Max Combo x{Math.max(1, Math.min(4, maxCombo))}
          {#if bestScore !== null}
            · Kỷ lục: <strong class="text-amber-300">{bestScore} điểm</strong>
          {/if}
        </p>
      </div>

      {#if mistakes.length > 0}
        <div class="mx-auto max-w-md space-y-1.5 text-left">
          <p class="text-xs font-semibold text-zinc-300">Từ cần ôn lại:</p>
          {#each mistakes as m}
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
            onclick={() => startGame(mistakes)}
            class="rounded-xl border border-amber-500/50 bg-amber-500/15 px-4 py-2.5 text-xs font-semibold text-amber-200 hover:bg-amber-500/25"
          >
            Ôn ngay {mistakes.length} từ vừa sai
          </button>
        {/if}
        <button
          type="button"
          onclick={() => startGame()}
          class="rounded-xl bg-indigo-600 px-5 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500"
        >
          Chơi lại
        </button>
      </div>
    </div>
  {:else if question}
    <div class="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-center">
      {#if mode === "audio"}
        <p class="mb-3 text-xs text-zinc-400">Nghe phát âm và chọn từ đúng (phím 1–4)</p>
        <button
          type="button"
          onclick={() => playWordAudio(question!.note.word)}
          class="mx-auto inline-flex items-center gap-2 rounded-2xl bg-indigo-600 px-5 py-3 text-sm font-semibold text-white hover:bg-indigo-500"
        >
          <Volume2 class="h-5 w-5" />
          Nghe lại
        </button>
        {#if question.note.ipa}
          <p class="mt-2 font-mono text-xs text-indigo-300">{question.note.ipa}</p>
        {/if}
      {:else}
        <p class="mb-2 text-xs text-zinc-400">Chọn nghĩa đúng (phím 1–4)</p>
        <h3 class="text-2xl font-bold text-white">{question.prompt}</h3>
        {#if question.subPrompt}
          <p class="mt-1 font-mono text-xs text-indigo-300">{question.subPrompt}</p>
        {/if}
      {/if}
    </div>

    <div class="grid gap-2.5 sm:grid-cols-2">
      {#each question.options as opt, idx (opt.id)}
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
          <span class="line-clamp-2 flex-1">{opt.label}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>
