<script lang="ts">
  import { ArrowLeft, CheckCircle2, Headphones, Pencil, Sparkles, Undo2 } from "@lucide/svelte";
  import { onMount } from "svelte";
  import AiPracticeModal from "$lib/features/ai-practice/AiPracticeModal.svelte";
  import NoteFormModal from "$lib/features/card-editor/components/NoteFormModal.svelte";
  import DictationPanel from "$lib/features/dictation/DictationPanel.svelte";
  import ShadowingBar from "$lib/features/shadowing/ShadowingBar.svelte";
  import { playWordAudio, speakWithTts, type AudioSpeed } from "$lib/shared/audio";
  import type { Card, DeckWithStats, Note, Rating, StudyCard } from "$lib/shared/types";
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
  let audioSpeed = $state<AudioSpeed>(1.0);
  let loading = $state(true);
  let submitting = $state(false);
  let cardShownAt = $state(Date.now());
  let completedCount = $state(0);
  let reviewedItems = $state<Array<{ word: string; lapses: number }>>([]);
  let undoStack = $state<Array<{ index: number; card: Card }>>([]);
  let showAiPractice = $state(false);
  let editingNote = $state<Note | null>(null);

  const currentCard = $derived(queue[currentIndex] ?? null);
  const hardWords = $derived(
    [...reviewedItems]
      .sort((a, b) => b.lapses - a.lapses)
      .map((x) => x.word)
      .slice(0, 5),
  );

  function speakText(text: string) {
    if (!currentCard) return;
    if (text === currentCard.note.word) {
      void playWordAudio(currentCard.note.word, text, audioSpeed);
    } else {
      speakWithTts(text, audioSpeed);
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
    const snapshot = { index: currentIndex, card: { ...currentCard.card } };

    try {
      await studyApi.submitReview(currentCard.card.id, rating, durationMs);
      undoStack = [...undoStack, snapshot];
      if (!reviewedItems.some((x) => x.word === currentCard.note.word)) {
        reviewedItems = [
          ...reviewedItems,
          { word: currentCard.note.word, lapses: currentCard.card.lapses + (rating === 1 ? 1 : 0) },
        ];
      }
      completedCount += 1;
      currentIndex += 1;
      revealed = false;
      cardShownAt = Date.now();
    } finally {
      submitting = false;
    }
  }

  async function handleUndo() {
    const last = undoStack[undoStack.length - 1];
    if (!last || submitting) return;
    submitting = true;
    try {
      await studyApi.undoReview(last.card);
      undoStack = undoStack.slice(0, -1);
      completedCount = Math.max(0, completedCount - 1);
      currentIndex = last.index;
      revealed = true;
      cardShownAt = Date.now();
    } finally {
      submitting = false;
    }
  }

  function handleNoteUpdated(updated?: Note) {
    if (!updated || !currentCard) return;
    queue[currentIndex] = { ...currentCard, note: updated };
  }

  function handleKeydown(e: KeyboardEvent) {
    if (editingNote || showAiPractice || loading) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      void handleUndo();
      return;
    }
    if (!currentCard) return;
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
    if (e.key.toLowerCase() === "e") {
      e.preventDefault();
      editingNote = currentCard.note;
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

    <div class="flex flex-wrap items-center gap-2">
      {#if undoStack.length > 0}
        <button
          type="button"
          onclick={handleUndo}
          title="Hoàn tác (Ctrl+Z)"
          class="inline-flex items-center gap-1 rounded-xl border border-zinc-800 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
        >
          <Undo2 class="h-3.5 w-3.5" />
          Undo
        </button>
      {/if}

      {#if currentCard}
        <button
          type="button"
          onclick={() => (editingNote = currentCard.note)}
          title="Sửa thẻ (E)"
          class="inline-flex items-center gap-1 rounded-xl border border-zinc-800 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
        >
          <Pencil class="h-3.5 w-3.5" />
          Sửa
        </button>
      {/if}

      <div class="flex rounded-xl border border-zinc-800 bg-zinc-900 p-0.5 text-[11px]">
        {#each [0.8, 1.0, 1.2] as const as spd (spd)}
          <button
            type="button"
            onclick={() => (audioSpeed = spd)}
            class="rounded-lg px-2 py-1 font-mono transition {audioSpeed === spd
              ? 'bg-zinc-800 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            {spd}x
          </button>
        {/each}
      </div>

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
    <div
      class="flex min-h-[320px] items-center justify-center rounded-3xl border border-zinc-800 bg-zinc-900/50 text-sm text-zinc-400"
    >
      Đang tải hàng đợi ôn tập...
    </div>
  {:else if !currentCard}
    <div
      class="flex flex-col items-center justify-center rounded-3xl border border-zinc-800 bg-zinc-900/60 px-6 py-16 text-center"
    >
      <div class="mb-4 rounded-2xl bg-emerald-500/10 p-4 text-emerald-400">
        <CheckCircle2 class="h-10 w-10" />
      </div>
      <h3 class="text-xl font-semibold text-white">Hoàn thành phiên học!</h3>
      <p class="mt-2 text-sm text-zinc-400">
        Bạn đã ôn xong {completedCount} thẻ trong bộ "{deck.name}".
      </p>
      <div class="mt-6 flex flex-wrap justify-center gap-3">
        {#if hardWords.length > 0}
          <button
            type="button"
            onclick={() => (showAiPractice = true)}
            class="inline-flex items-center gap-2 rounded-xl border border-indigo-500/40 bg-indigo-500/15 px-4 py-2.5 text-sm font-medium text-indigo-300 hover:bg-indigo-500/25"
          >
            <Sparkles class="h-4 w-4" />
            Luyện đặt câu với AI ({hardWords.length} từ)
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
      <FlashcardView note={currentCard.note} {revealed} onSpeak={speakText} />
    {/if}

    {#if !revealed && !dictationMode}
      <button
        type="button"
        onclick={() => (revealed = true)}
        class="w-full rounded-2xl bg-indigo-600 py-3.5 text-sm font-semibold text-white transition hover:bg-indigo-500"
      >
        Hiện đáp án <span class="ml-2 rounded bg-black/25 px-2 py-0.5 font-mono text-xs">Space</span
        >
      </button>
    {:else}
      <ShadowingBar
        onPlaySample={() => speakText(currentCard.note.exampleSentence || currentCard.note.word)}
      />
      <RatingBar intervals={currentCard.intervals} {submitting} onRate={handleRate} />
    {/if}

    <p class="text-center text-xs text-zinc-500">
      Phím tắt: <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">Space</kbd> lật thẻ ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">1-4</kbd> chấm điểm ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">R</kbd> phát âm ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">E</kbd> sửa thẻ ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">Ctrl+Z</kbd> undo ·
      <kbd class="rounded bg-zinc-800 px-1.5 py-0.5">Esc</kbd> thoát
    </p>
  {/if}
</div>

<AiPracticeModal
  open={showAiPractice}
  targetWords={hardWords}
  onClose={() => (showAiPractice = false)}
/>

{#if editingNote}
  <NoteFormModal
    open={true}
    deckId={deck.id}
    deckName={deck.name}
    editNote={editingNote}
    onClose={() => (editingNote = null)}
    onSaved={handleNoteUpdated}
  />
{/if}
