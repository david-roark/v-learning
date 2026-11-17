<script lang="ts">
  import { Plus, Search, Volume2 } from "@lucide/svelte";
  import { playWordAudio } from "$lib/shared/audio";
  import Select from "$lib/shared/components/Select.svelte";
  import type { DeckWithStats, DictionaryEntry } from "$lib/shared/types";

  interface Props {
    decks: DeckWithStats[];
    selectedDeckId: string;
    creatingDeck: boolean;
    newDeckName: string;
    manualQuery: string;
    lookingUp: boolean;
    lookupResult: DictionaryEntry | null;
    customMeaning: string;
    contextSentence: string;
    statusMsg: string | null;
    error: string | null;
    onQuickCreateDeck: (e: SubmitEvent) => void;
    onManualSearch: (e: SubmitEvent) => void;
    onSaveWithContext: () => void;
  }

  let {
    decks,
    selectedDeckId = $bindable(),
    creatingDeck = $bindable(),
    newDeckName = $bindable(),
    manualQuery = $bindable(),
    lookingUp,
    lookupResult,
    customMeaning = $bindable(),
    contextSentence = $bindable(),
    statusMsg,
    error,
    onQuickCreateDeck,
    onManualSearch,
    onSaveWithContext,
  }: Props = $props();
</script>

<div
  class="h-fit space-y-3 rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4 lg:sticky lg:top-0 lg:col-span-4 xl:col-span-3"
>
  <h3 class="text-sm font-semibold text-white">Tra cứu nhanh</h3>

  <div class="flex items-center gap-1.5">
    {#if decks.length > 0}
      <Select
        options={decks.map((d) => ({ value: d.id, label: d.name }))}
        bind:value={selectedDeckId}
        prefix="Lưu vào: "
        class="flex-1"
      />
    {:else}
      <span class="flex-1 text-xs text-zinc-400">Chưa có bộ thẻ nào</span>
    {/if}
    <button
      type="button"
      onclick={() => (creatingDeck = !creatingDeck)}
      title="Tạo nhanh bộ thẻ mới"
      class="shrink-0 rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500 hover:text-indigo-300"
    >
      <Plus class="h-3.5 w-3.5" />
    </button>
  </div>

  {#if creatingDeck}
    <form onsubmit={onQuickCreateDeck} class="flex gap-1.5">
      <input
        type="text"
        bind:value={newDeckName}
        placeholder="Tên bộ thẻ mới..."
        class="flex-1 rounded-xl border border-zinc-700 bg-zinc-950 px-2.5 py-1.5 text-xs text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
      />
      <button
        type="submit"
        disabled={!newDeckName.trim()}
        class="rounded-xl bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-40"
      >
        Tạo
      </button>
    </form>
  {/if}

  <form onsubmit={onManualSearch} class="flex gap-1.5">
    <div class="relative flex-1">
      <Search class="absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-zinc-500" />
      <input
        type="text"
        bind:value={manualQuery}
        placeholder="Nhập từ, cụm từ hoặc câu..."
        class="w-full rounded-xl border border-zinc-700 bg-zinc-950 py-2 pr-2.5 pl-8 text-xs text-white placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
      />
    </div>
    <button
      type="submit"
      disabled={lookingUp || !manualQuery.trim()}
      class="rounded-xl bg-zinc-800 px-3 py-2 text-xs font-medium text-zinc-200 hover:bg-indigo-600 hover:text-white disabled:opacity-40"
    >
      Tra
    </button>
  </form>

  {#if lookingUp}
    <p class="py-6 text-center text-xs text-zinc-400">Đang tra từ...</p>
  {:else if lookupResult}
    <div class="space-y-3 border-t border-zinc-800 pt-3">
      <div class="flex items-center justify-between">
        <div>
          <div class="flex flex-wrap items-center gap-1.5">
            <p class="text-lg font-bold text-white">{lookupResult.word}</p>
            {#if lookupResult.matchedForm}
              <span class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] text-amber-300">
                ← {lookupResult.matchedForm}
              </span>
            {/if}
          </div>
          <div class="flex flex-wrap gap-2 font-mono text-xs">
            {#if lookupResult.ipaUs}
              <span class="text-indigo-400">US {lookupResult.ipaUs}</span>
            {/if}
            {#if lookupResult.ipaUk && lookupResult.ipaUk !== lookupResult.ipaUs}
              <span class="text-sky-400">UK {lookupResult.ipaUk}</span>
            {:else if !lookupResult.ipaUs && lookupResult.ipa}
              <span class="text-indigo-400">{lookupResult.ipa}</span>
            {/if}
          </div>
        </div>
        <button
          type="button"
          onclick={() => playWordAudio(lookupResult!.word)}
          class="rounded-xl border border-zinc-700 p-2 text-zinc-300 hover:border-indigo-500"
        >
          <Volume2 class="h-4 w-4" />
        </button>
      </div>

      {#if lookupResult.senses.length > 0}
        <div class="max-h-56 space-y-2 overflow-y-auto pr-1">
          {#each lookupResult.senses as sense, sIdx (sIdx)}
            <div class="space-y-0.5 border-l-2 border-zinc-800 pl-2.5 text-xs">
              <div class="flex flex-wrap items-center gap-1">
                <span class="rounded bg-indigo-500/15 px-1 py-0.5 text-[10px] text-indigo-300">
                  {sense.pos}
                </span>
                {#each sense.tags as tag, tagIdx (tagIdx)}
                  <span class="rounded bg-zinc-800 px-1 py-0.5 text-[10px] text-zinc-400">
                    {tag}
                  </span>
                {/each}
              </div>
              {#if sense.definitionVi}
                <p class="text-emerald-400">{sense.definitionVi}</p>
              {/if}
              {#if sense.definitionEn}
                <p class="text-zinc-300">{sense.definitionEn}</p>
              {/if}
              {#if sense.synonyms.length > 0}
                <p class="text-[11px] text-zinc-400">
                  Đồng nghĩa: {sense.synonyms.join(", ")}
                </p>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-emerald-400">
          {lookupResult.meaningVi ?? lookupResult.meaningEn ?? "Chưa có nghĩa"}
        </p>
      {/if}

      <div>
        <label class="mb-1 block text-[11px] text-zinc-400" for="reader-meaning">
          Nghĩa / Bản dịch (có thể sửa trước khi lưu)
        </label>
        <textarea
          id="reader-meaning"
          rows="2"
          bind:value={customMeaning}
          placeholder="Nhập hoặc chỉnh sửa nghĩa..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2.5 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        ></textarea>
      </div>

      <div>
        <label class="mb-1 block text-[11px] text-zinc-400" for="reader-context">
          Câu ngữ cảnh (tuỳ chọn)
        </label>
        <textarea
          id="reader-context"
          rows="2"
          bind:value={contextSentence}
          placeholder="Nhập hoặc sửa câu ví dụ..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2.5 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        ></textarea>
      </div>

      <button
        type="button"
        onclick={onSaveWithContext}
        disabled={!selectedDeckId}
        class="flex w-full items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
      >
        <Plus class="h-3.5 w-3.5" />
        Lưu vào bộ thẻ
      </button>
    </div>
  {:else if manualQuery.trim()}
    <div class="space-y-3 border-t border-zinc-800 pt-3">
      <p class="text-sm font-semibold text-white">{manualQuery.trim()}</p>
      <div>
        <label class="mb-1 block text-[11px] text-zinc-400" for="reader-meaning-fallback">
          Nghĩa / Bản dịch
        </label>
        <textarea
          id="reader-meaning-fallback"
          rows="2"
          bind:value={customMeaning}
          placeholder="Nhập nghĩa hoặc bản dịch cho cụm từ / câu này..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950 p-2.5 text-xs text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
        ></textarea>
      </div>
      <button
        type="button"
        onclick={onSaveWithContext}
        disabled={!selectedDeckId}
        class="flex w-full items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white hover:bg-indigo-500 disabled:opacity-40"
      >
        <Plus class="h-3.5 w-3.5" />
        Lưu vào bộ thẻ
      </button>
    </div>
  {:else}
    <p class="py-6 text-center text-xs text-zinc-500">
      Click 1 từ hoặc bôi đen cả cụm từ / nguyên câu để tra nghĩa và lưu thành flashcard.
    </p>
  {/if}

  {#if statusMsg}
    <p class="rounded-xl bg-emerald-500/10 p-2.5 text-xs text-emerald-400">{statusMsg}</p>
  {/if}
  {#if error}
    <p class="rounded-xl bg-rose-500/10 p-2.5 text-xs text-rose-400">{error}</p>
  {/if}
</div>
