<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { BookA, Download, Globe, Plus, Search, Upload, Volume2 } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import type {
    DeckWithStats,
    DictionaryEntry,
    DictionaryPack,
    ImportProgress,
  } from "$lib/shared/types";
  import { dictionaryApi } from "../api";

  interface Props {
    decks: DeckWithStats[];
    onCardAdded: () => void;
  }

  let { decks, onCardAdded }: Props = $props();

  let query = $state("");
  let entries = $state<DictionaryEntry[]>([]);
  let packs = $state<DictionaryPack[]>([]);
  let totalCount = $state(0);
  let selectedDeckId = $state("");
  let loading = $state(false);
  let downloadingPackId = $state<string | null>(null);
  let progress = $state<ImportProgress | null>(null);
  let showImport = $state(false);
  let statusMsg = $state("");
  let error = $state("");
  let unlisten: (() => void) | null = null;

  $effect(() => {
    if (!selectedDeckId && decks.length > 0) {
      selectedDeckId = decks[0].id;
    }
  });

  async function loadList() {
    loading = true;
    error = "";
    try {
      const [list, count] = await Promise.all([
        dictionaryApi.search(query.trim(), 40),
        dictionaryApi.count(),
      ]);
      entries = list;
      totalCount = count;
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function handleOnlineLookup(e: Event) {
    e.preventDefault();
    if (!query.trim()) return;
    loading = true;
    error = "";
    statusMsg = "";
    try {
      const entry = await dictionaryApi.lookup(query.trim());
      entries = [entry, ...entries.filter((x) => x.id !== entry.id)];
      totalCount = await dictionaryApi.count();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function handleSaveCard(entry: DictionaryEntry) {
    if (!selectedDeckId) {
      error = "Hãy tạo ít nhất 1 bộ thẻ trước khi lưu.";
      return;
    }
    error = "";
    statusMsg = "";
    try {
      await notesApi.create({
        deckId: selectedDeckId,
        noteType: "vocab",
        word: entry.word,
        ipa: entry.ipa ?? undefined,
        meaning: entry.meaningVi ?? entry.meaningEn ?? entry.word,
        exampleSentence: entry.exampleSentence ?? undefined,
      });
      statusMsg = `Đã thêm "${entry.word}" vào bộ thẻ!`;
      onCardAdded();
    } catch (err) {
      error = String(err);
    }
  }

  async function handleFileUpload(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    loading = true;
    error = "";
    try {
      const text = await file.text();
      const imported = await dictionaryApi.importTsv(text);
      statusMsg = `Đã import thành công ${imported} mục từ vào từ điển offline.`;
      await loadList();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
      input.value = "";
    }
  }

  async function handleDownloadPack(pack: DictionaryPack) {
    downloadingPackId = pack.id;
    progress = {
      stage: "downloading",
      current: 0,
      total: 0,
      message: `Đang kết nối để tải ${pack.name}...`,
    };
    error = "";
    statusMsg = "";
    try {
      const report = await dictionaryApi.downloadPack(pack.id);
      statusMsg = `Hoàn tất "${pack.name}": +${report.inserted.toLocaleString()} mới · ${report.updated.toLocaleString()} cập nhật · ${report.skipped.toLocaleString()} bỏ qua trùng (Tổng: ${report.totalAfter.toLocaleString()} từ).`;
      await loadList();
    } catch (err) {
      statusMsg = "";
      error = String(err);
    } finally {
      downloadingPackId = null;
      progress = null;
    }
  }

  onMount(async () => {
    unlisten = await listen<ImportProgress>("dict-import-progress", (ev) => {
      progress = ev.payload;
    });
    packs = await dictionaryApi.listPacks();
    await loadList();
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div class="space-y-5">
  <div
    class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4"
  >
    <div class="flex items-center gap-3">
      <div class="rounded-xl bg-indigo-500/15 p-2.5 text-indigo-400">
        <BookA class="h-5 w-5" />
      </div>
      <div>
        <h2 class="text-sm font-semibold text-white">Từ điển Offline & Online</h2>
        <p class="text-xs text-zinc-400">
          Hiện có <strong class="text-zinc-200">{totalCount}</strong> từ trong SQLite local (tự động cache
          khi tra online)
        </p>
      </div>
    </div>

    <div class="flex items-center gap-2">
      {#if decks.length > 0}
        <Select
          options={decks.map((d) => ({ value: d.id, label: d.name }))}
          bind:value={selectedDeckId}
          prefix="Lưu vào: "
          class="min-w-[180px]"
        />
      {/if}
      <button
        type="button"
        onclick={() => (showImport = !showImport)}
        class="inline-flex items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
      >
        <Download class="h-3.5 w-3.5" />
        Tải / Import từ điển
      </button>
    </div>
  </div>

  {#if showImport}
    <div
      class="space-y-4 rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 text-xs text-zinc-300"
    >
      <div>
        <p class="font-semibold text-white">Gói từ điển Offline có sẵn (Tải 1 chạm)</p>
        <div class="mt-2.5 grid gap-2.5 sm:grid-cols-2">
          {#each packs as pack (pack.id)}
            <div
              class="flex flex-col justify-between rounded-xl border border-zinc-800 bg-zinc-950/70 p-3.5"
            >
              <div>
                <div class="flex items-center justify-between gap-2">
                  <span class="font-medium text-zinc-100">{pack.name}</span>
                  <span
                    class="shrink-0 rounded bg-zinc-800 px-1.5 py-0.5 text-[10px] text-zinc-400"
                  >
                    {pack.sizeHint}
                  </span>
                </div>
                <p class="mt-1 text-[11px] text-zinc-400">{pack.description}</p>
              </div>
              <button
                type="button"
                disabled={downloadingPackId !== null}
                onclick={() => handleDownloadPack(pack)}
                class="mt-3 inline-flex items-center justify-center gap-1.5 rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
              >
                <Download class="h-3.5 w-3.5" />
                {downloadingPackId === pack.id ? "Đang tải & import..." : "Tải & Cài đặt"}
              </button>
            </div>
          {/each}
        </div>
      </div>

      <div class="border-t border-zinc-800 pt-3">
        <p class="font-medium text-white">Hoặc Import file thủ công (.tsv / .txt)</p>
        <p class="mt-1 text-zinc-400">
          Định dạng mỗi dòng: <code class="rounded bg-zinc-950 px-1.5 py-0.5 text-indigo-300"
            >word[TAB]ipa[TAB]pos[TAB]meaning_vi[TAB]example</code
          >
        </p>
        <label
          class="mt-2.5 inline-flex cursor-pointer items-center gap-2 rounded-xl border border-zinc-700 px-3.5 py-2 font-medium text-zinc-200 hover:bg-zinc-800"
        >
          <Upload class="h-3.5 w-3.5" />
          Chọn file .tsv / .txt
          <input type="file" accept=".tsv,.txt" onchange={handleFileUpload} class="hidden" />
        </label>
      </div>
    </div>
  {/if}

  <form onsubmit={handleOnlineLookup} class="flex gap-2">
    <div class="relative flex-1">
      <Search class="absolute top-1/2 left-3.5 h-4 w-4 -translate-y-1/2 text-zinc-500" />
      <input
        type="text"
        bind:value={query}
        oninput={loadList}
        placeholder="Tìm trong từ điển local hoặc nhấn Enter để tra Online..."
        class="w-full rounded-xl border border-zinc-700 bg-zinc-900 py-2.5 pr-4 pl-10 text-sm text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
      />
    </div>
    <button
      type="submit"
      disabled={loading || !query.trim()}
      class="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
    >
      <Globe class="h-3.5 w-3.5" />
      Tra Online
    </button>
  </form>

  {#if progress}
    {@const pct =
      progress.total > 0 ? Math.min(100, Math.round((progress.current / progress.total) * 100)) : 0}
    <div class="space-y-2 rounded-xl border border-indigo-500/30 bg-indigo-500/10 p-3.5 text-xs">
      <div class="flex items-center justify-between text-indigo-200">
        <span>{progress.message}</span>
        {#if progress.total > 0}
          <span class="font-mono font-semibold">{pct}%</span>
        {/if}
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-zinc-800">
        <div
          class="h-full rounded-full bg-indigo-500 transition-all duration-200 {progress.total === 0
            ? 'w-1/3 animate-pulse'
            : ''}"
          style={progress.total > 0 ? `width: ${pct}%` : ""}
        ></div>
      </div>
    </div>
  {/if}

  {#if statusMsg}
    <p class="rounded-xl bg-emerald-500/10 px-3.5 py-2 text-xs text-emerald-400">{statusMsg}</p>
  {/if}
  {#if error}
    <p class="rounded-xl bg-rose-500/10 px-3.5 py-2 text-xs text-rose-400">{error}</p>
  {/if}

  <div class="space-y-2.5">
    {#each entries as entry (entry.id)}
      <div
        class="flex items-start justify-between gap-4 rounded-2xl border border-zinc-800/80 bg-zinc-900/60 p-4"
      >
        <div class="space-y-1">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-base font-bold text-white">{entry.word}</span>
            {#if entry.ipa}
              <span class="font-mono text-xs text-indigo-400">{entry.ipa}</span>
            {/if}
            {#if entry.pos}
              <span class="rounded bg-zinc-800 px-1.5 py-0.5 text-[11px] text-zinc-400"
                >{entry.pos}</span
              >
            {/if}
            <span class="rounded bg-zinc-800/70 px-1.5 py-0.5 text-[10px] text-zinc-500"
              >{entry.source}</span
            >
          </div>
          <p class="text-sm text-emerald-400">{entry.meaningVi ?? entry.meaningEn ?? "—"}</p>
          {#if entry.meaningVi && entry.meaningEn}
            <p class="text-xs text-zinc-400">{entry.meaningEn}</p>
          {/if}
          {#if entry.exampleSentence}
            <p class="text-xs text-zinc-400 italic">"{entry.exampleSentence}"</p>
          {/if}
        </div>

        <div class="flex shrink-0 items-center gap-2">
          <button
            type="button"
            onclick={() => playWordAudio(entry.word)}
            title="Phát âm"
            class="rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500 hover:text-indigo-300"
          >
            <Volume2 class="h-4 w-4" />
          </button>
          <button
            type="button"
            onclick={() => handleSaveCard(entry)}
            class="inline-flex items-center gap-1 rounded-xl bg-zinc-800 px-3 py-2 text-xs font-medium text-zinc-200 hover:bg-indigo-600 hover:text-white"
          >
            <Plus class="h-3.5 w-3.5" />
            Lưu thẻ
          </button>
        </div>
      </div>
    {:else}
      <p class="py-10 text-center text-sm text-zinc-500">
        Không tìm thấy từ trong local. Nhấn <strong>Tra Online</strong> để tải và lưu vào máy.
      </p>
    {/each}
  </div>
</div>
