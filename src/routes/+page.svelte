<script lang="ts">
  import { Flame, Layers, Plus, Sparkles, Upload } from "@lucide/svelte";
  import { onMount } from "svelte";
  import AboutModal from "$lib/features/about/AboutModal.svelte";
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
  import MiniGameView from "$lib/features/games/MiniGameView.svelte";
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
  let showAbout = $state(false);
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

  function handleGlobalShortcut(e: KeyboardEvent) {
    if (activeDeck || !(e.ctrlKey || e.metaKey)) return;
    if (e.key === "1") activeTab = "decks";
    else if (e.key === "2") activeTab = "dictionary";
    else if (e.key === "3") activeTab = "reader";
    else if (e.key === "4") activeTab = "games";
    else if (e.key === "5") activeTab = "analytics";
  }

  onMount(() => {
    refreshDashboard();
    const onRestored = () => {
      void refreshDashboard();
    };
    window.addEventListener("vlearning:data-restored", onRestored);
    return () => {
      window.removeEventListener("vlearning:data-restored", onRestored);
    };
  });
</script>

<svelte:window onkeydown={handleGlobalShortcut} />

{#if activeDeck}
  <div class="h-screen overflow-y-auto px-3 py-4 sm:px-6 sm:py-8">
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
      onOpenAbout={() => (showAbout = true)}
    />

    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Scrollable Main Workspace -->
      <main
        class="flex-1 overflow-y-auto px-3.5 pt-[max(0.75rem,env(safe-area-inset-top))] pb-20 sm:px-5 sm:pt-4 md:pb-4"
      >
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
            <AnalyticsView {decks} />
          {:else if activeTab === "decks"}
            <!-- Compact Daily Summary + Actions -->
            <div class="mb-5 flex flex-col gap-3 sm:flex-row sm:items-stretch sm:justify-between">
              <div class="grid flex-1 grid-cols-3 gap-2 sm:gap-3">
                <div
                  class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 px-3 py-2.5 sm:px-4 sm:py-3"
                >
                  <div class="flex items-center gap-1.5 text-[11px] font-medium text-zinc-400">
                    <Flame class="h-3.5 w-3.5 shrink-0 text-amber-400" />
                    <span class="truncate">Đã ôn hôm nay</span>
                  </div>
                  <p class="mt-1 text-xl font-bold text-white sm:text-2xl">
                    {stats.reviewedToday}
                  </p>
                </div>
                <div
                  class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 px-3 py-2.5 sm:px-4 sm:py-3"
                >
                  <div class="flex items-center gap-1.5 text-[11px] font-medium text-zinc-400">
                    <Sparkles class="h-3.5 w-3.5 shrink-0 text-sky-400" />
                    <span class="truncate">Từ mới hôm nay</span>
                  </div>
                  <p class="mt-1 text-xl font-bold text-sky-400 sm:text-2xl">
                    {stats.newLearnedToday}
                  </p>
                </div>
                <div
                  class="rounded-2xl border border-zinc-800/80 bg-zinc-900/60 px-3 py-2.5 sm:px-4 sm:py-3"
                >
                  <div class="flex items-center gap-1.5 text-[11px] font-medium text-zinc-400">
                    <Layers class="h-3.5 w-3.5 shrink-0 text-indigo-400" />
                    <span class="truncate">Cần ôn còn lại</span>
                  </div>
                  <p class="mt-1 text-xl font-bold text-indigo-400 sm:text-2xl">
                    {stats.dueRemaining}
                  </p>
                </div>
              </div>

              <div
                class="flex shrink-0 items-center justify-end gap-2 sm:flex-col sm:justify-center"
              >
                <button
                  type="button"
                  onclick={() => (showDeckModal = true)}
                  class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-sm transition hover:bg-indigo-500 sm:w-full sm:flex-none"
                >
                  <Plus class="h-3.5 w-3.5" />
                  Bộ thẻ mới
                </button>
                <label
                  class="inline-flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-xl border border-zinc-700/90 bg-zinc-900/60 px-3.5 py-2 text-xs font-medium text-zinc-300 transition hover:bg-zinc-800 sm:w-full sm:flex-none"
                >
                  <Upload class="h-3.5 w-3.5" />
                  Import .apkg
                  <input type="file" accept=".apkg" onchange={handleImportApkg} class="hidden" />
                </label>
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
          {#if activeTab === "games"}
            <MiniGameView {decks} />
          {/if}
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

<AboutModal open={showAbout} onClose={() => (showAbout = false)} />
