<script lang="ts">
  import { Plus, Search, Volume2 } from "@lucide/svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import { decksApi } from "$lib/features/decks/api";
  import { dictionaryApi } from "$lib/features/dictionary/api";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import { invokeCmd } from "$lib/shared/tauri";
  import type { DeckWithStats, DictionaryEntry } from "$lib/shared/types";

  let query = $state("");
  let inputEl = $state<HTMLInputElement | null>(null);
  let entry = $state<DictionaryEntry | null>(null);
  let decks = $state<DeckWithStats[]>([]);
  let selectedDeckId = $state("");
  let loading = $state(false);
  let savedMsg = $state("");
  let error = $state("");

  async function handleLookup(e?: Event) {
    e?.preventDefault();
    if (!query.trim()) return;
    loading = true;
    error = "";
    savedMsg = "";
    try {
      entry = await dictionaryApi.lookup(query.trim());
    } catch (err) {
      entry = null;
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function handleSaveToDeck() {
    if (!entry || !selectedDeckId) return;
    try {
      await notesApi.create({
        deckId: selectedDeckId,
        noteType: "vocab",
        word: entry.word,
        ipa: entry.ipa ?? undefined,
        meaning: entry.meaningVi ?? entry.meaningEn ?? entry.word,
        exampleSentence: entry.exampleSentence ?? undefined,
      });
      savedMsg = `Đã lưu "${entry.word}" vào bộ thẻ!`;
    } catch (err) {
      error = String(err);
    }
  }

  async function refreshOnOpen() {
    inputEl?.focus();
    decks = await decksApi.list();
    if (decks.length > 0 && !decks.some((d) => d.id === selectedDeckId)) {
      selectedDeckId = decks[0].id;
    }

    try {
      const clip = (await invokeCmd<string>("read_clipboard_text")).trim();
      if (clip && clip.length <= 80 && !clip.includes("\n") && clip !== query) {
        query = clip;
        await handleLookup();
      }
    } catch {
      // ignore clipboard errors
    }
  }

  onMount(() => {
    void refreshOnOpen();
    let unlisten: (() => void) | null = null;
    void listen("quick-lookup-opened", () => {
      void refreshOnOpen();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  });
</script>

<div class="flex min-h-screen flex-col justify-between bg-zinc-950 p-5 text-zinc-100">
  <div class="space-y-4">
    <form onsubmit={handleLookup} class="flex gap-2">
      <div class="relative flex-1">
        <Search class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-zinc-500" />
        <input
          bind:this={inputEl}
          type="text"
          bind:value={query}
          placeholder="Nhập từ tiếng Anh để tra nhanh..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-900 py-2.5 pr-3 pl-9 text-sm text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        />
      </div>
      <button
        type="submit"
        disabled={loading}
        class="rounded-xl bg-indigo-600 px-4 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-50"
      >
        {loading ? "Đang tra..." : "Tra từ"}
      </button>
    </form>

    {#if error}
      <p class="rounded-xl bg-rose-500/10 p-3 text-xs text-rose-400">{error}</p>
    {/if}

    {#if entry}
      <div class="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-2.5">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-xl font-bold text-white">{entry.word}</h2>
            <div class="flex items-center gap-2 text-xs text-zinc-400">
              {#if entry.ipa}
                <span class="font-mono text-indigo-400">{entry.ipa}</span>
              {/if}
              {#if entry.pos}
                <span class="rounded bg-zinc-800 px-1.5 py-0.5">{entry.pos}</span>
              {/if}
            </div>
          </div>
          <button
            type="button"
            onclick={() => playWordAudio(entry!.word)}
            class="rounded-xl border border-zinc-700 bg-zinc-800 p-2 text-zinc-300 hover:border-indigo-500 hover:text-indigo-300"
          >
            <Volume2 class="h-4 w-4" />
          </button>
        </div>

        <p class="text-sm font-medium text-emerald-400">
          {entry.meaningVi ?? entry.meaningEn ?? "Chưa có nghĩa"}
        </p>

        {#if entry.meaningVi && entry.meaningEn}
          <p class="text-xs text-zinc-400">{entry.meaningEn}</p>
        {/if}

        {#if entry.exampleSentence}
          <p class="text-xs text-zinc-300 italic">"{entry.exampleSentence}"</p>
        {/if}
      </div>
    {/if}
  </div>

  {#if entry}
    <div class="mt-4 space-y-2 border-t border-zinc-800 pt-3">
      {#if savedMsg}
        <p class="text-xs font-medium text-emerald-400">{savedMsg}</p>
      {/if}
      <div class="flex items-center gap-2">
        <Select
          options={decks.map((d) => ({ value: d.id, label: d.name }))}
          bind:value={selectedDeckId}
          placeholder="Chọn bộ thẻ..."
          class="flex-1"
        />
        <button
          type="button"
          onclick={handleSaveToDeck}
          disabled={!selectedDeckId}
          class="inline-flex items-center gap-1.5 rounded-xl bg-emerald-600 px-3.5 py-2 text-xs font-semibold text-white hover:bg-emerald-500 disabled:opacity-40"
        >
          <Plus class="h-3.5 w-3.5" />
          Lưu thành thẻ
        </button>
      </div>
    </div>
  {/if}
</div>
