<script lang="ts">
  import { RotateCcw, Trophy, Volume2 } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { playWordAudio } from "$lib/shared/audio";
  import type { Note } from "$lib/shared/types";
  import { buildMatchRound, saveBestScore, type MatchRound } from "./gameEngine";

  interface Props {
    notes: Note[];
    onScoreUpdated: () => void;
  }

  let { notes, onScoreUpdated }: Props = $props();

  let round = $state<MatchRound>({ left: [], right: [] });
  let matchedIds = new SvelteSet<string>();
  let selectedLeftId = $state<string | null>(null);
  let selectedRightId = $state<string | null>(null);
  let wrongPair = $state<{ left: string; right: string } | null>(null);
  let mistakes = $state<Note[]>([]);
  let wrongCount = $state(0);
  let elapsedMs = $state(0);
  let finished = $state(false);
  let bestTimeMs = $state<number | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;
  let startedAt = 0;

  function startRound() {
    if (timer) clearInterval(timer);
    round = buildMatchRound(notes, 6);
    matchedIds.clear();
    selectedLeftId = null;
    selectedRightId = null;
    wrongPair = null;
    mistakes = [];
    wrongCount = 0;
    elapsedMs = 0;
    finished = false;
    startedAt = Date.now();
    timer = setInterval(() => {
      elapsedMs = Date.now() - startedAt;
    }, 100);
  }

  function checkMatch() {
    if (!selectedLeftId || !selectedRightId) return;
    const leftId = selectedLeftId;
    const rightId = selectedRightId;

    if (leftId === rightId) {
      matchedIds.add(leftId);
      selectedLeftId = null;
      selectedRightId = null;

      if (matchedIds.size === round.left.length && round.left.length > 0) {
        if (timer) clearInterval(timer);
        elapsedMs = Date.now() - startedAt;
        finished = true;
        bestTimeMs = saveBestScore("match", elapsedMs, true);
        onScoreUpdated();
      }
    } else {
      wrongCount += 1;
      const missed = notes.find((n) => n.id === leftId);
      if (missed && !mistakes.some((m) => m.id === missed.id)) {
        mistakes = [...mistakes, missed];
      }
      wrongPair = { left: leftId, right: rightId };
      selectedLeftId = null;
      selectedRightId = null;
      setTimeout(() => {
        wrongPair = null;
      }, 450);
    }
  }

  function pickLeft(id: string) {
    if (matchedIds.has(id) || finished) return;
    selectedLeftId = selectedLeftId === id ? null : id;
    checkMatch();
  }

  function pickRight(id: string) {
    if (matchedIds.has(id) || finished) return;
    selectedRightId = selectedRightId === id ? null : id;
    checkMatch();
  }

  onMount(startRound);
  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<div class="space-y-4">
  <div
    class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-xs"
  >
    <div class="flex items-center gap-4">
      <span class="font-mono text-sm font-semibold text-indigo-300">
        {(elapsedMs / 1000).toFixed(1)}s
      </span>
      <span class="text-zinc-400">
        Đã nối: <strong class="text-white">{matchedIds.size}/{round.left.length}</strong>
      </span>
      <span class="text-zinc-400">
        Sai: <strong class="text-rose-400">{wrongCount}</strong>
      </span>
    </div>
    <button
      type="button"
      onclick={startRound}
      class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-zinc-200 hover:bg-zinc-800"
    >
      <RotateCcw class="h-3.5 w-3.5" />
      Ván mới
    </button>
  </div>

  {#if finished}
    <div
      class="space-y-4 rounded-2xl border border-emerald-500/30 bg-emerald-500/5 p-6 text-center"
    >
      <Trophy class="mx-auto h-10 w-10 text-amber-400" />
      <div>
        <h3 class="text-lg font-bold text-white">
          Hoàn thành trong {(elapsedMs / 1000).toFixed(1)}s!
        </h3>
        <p class="mt-1 text-xs text-zinc-400">
          Số lần chọn nhầm: {wrongCount}
          {#if bestTimeMs !== null}
            · Kỷ lục nhanh nhất: <strong class="text-amber-300"
              >{(bestTimeMs / 1000).toFixed(1)}s</strong
            >
          {/if}
        </p>
      </div>

      {#if mistakes.length > 0}
        <div class="mx-auto max-w-md space-y-1.5 text-left">
          <p class="text-xs font-semibold text-zinc-300">Từ vừa chọn nhầm:</p>
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

      <button
        type="button"
        onclick={startRound}
        class="rounded-xl bg-indigo-600 px-5 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500"
      >
        Chơi ván tiếp theo
      </button>
    </div>
  {:else}
    <div class="grid gap-4 md:grid-cols-2">
      <div class="space-y-2">
        {#each round.left as card (card.id)}
          {@const isMatched = matchedIds.has(card.id)}
          {@const isSelected = selectedLeftId === card.id}
          {@const isWrong = wrongPair?.left === card.id}
          <button
            type="button"
            disabled={isMatched}
            onclick={() => pickLeft(card.id)}
            class="flex w-full items-center justify-between rounded-xl border px-4 py-3 text-left transition {isMatched
              ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300 opacity-45'
              : isWrong
                ? 'border-rose-500 bg-rose-500/20 text-rose-200'
                : isSelected
                  ? 'border-indigo-500 bg-indigo-500/20 text-white'
                  : 'border-zinc-800 bg-zinc-900/70 text-zinc-100 hover:border-zinc-700'}"
          >
            <div>
              <p class="text-sm font-semibold">{card.text}</p>
              {#if card.subText}
                <p class="font-mono text-[11px] text-indigo-300/80">{card.subText}</p>
              {/if}
            </div>
          </button>
        {/each}
      </div>

      <div class="space-y-2">
        {#each round.right as card (card.id)}
          {@const isMatched = matchedIds.has(card.id)}
          {@const isSelected = selectedRightId === card.id}
          {@const isWrong = wrongPair?.right === card.id}
          <button
            type="button"
            disabled={isMatched}
            onclick={() => pickRight(card.id)}
            class="flex w-full items-center rounded-xl border px-4 py-3 text-left text-sm transition {isMatched
              ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300 opacity-45'
              : isWrong
                ? 'border-rose-500 bg-rose-500/20 text-rose-200'
                : isSelected
                  ? 'border-indigo-500 bg-indigo-500/20 text-white'
                  : 'border-zinc-800 bg-zinc-900/70 text-zinc-200 hover:border-zinc-700'}"
          >
            <span class="line-clamp-2">{card.text}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
