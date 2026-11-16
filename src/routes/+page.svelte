<script lang="ts">
  import { Flame, Layers, Plus, Sparkles } from "@lucide/svelte";
  import { onMount } from "svelte";
  import NoteFormModal from "$lib/features/card-editor/components/NoteFormModal.svelte";
  import { decksApi } from "$lib/features/decks/api";
  import DeckList from "$lib/features/decks/components/DeckList.svelte";
  import DeckModal from "$lib/features/decks/components/DeckModal.svelte";
  import { studyApi } from "$lib/features/study/api";
  import StudySession from "$lib/features/study/components/StudySession.svelte";
  import type { DailyStats, DeckWithStats } from "$lib/shared/types";

  let decks = $state<DeckWithStats[]>([]);
  let stats = $state<DailyStats>({ reviewedToday: 0, newLearnedToday: 0, dueRemaining: 0 });
  let loading = $state(true);

  let activeDeck = $state<DeckWithStats | null>(null);
  let showDeckModal = $state(false);
  let addCardTarget = $state<DeckWithStats | null>(null);

  async function refreshDashboard() {
    loading = true;
    try {
      const [deckList, daily] = await Promise.all([
        decksApi.list(),
        studyApi.getDailyStats(),
      ]);
      decks = deckList;
      stats = daily;
    } finally {
      loading = false;
    }
  }

  async function handleDeleteDeck(deck: DeckWithStats) {
    await decksApi.remove(deck.id);
    await refreshDashboard();
  }

  async function handleExitStudy() {
    activeDeck = null;
    await refreshDashboard();
  }

  onMount(() => {
    refreshDashboard();
  });
</script>

<div class="mx-auto max-w-5xl px-6 py-8">
  <header class="mb-8 flex flex-wrap items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-indigo-600 text-white shadow-lg shadow-indigo-600/25">
        <Sparkles class="h-5 w-5" />
      </div>
      <div>
        <h1 class="text-xl font-bold tracking-tight text-white">V-Learning</h1>
        <p class="text-xs text-zinc-400">FSRS Spaced Repetition · Local-first</p>
      </div>
    </div>

    {#if !activeDeck}
      <button
        type="button"
        onclick={() => (showDeckModal = true)}
        class="inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-indigo-500"
      >
        <Plus class="h-4 w-4" />
        Bộ thẻ mới
      </button>
    {/if}
  </header>

  {#if activeDeck}
    <StudySession deck={activeDeck} onExit={handleExitStudy} />
  {:else}
    <!-- Daily Stats Bar -->
    <div class="mb-8 grid grid-cols-3 gap-4">
      <div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs font-medium text-zinc-400">
          <Flame class="h-4 w-4 text-amber-400" />
          Đã ôn hôm nay
        </div>
        <p class="mt-2 text-2xl font-bold text-white">{stats.reviewedToday}</p>
      </div>
      <div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4">
        <div class="text-xs font-medium text-zinc-400">Từ mới hôm nay</div>
        <p class="mt-2 text-2xl font-bold text-sky-400">{stats.newLearnedToday}</p>
      </div>
      <div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4">
        <div class="flex items-center gap-2 text-xs font-medium text-zinc-400">
          <Layers class="h-4 w-4 text-indigo-400" />
          Tổng thẻ cần ôn
        </div>
        <p class="mt-2 text-2xl font-bold text-indigo-400">{stats.dueRemaining}</p>
      </div>
    </div>

    {#if loading}
      <div class="py-12 text-center text-sm text-zinc-500">Đang tải dữ liệu...</div>
    {:else}
      <DeckList
        {decks}
        onStudy={(deck) => (activeDeck = deck)}
        onAddCard={(deck) => (addCardTarget = deck)}
        onDelete={handleDeleteDeck}
        onCreateDeck={() => (showDeckModal = true)}
      />
    {/if}
  {/if}
</div>

<DeckModal
  open={showDeckModal}
  onClose={() => (showDeckModal = false)}
  onCreated={refreshDashboard}
/>

{#if addCardTarget}
  <NoteFormModal
    open={true}
    deckId={addCardTarget.id}
    deckName={addCardTarget.name}
    onClose={() => (addCardTarget = null)}
    onSaved={refreshDashboard}
  />
{/if}
