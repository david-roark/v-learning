<script lang="ts">
  import { ArrowLeft, CheckCircle2, Headphones, Sparkles } from "@lucide/svelte";
  import { onMount } from "svelte";
  import AiPracticeModal from "$lib/features/ai-practice/AiPracticeModal.svelte";
  import DictationPanel from "$lib/features/dictation/DictationPanel.svelte";
  import ShadowingBar from "$lib/features/shadowing/ShadowingBar.svelte";
  import { playWordAudio, speakWithTts } from "$lib/shared/audio";
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
  let dictationMode = $state(false);
  let loading = $state(true);
  let submitting = $state(false);
  let cardShownAt = $state(Date.now());
  let completedCount = $state(0);
  let reviewedWords = $state<string[]>([]);
  let showAiPractice = $state(false);

  const currentCard = $derived(queue[currentIndex] ?? null);

  function speakText(text: string) {
    if (!currentCard) return;
    if (text === currentCard.note.word) {
      void playWordAudio(currentCard.note.word, text);
    } else {
      speakWithTts(text);
    }
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
      if (!reviewedWords.includes(currentCard.note.word)) {
        reviewedWords = [...reviewedWords, currentCard.note.word];
      }
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

    <div class="flex items-center gap-3">
      <button
        type="button"
        onclick={() => (dictationMode = !dictationMode)}
        class="inline-flex items-center gap-1.5 rounded-xl border px-3 py-1.5 text-xs font-medium transition {dictationMode
          ? 'border-indigo-500/60 bg-indigo-500/15 text-indigo-300'
          : 'border-zinc-800 bg-zinc-900 text-zinc-400 hover:text-zinc-200'}"
      >
        <Headphones class="h-3.5 w-3.5" />
        Dictation
      </button>

      <div class="text-xs font-medium text-zinc-400">
        {#if !loading && currentCard}
          Thẻ {currentIndex + 1} / {queue.length}
        {/if}
      </div>
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
      <div class="mt-6 flex flex-wrap justify-center gap-3">
        {#if reviewedWords.length > 0}
          <button
            type="button"
            onclick={() => (showAiPractice = true)}
            class="inline-flex items-center gap-2 rounded-xl border border-indigo-500/40 bg-indigo-500/15 px-4 py-2.5 text-sm font-medium text-indigo-300 hover:bg-indigo-500/25"
          >
            <Sparkles class="h-4 w-4" />
            Luyện đặt câu với AI ({Math.min(5, reviewedWords.length)} từ)
          </button>
        {/if}
        <button
          type="button"
          onclick={onExit}
          class="rounded-xl bg-indigo-600 px-5 py-2.5 text-sm font-medium text-white hover:bg-indigo-500"
        >
          Về danh sách bộ thẻ
        </button>
      </div>
    </div>
  {:else}
    {#if dictationMode}
      <DictationPanel
        targetText={currentCard.note.exampleSentence || currentCard.note.word}
        onPlay={() => speakText(currentCard.note.exampleSentence || currentCard.note.word)}
        onChecked={() => (revealed = true)}
      />
    {:else}
      <FlashcardView
        note={currentCard.note}
        {revealed}
        onSpeak={speakText}
      />
    {/if}

    {#if !revealed && !dictationMode}
      <button
        type="button"
        onclick={() => (revealed = true)}
        class="w-full rounded-2xl bg-indigo-600 py-3.5 text-sm font-semibold text-white transition hover:bg-indigo-500"
      >
        Hiện đáp án <span class="ml-2 rounded bg-black/25 px-2 py-0.5 font-mono text-xs">Space</span>
      </button>
    {:else}
      <ShadowingBar onPlaySample={() => speakText(currentCard.note.exampleSentence || currentCard.note.word)} />
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

<AiPracticeModal
  open={showAiPractice}
  targetWords={reviewedWords.slice(0, 5)}
  onClose={() => (showAiPractice = false)}
/>
