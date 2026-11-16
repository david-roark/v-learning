<script lang="ts">
  import { ArrowLeft, CheckCircle2 } from "@lucide/svelte";
  import { onMount } from "svelte";
  import type { DeckWithStats, Rating, StudyCard } from "$lib/shared/types";
  import { studyApi } from "../api";
  import FlashcardView from "./FlashcardView.svelte";
  import RatingBar from "./RatingBar.svelte";

  interface Props {
    deck: DeckWithStats;
    onExit: () => void;
  }

  let { deck, onExit }: Props = $props();

  let queue = $state<StudyCard[]>([]);
  let currentIndex = $state(0);
  let revealed = $state(false);
  let loading = $state(true);
  let submitting = $state(false);
  let cardShownAt = $state(Date.now());
  let completedCount = $state(0);

  const currentCard = $derived(queue[currentIndex] ?? null);

  function speakText(text: string) {
    if (!text || typeof window === "undefined" || !("speechSynthesis" in window)) return;
    window.speechSynthesis.cancel();
    const utter = new SpeechSynthesisUtterance(text);
    utter.lang = "en-US";
    utter.rate = 0.95;
    window.speechSynthesis.speak(utter);
  }

  async function loadQueue() {
    loading = true;
    try {
      queue = await studyApi.getQueue(deck.id, 50);
      currentIndex = 0;
      revealed = false;
      cardShownAt = Date.now();
    } finally {
      loading = false;
    }
  }

  async function handleRate(rating: Rating) {
    if (!currentCard || submitting) return;
    submitting = true;
    const durationMs = Date.now() - cardShownAt;

    try {
      await studyApi.submitReview(currentCard.card.id, rating, durationMs);
      completedCount += 1;
      currentIndex += 1;
      revealed = false;
      cardShownAt = Date.now();
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!currentCard || loading) return;
    if (e.key === "Escape") {
      onExit();
      return;
    }
    if ((e.key === " " || e.key === "Enter") && !revealed) {
      e.preventDefault();
      revealed = true;
      return;
    }
    if (e.key.toLowerCase() === "r") {
      e.preventDefault();
      speakText(currentCard.note.word);
      return;
    }
    if (revealed && ["1", "2", "3", "4"].includes(e.key)) {
      e.preventDefault();
      handleRate(Number(e.key) as Rating);
    }
  }

  onMount(() => {
    loadQueue();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="mx-auto max-w-2xl space-y-6">
  <div class="flex items-center justify-between">
    <button
      type="button"
      onclick={onExit}
      class="inline-flex items-center gap-2 rounded-xl border border-zinc-800 bg-zinc-900 px-3.5 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
    >
      <ArrowLeft class="h-4 w-4" />
      Quay lại ({deck.name})
    </button>

    <div class="text-xs font-medium text-zinc-400">
      {#if !loading && currentCard}
        Thẻ {currentIndex + 1} / {queue.length}
      {/if}
    </div>
  </div>

  {#if loading}
    <div class="flex min-h-[320px] items-center justify-center rounded-3xl border border-zinc-800 bg-zinc-900/50 text-sm text-zinc-400">
      Đang tải hàng đợi ôn tập...
    </div>
  {:else if !currentCard}
    <div class="flex flex-col items-center justify-center rounded-3xl border border-zinc-800 bg-zinc-900/60 px-6 py-16 text-center">
      <div class="mb-4 rounded-2xl bg-emerald-500/10 p-4 text-emerald-400">
        <CheckCircle2 class="h-10 w-10" />
      </div>
      <h3 class="text-xl font-semibold text-white">Hoàn thành phiên học!</h3>
      <p class="mt-2 text-sm text-zinc-400">
        Bạn đã ôn xong {completedCount} thẻ trong bộ "{deck.name}".
      </p>
      <button
        type="button"
        onclick={onExit}
        class="mt-6 rounded-xl bg-indigo-600 px-5 py-2.5 text-sm font-medium text-white hover:bg-indigo-500"
      >
        Về danh sách bộ thẻ
      </button>
    </div>
  {:else}
    <FlashcardView
      note={currentCard.note}
      {revealed}
      onSpeak={speakText}
    />

    {#if !revealed}
      <button
        type="button"
        onclick={() => (revealed = true)}
        class="w-full rounded-2xl bg-indigo-600 py-3.5 text-sm font-semibold text-white transition hover:bg-indigo-500"
      >
        Hiện đáp án <span class="ml-2 rounded bg-black/25 px-2 py-0.5 font-mono text-xs">Space</span>
      </button>
    {:else}
      <RatingBar intervals={currentCard.intervals} {submitting} onRate={handleRate} />
    {/if}

    <p class="text-center text-xs text-zinc-500">
      Phím tắt: <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">Space</kbd> lật thẻ ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">1-4</kbd> chấm điểm ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">R</kbd> nghe phát âm ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">Esc</kbd> thoát
    </p>
  {/if}
</div>
