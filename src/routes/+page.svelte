<script lang="ts">
  import {
    BarChart3,
    BookA,
    BookOpenText,
    Cloud,
    Flame,
    Layers,
    Palette,
    Plus,
    Settings,
    Sparkles,
    Upload,
  } from "@lucide/svelte";
  import { onMount } from "svelte";
  import AiSettingsModal from "$lib/features/ai-practice/AiSettingsModal.svelte";
  import AnalyticsView from "$lib/features/analytics/AnalyticsView.svelte";
  import AppearanceModal from "$lib/features/appearance/AppearanceModal.svelte";
  import NoteFormModal from "$lib/features/card-editor/components/NoteFormModal.svelte";
  import { decksApi } from "$lib/features/decks/api";
  import DeckList from "$lib/features/decks/components/DeckList.svelte";
  import DeckModal from "$lib/features/decks/components/DeckModal.svelte";
  import DeckNotesModal from "$lib/features/decks/components/DeckNotesModal.svelte";
  import DictionaryView from "$lib/features/dictionary/components/DictionaryView.svelte";
  import ReaderView from "$lib/features/reader/ReaderView.svelte";
  import { studyApi } from "$lib/features/study/api";
  import StudySession from "$lib/features/study/components/StudySession.svelte";
  import SyncModal from "$lib/features/sync/SyncModal.svelte";
  import { invokeCmd } from "$lib/shared/tauri";
  import type { ApkgImportReport, DailyStats, DeckWithStats } from "$lib/shared/types";

  let decks = $state<DeckWithStats[]>([]);
  let stats = $state<DailyStats>({ reviewedToday: 0, newLearnedToday: 0, dueRemaining: 0 });
  let loading = $state(true);

  let activeTab = $state<"decks" | "dictionary" | "reader" | "analytics">("decks");
  let activeDeck = $state<DeckWithStats | null>(null);
  let showDeckModal = $state(false);
  let showAiSettings = $state(false);
  let showSyncModal = $state(false);
  let showAppearance = $state(false);
  let addCardTarget = $state<DeckWithStats | null>(null);
  let manageNotesTarget = $state<DeckWithStats | null>(null);
  let apkgMsg = $state("");

  async function refreshDashboard() {
    loading = true;
    try {
      const [deckList, daily] = await Promise.all([decksApi.list(), studyApi.getDailyStats()]);
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

  async function handleImportApkg(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    apkgMsg = "Đang import file .apkg...";
    try {
      const buf = new Uint8Array(await file.arrayBuffer());
      const deckName = file.name.replace(/\.apkg$/i, "");
      const report = await invokeCmd<ApkgImportReport>("import_apkg", {
        deckName,
        bytes: Array.from(buf),
      });
      apkgMsg = `Đã import "${report.deckName}": ${report.imported} thẻ mới (${report.skipped} bỏ qua).`;
      await refreshDashboard();
    } catch (err) {
      apkgMsg = `Lỗi import .apkg: ${String(err)}`;
    } finally {
      input.value = "";
    }
  }

  onMount(() => {
    refreshDashboard();
  });
</script>

<div class="mx-auto max-w-5xl px-6 py-8">
  <header class="mb-8 flex flex-wrap items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      <div
        class="flex h-10 w-10 items-center justify-center rounded-xl bg-indigo-600 text-white shadow-lg shadow-indigo-600/25"
      >
        <Sparkles class="h-5 w-5" />
      </div>
      <div>
        <h1 class="text-xl font-bold tracking-tight text-white">V-Learning</h1>
        <p class="text-xs text-zinc-400">FSRS Spaced Repetition · Local-first</p>
      </div>
    </div>

    {#if !activeDeck}
      <div class="flex items-center gap-3">
        <div class="flex rounded-xl border border-zinc-800 bg-zinc-900 p-1 text-xs font-medium">
          <button
            type="button"
            onclick={() => (activeTab = "decks")}
            class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 transition {activeTab ===
            'decks'
              ? 'bg-zinc-800 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            <Layers class="h-3.5 w-3.5" />
            Bộ thẻ
          </button>
          <button
            type="button"
            onclick={() => (activeTab = "dictionary")}
            class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 transition {activeTab ===
            'dictionary'
              ? 'bg-zinc-800 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            <BookA class="h-3.5 w-3.5" />
            Từ điển
          </button>
          <button
            type="button"
            onclick={() => (activeTab = "reader")}
            class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 transition {activeTab ===
            'reader'
              ? 'bg-zinc-800 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            <BookOpenText class="h-3.5 w-3.5" />
            Đọc & Khai thác
          </button>
          <button
            type="button"
            onclick={() => (activeTab = "analytics")}
            class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 transition {activeTab ===
            'analytics'
              ? 'bg-zinc-800 text-white'
              : 'text-zinc-400 hover:text-zinc-200'}"
          >
            <BarChart3 class="h-3.5 w-3.5" />
            Thống kê
          </button>
        </div>

        <button
          type="button"
          onclick={() => (showAppearance = true)}
          title="Giao diện, Font & Cỡ chữ"
          class="rounded-xl border border-zinc-800 bg-zinc-900 p-2 text-zinc-400 hover:text-zinc-200"
        >
          <Palette class="h-4 w-4" />
        </button>

        <button
          type="button"
          onclick={() => (showSyncModal = true)}
          title="Sao lưu & Đồng bộ"
          class="rounded-xl border border-zinc-800 bg-zinc-900 p-2 text-zinc-400 hover:text-zinc-200"
        >
          <Cloud class="h-4 w-4" />
        </button>

        <button
          type="button"
          onclick={() => (showAiSettings = true)}
          title="Cấu hình AI Gia sư"
          class="rounded-xl border border-zinc-800 bg-zinc-900 p-2 text-zinc-400 hover:text-zinc-200"
        >
          <Settings class="h-4 w-4" />
        </button>

        {#if activeTab === "decks"}
          <label
            class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
          >
            <Upload class="h-3.5 w-3.5" />
            Import .apkg
            <input type="file" accept=".apkg" onchange={handleImportApkg} class="hidden" />
          </label>
          <button
            type="button"
            onclick={() => (showDeckModal = true)}
            class="inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-500"
          >
            <Plus class="h-4 w-4" />
            Bộ thẻ mới
          </button>
        {/if}
      </div>
    {/if}
  </header>

  {#if apkgMsg && activeTab === "decks" && !activeDeck}
    <p
      class="mb-4 rounded-xl border border-zinc-800 bg-zinc-900/80 px-4 py-2.5 text-xs text-zinc-300"
    >
      {apkgMsg}
    </p>
  {/if}

  {#if activeDeck}
    <StudySession deck={activeDeck} onExit={handleExitStudy} />
  {:else if activeTab === "dictionary"}
    <DictionaryView {decks} onCardAdded={refreshDashboard} />
  {:else if activeTab === "reader"}
    <ReaderView {decks} onCardAdded={refreshDashboard} />
  {:else if activeTab === "analytics"}
    <AnalyticsView />
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
        onManageNotes={(deck) => (manageNotesTarget = deck)}
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

{#if manageNotesTarget}
  <DeckNotesModal
    deck={manageNotesTarget}
    onClose={() => (manageNotesTarget = null)}
    onChanged={refreshDashboard}
  />
{/if}

<AiSettingsModal open={showAiSettings} onClose={() => (showAiSettings = false)} />

<SyncModal
  open={showSyncModal}
  onClose={() => (showSyncModal = false)}
  onRestored={refreshDashboard}
/>

<AppearanceModal open={showAppearance} onClose={() => (showAppearance = false)} />
