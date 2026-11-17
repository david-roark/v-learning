<script lang="ts">
  import { AlertTriangle, Pencil, Search, Trash2, X } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { notesApi } from "$lib/features/card-editor/api";
  import NoteFormModal from "$lib/features/card-editor/components/NoteFormModal.svelte";
  import type { DeckWithStats, Note } from "$lib/shared/types";

  interface Props {
    deck: DeckWithStats;
    onClose: () => void;
    onChanged: () => void;
  }

  let { deck, onClose, onChanged }: Props = $props();

  let notes = $state<Note[]>([]);
  let loading = $state(true);
  let searchQuery = $state("");
  let editingNote = $state<Note | null>(null);
  let confirmDeleteNote = $state<Note | null>(null);

  const filteredNotes = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return notes;
    return notes.filter(
      (n) =>
        n.word.toLowerCase().includes(q) ||
        n.meaning.toLowerCase().includes(q) ||
        (n.exampleSentence?.toLowerCase().includes(q) ?? false),
    );
  });

  async function loadNotes() {
    loading = true;
    try {
      notes = await notesApi.listByDeck(deck.id);
    } finally {
      loading = false;
    }
  }

  async function handleConfirmDelete() {
    if (!confirmDeleteNote) return;
    const id = confirmDeleteNote.id;
    confirmDeleteNote = null;
    await notesApi.remove(id);
    notes = notes.filter((n) => n.id !== id);
    onChanged();
  }

  function handleNoteSaved(updated?: Note) {
    if (updated) {
      notes = notes.map((n) => (n.id === updated.id ? updated : n));
    }
    editingNote = null;
    onChanged();
  }

  onMount(() => {
    loadNotes();
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") {
      if (confirmDeleteNote) {
        confirmDeleteNote = null;
      } else if (!editingNote) {
        onClose();
      }
    }
  }}
/>

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div
    role="dialog"
    aria-modal="true"
    class="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-2xl border border-zinc-800 bg-zinc-900 shadow-2xl"
  >
    <div class="flex items-center justify-between border-b border-zinc-800 px-6 py-4">
      <div>
        <h2 class="text-base font-semibold text-white">Danh sách thẻ — {deck.name}</h2>
        <p class="text-xs text-zinc-400">
          {filteredNotes.length === notes.length
            ? `${notes.length} thẻ trong bộ này`
            : `${filteredNotes.length}/${notes.length} thẻ khớp`}
        </p>
      </div>
      <button
        type="button"
        onclick={onClose}
        class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    {#if !loading && notes.length > 0}
      <div class="border-b border-zinc-800/80 px-6 py-2.5">
        <div class="relative">
          <Search class="absolute top-1/2 left-3 h-3.5 w-3.5 -translate-y-1/2 text-zinc-500" />
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Tìm từ, nghĩa hoặc câu ví dụ..."
            class="w-full rounded-xl border border-zinc-800 bg-zinc-950/70 py-1.5 pr-3 pl-8 text-xs text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
          />
        </div>
      </div>
    {/if}

    <div class="flex-1 space-y-2.5 overflow-y-auto p-6">
      {#if loading}
        <p class="py-8 text-center text-sm text-zinc-500">Đang tải thẻ...</p>
      {:else if notes.length === 0}
        <p class="py-8 text-center text-sm text-zinc-500">Bộ thẻ này chưa có thẻ nào.</p>
      {:else if filteredNotes.length === 0}
        <p class="py-8 text-center text-sm text-zinc-500">Không tìm thấy thẻ nào phù hợp.</p>
      {:else}
        {#each filteredNotes as note (note.id)}
          <div
            class="flex items-start justify-between gap-3 rounded-xl border border-zinc-800 bg-zinc-950/70 p-3.5"
          >
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
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
            <div class="flex shrink-0 items-center gap-1">
              <button
                type="button"
                onclick={() => (editingNote = note)}
                title="Sửa thẻ"
                class="rounded-lg p-1.5 text-zinc-400 hover:bg-indigo-500/10 hover:text-indigo-400"
              >
                <Pencil class="h-4 w-4" />
              </button>
              <button
                type="button"
                onclick={() => (confirmDeleteNote = note)}
                title="Xoá thẻ"
                class="rounded-lg p-1.5 text-zinc-400 hover:bg-rose-500/10 hover:text-rose-400"
              >
                <Trash2 class="h-4 w-4" />
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

{#if editingNote}
  <NoteFormModal
    open={true}
    deckId={deck.id}
    deckName={deck.name}
    editNote={editingNote}
    onClose={() => (editingNote = null)}
    onSaved={handleNoteSaved}
  />
{/if}

{#if confirmDeleteNote}
  <div
    class="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 p-4 backdrop-blur-xs"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) confirmDeleteNote = null;
    }}
  >
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="delete-note-title"
      class="w-full max-w-sm rounded-2xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl"
    >
      <div class="flex items-start gap-3">
        <div class="shrink-0 rounded-xl bg-rose-500/15 p-2.5 text-rose-400">
          <AlertTriangle class="h-5 w-5" />
        </div>
        <div class="min-w-0">
          <h3 id="delete-note-title" class="truncate text-base font-semibold text-white">
            Xoá từ "{confirmDeleteNote.word}"?
          </h3>
          <p class="mt-1.5 text-xs leading-relaxed text-zinc-400">
            Thẻ và lịch sử ôn tập của từ này sẽ bị xoá vĩnh viễn và không thể hoàn tác.
          </p>
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-2.5">
        <button
          type="button"
          onclick={() => (confirmDeleteNote = null)}
          class="rounded-xl border border-zinc-700 px-4 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
        >
          Huỷ
        </button>
        <button
          type="button"
          onclick={handleConfirmDelete}
          class="rounded-xl bg-rose-600 px-4 py-2 text-xs font-semibold text-white hover:bg-rose-500"
        >
          Xoá vĩnh viễn
        </button>
      </div>
    </div>
  </div>
{/if}
