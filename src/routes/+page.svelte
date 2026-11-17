<script lang="ts">
  import { Flame, Layers, Plus, Upload } from "@lucide/svelte";
  import { onMount } from "svelte";
  import AiSettingsModal from "$lib/features/ai-practice/AiSettingsModal.svelte";
  import AnalyticsView from "$lib/features/analytics/AnalyticsView.svelte";
  import AppearanceModal from "$lib/features/appearance/AppearanceModal.svelte";
  import TtsSettingsModal from "$lib/features/appearance/TtsSettingsModal.svelte";
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
  import Sidebar, { type NavTab } from "$lib/shared/components/Sidebar.svelte";
  import { invokeCmd } from "$lib/shared/tauri";
  import type { ApkgImportReport, DailyStats, DeckWithStats } from "$lib/shared/types";

  let decks = $state<DeckWithStats[]>([]);
  let stats = $state<DailyStats>({ reviewedToday: 0, newLearnedToday: 0, dueRemaining: 0 });
  let loading = $state(true);

  let activeTab = $state<NavTab>("decks");
  let activeDeck = $state<DeckWithStats | null>(null);
  let showDeckModal = $state(false);
  let showAiSettings = $state(false);
  let showSyncModal = $state(false);
  let showAppearance = $state(false);
  let showTtsSettings = $state(false);
  let addCardTarget = $state<DeckWithStats | null>(null);
  let manageNotesTarget = $state<DeckWithStats | null>(null);
  let apkgMsg = $state("");

  const TAB_TITLES: Record<NavTab, string> = {
    decks: "Bộ thẻ của bạn",
    dictionary: "Từ điển Offline & Online",
    reader: "Trình đọc & Khai thác câu",
    analytics: "Thống kê học tập",
  };

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

  function handleGlobalShortcut(e: KeyboardEvent) {
    if (activeDeck || !(e.ctrlKey || e.metaKey)) return;
    if (e.key === "1") activeTab = "decks";
    else if (e.key === "2") activeTab = "dictionary";
    else if (e.key === "3") activeTab = "reader";
    else if (e.key === "4") activeTab = "analytics";
  }

  onMount(() => {
    refreshDashboard();
  });
</script>

<svelte:window onkeydown={handleGlobalShortcut} />

{#if activeDeck}
  <div class="h-screen overflow-y-auto px-6 py-8">
    <StudySession deck={activeDeck} onExit={handleExitStudy} />
  </div>
{:else}
  <div class="flex h-screen overflow-hidden">
    <Sidebar
      {activeTab}
      onSelectTab={(tab) => (activeTab = tab)}
      onOpenAppearance={() => (showAppearance = true)}
      onOpenTtsSettings={() => (showTtsSettings = true)}
      onOpenSync={() => (showSyncModal = true)}
      onOpenAiSettings={() => (showAiSettings = true)}
    />

    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Compact Desktop Toolbar -->
      <header class="flex h-14 shrink-0 items-center justify-between border-b border-zinc-800 px-6">
        <h2 class="text-sm font-semibold text-white">{TAB_TITLES[activeTab]}</h2>

        {#if activeTab === "decks"}
          <div class="flex items-center gap-2.5">
            <label
              class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
            >
              <Upload class="h-3.5 w-3.5" />
              Import .apkg
              <input type="file" accept=".apkg" onchange={handleImportApkg} class="hidden" />
            </label>
            <button
              type="button"
              onclick={() => (showDeckModal = true)}
              class="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-3.5 py-1.5 text-xs font-semibold text-white transition hover:bg-indigo-500"
            >
              <Plus class="h-3.5 w-3.5" />
              Bộ thẻ mới
            </button>
          </div>
        {/if}
      </header>

      <!-- Scrollable Main Workspace -->
      <main class="flex-1 overflow-y-auto p-5">
        <div class="mx-auto w-full max-w-7xl">
          {#if apkgMsg && activeTab === "decks"}
            <p
              class="mb-4 rounded-xl border border-zinc-800 bg-zinc-900/80 px-4 py-2.5 text-xs text-zinc-300"
            >
              {apkgMsg}
            </p>
          {/if}

          {#if activeTab === "dictionary"}
            <DictionaryView {decks} onCardAdded={refreshDashboard} />
          {:else if activeTab === "analytics"}
            <AnalyticsView />
          {:else if activeTab === "decks"}
            <div class="mb-6 grid grid-cols-3 gap-4">
              <div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4">
                <div class="flex items-center gap-2 text-xs font-medium text-zinc-400">
                  <Flame class="h-4 w-4 text-amber-400" />
                  Đã ôn hôm nay
                </div>
                <p class="mt-1.5 text-2xl font-bold text-white">{stats.reviewedToday}</p>
              </div>
              <div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4">
                <div class="text-xs font-medium text-zinc-400">Từ mới hôm nay</div>
                <p class="mt-1.5 text-2xl font-bold text-sky-400">{stats.newLearnedToday}</p>
              </div>
              <div class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4">
                <div class="flex items-center gap-2 text-xs font-medium text-zinc-400">
                  <Layers class="h-4 w-4 text-indigo-400" />
                  Tổng thẻ cần ôn
                </div>
                <p class="mt-1.5 text-2xl font-bold text-indigo-400">{stats.dueRemaining}</p>
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

          <div class={activeTab === "reader" ? "" : "hidden"}>
            <ReaderView {decks} onCardAdded={refreshDashboard} />
          </div>
        </div>
      </main>
    </div>
  </div>
{/if}

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

<TtsSettingsModal open={showTtsSettings} onClose={() => (showTtsSettings = false)} />
