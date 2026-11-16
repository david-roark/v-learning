<script lang="ts">
  import { Trash2, X } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import type { DeckWithStats, Note } from "$lib/shared/types";

  interface Props {
    deck: DeckWithStats;
    onClose: () => void;
    onChanged: () => void;
  }

  let { deck, onClose, onChanged }: Props = $props();

  let notes = $state<Note[]>([]);
  let loading = $state(true);

  async function loadNotes() {
    loading = true;
    try {
      notes = await notesApi.listByDeck(deck.id);
    } finally {
      loading = false;
    }
  }

  async function handleDelete(noteId: string) {
    await notesApi.remove(noteId);
    await loadNotes();
    onChanged();
  }

  onMount(() => {
    loadNotes();
  });
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs">
  <div
    class="flex max-h-[80vh] w-full max-w-2xl flex-col rounded-2xl border border-zinc-800 bg-zinc-900 shadow-2xl"
  >
    <div class="flex items-center justify-between border-b border-zinc-800 px-6 py-4">
      <div>
        <h2 class="text-base font-semibold text-white">Danh sách thẻ — {deck.name}</h2>
        <p class="text-xs text-zinc-400">{notes.length} thẻ trong bộ này</p>
      </div>
      <button
        type="button"
        onclick={onClose}
        class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <div class="flex-1 space-y-2.5 overflow-y-auto p-6">
      {#if loading}
        <p class="py-8 text-center text-sm text-zinc-500">Đang tải thẻ...</p>
      {:else if notes.length === 0}
        <p class="py-8 text-center text-sm text-zinc-500">Bộ thẻ này chưa có thẻ nào.</p>
      {:else}
        {#each notes as note (note.id)}
          <div
            class="flex items-start justify-between gap-3 rounded-xl border border-zinc-800 bg-zinc-950/70 p-3.5"
          >
            <div>
              <div class="flex items-center gap-2">
                <span class="font-semibold text-white">{note.word}</span>
                {#if note.ipa}
                  <span class="font-mono text-xs text-indigo-400">{note.ipa}</span>
                {/if}
                <span class="rounded bg-zinc-800 px-1.5 py-0.5 text-[10px] text-zinc-400">
                  {note.noteType}
                </span>
              </div>
              <p class="mt-0.5 text-xs text-emerald-400">{note.meaning}</p>
              {#if note.exampleSentence}
                <p class="mt-1 text-xs text-zinc-400 italic">"{note.exampleSentence}"</p>
              {/if}
            </div>
            <button
              type="button"
              onclick={() => handleDelete(note.id)}
              title="Xoá thẻ"
              class="rounded-lg p-1.5 text-zinc-500 hover:bg-rose-500/10 hover:text-rose-400"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
