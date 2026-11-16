<script lang="ts">
  import { BookOpenText, Plus, Upload, Volume2 } from "@lucide/svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import { dictionaryApi } from "$lib/features/dictionary/api";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import type { DeckWithStats, DictionaryEntry } from "$lib/shared/types";
  import { stripSubtitleMetadata, tokenizeWithSentences, type TextToken } from "./textParser";

  interface Props {
    decks: DeckWithStats[];
    onCardAdded: () => void;
  }

  let { decks, onCardAdded }: Props = $props();

  let articleText = $state(
    "Smartphones have become ubiquitous in modern life. Taking a pragmatic approach helps learners stay resilient when facing ambiguous grammar rules.",
  );
  let editing = $state(false);
  let selectedDeckId = $state("");
  let selectedToken = $state<TextToken | null>(null);
  let lookupResult = $state<DictionaryEntry | null>(null);
  let lookingUp = $state(false);
  let statusMsg = $state("");
  let error = $state("");

  $effect(() => {
    if (!selectedDeckId && decks.length > 0) {
      selectedDeckId = decks[0].id;
    }
  });

  const tokens = $derived(tokenizeWithSentences(articleText));

  async function handleWordClick(tok: TextToken) {
    if (!tok.isWord) return;
    selectedToken = tok;
    lookingUp = true;
    error = "";
    statusMsg = "";
    lookupResult = null;
    try {
      lookupResult = await dictionaryApi.lookup(tok.text);
    } catch (err) {
      error = String(err);
    } finally {
      lookingUp = false;
    }
  }

  async function handleSaveWithContext() {
    if (!lookupResult || !selectedToken || !selectedDeckId) return;
    error = "";
    try {
      await notesApi.create({
        deckId: selectedDeckId,
        noteType: "vocab",
        word: lookupResult.word,
        ipa: lookupResult.ipa ?? undefined,
        meaning: lookupResult.meaningVi ?? lookupResult.meaningEn ?? lookupResult.word,
        exampleSentence: selectedToken.sentence,
      });
      statusMsg = `Đã lưu "${lookupResult.word}" kèm câu ngữ cảnh!`;
      onCardAdded();
    } catch (err) {
      error = String(err);
    }
  }

  async function handleFileUpload(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const raw = await file.text();
    const isSub = /\.(srt|vtt)$/i.test(file.name);
    articleText = isSub ? stripSubtitleMetadata(raw) : raw;
    editing = false;
    input.value = "";
  }
</script>

<div class="grid gap-5 lg:grid-cols-3">
  <div class="space-y-4 lg:col-span-2">
    <div
      class="flex flex-wrap items-center justify-between gap-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4"
    >
      <div class="flex items-center gap-2.5">
        <BookOpenText class="h-5 w-5 text-indigo-400" />
        <div>
          <h2 class="text-sm font-semibold text-white">
            Trình đọc & Khai thác câu (Sentence Mining)
          </h2>
          <p class="text-xs text-zinc-400">
            Click vào bất kỳ từ nào để tra và lưu kèm câu ngữ cảnh
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <label
          class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
        >
          <Upload class="h-3.5 w-3.5" />
          Mở .txt/.srt
          <input
            type="file"
            accept=".txt,.md,.srt,.vtt"
            onchange={handleFileUpload}
            class="hidden"
          />
        </label>
        <button
          type="button"
          onclick={() => (editing = !editing)}
          class="rounded-xl border border-zinc-700 px-3 py-1.5 text-xs font-medium text-zinc-200 hover:bg-zinc-800"
        >
          {editing ? "Xem để đọc" : "Dán văn bản"}
        </button>
      </div>
    </div>

    {#if editing}
      <textarea
        rows="12"
        bind:value={articleText}
        placeholder="Dán đoạn văn hoặc bài báo tiếng Anh vào đây..."
        class="w-full rounded-2xl border border-zinc-700 bg-zinc-900 p-4 text-sm leading-relaxed text-zinc-100 focus:border-indigo-500 focus:outline-none"
      ></textarea>
    {:else}
      <div
        class="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-base leading-8 text-zinc-200"
      >
        {#each tokens as tok}
          {#if tok.isWord}
            <button
              type="button"
              onclick={() => handleWordClick(tok)}
              class="rounded px-0.5 transition hover:bg-indigo-500/25 hover:text-indigo-200 {selectedToken?.text ===
              tok.text
                ? 'bg-indigo-600/30 text-indigo-200'
                : ''}"
            >
              {tok.text}
            </button>
          {:else}
            <span>{tok.text}</span>
          {/if}
        {/each}
      </div>
    {/if}
  </div>

  <!-- Side Lookup Panel -->
  <div class="space-y-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
    <h3 class="text-sm font-semibold text-white">Tra cứu nhanh</h3>

    {#if decks.length > 0}
      <Select
        options={decks.map((d) => ({ value: d.id, label: d.name }))}
        bind:value={selectedDeckId}
        prefix="Lưu vào: "
      />
    {/if}

    {#if lookingUp}
      <p class="py-6 text-center text-xs text-zinc-400">Đang tra từ...</p>
    {:else if lookupResult && selectedToken}
      <div class="space-y-3 border-t border-zinc-800 pt-3">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-lg font-bold text-white">{lookupResult.word}</p>
            {#if lookupResult.ipa}
              <p class="font-mono text-xs text-indigo-400">{lookupResult.ipa}</p>
            {/if}
          </div>
          <button
            type="button"
            onclick={() => playWordAudio(lookupResult!.word)}
            class="rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500"
          >
            <Volume2 class="h-4 w-4" />
          </button>
        </div>

        <p class="text-sm text-emerald-400">
          {lookupResult.meaningVi ?? lookupResult.meaningEn ?? "Chưa có nghĩa"}
        </p>

        <div class="rounded-xl bg-zinc-950 p-3 text-xs text-zinc-300 italic">
          "{selectedToken.sentence}"
        </div>

        <button
          type="button"
          onclick={handleSaveWithContext}
          disabled={!selectedDeckId}
          class="flex w-full items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
        >
          <Plus class="h-3.5 w-3.5" />
          Lưu thẻ kèm câu này
        </button>
      </div>
    {:else}
      <p class="py-6 text-center text-xs text-zinc-500">
        Click vào một từ bên trái để xem nghĩa và lưu thành flashcard.
      </p>
    {/if}

    {#if statusMsg}
      <p class="rounded-xl bg-emerald-500/10 p-2.5 text-xs text-emerald-400">{statusMsg}</p>
    {/if}
    {#if error}
      <p class="rounded-xl bg-rose-500/10 p-2.5 text-xs text-rose-400">{error}</p>
    {/if}
  </div>
</div>
