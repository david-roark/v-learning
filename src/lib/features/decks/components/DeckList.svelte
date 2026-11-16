<script lang="ts">
  import { BookOpen, List, Play, Plus, Trash2 } from "@lucide/svelte";
  import type { DeckWithStats } from "$lib/shared/types";

  interface Props {
    decks: DeckWithStats[];
    onStudy: (deck: DeckWithStats) => void;
    onAddCard: (deck: DeckWithStats) => void;
    onManageNotes: (deck: DeckWithStats) => void;
    onDelete: (deck: DeckWithStats) => void;
    onCreateDeck: () => void;
  }

  let { decks, onStudy, onAddCard, onManageNotes, onDelete, onCreateDeck }: Props = $props();
</script>

{#if decks.length === 0}
  <div
    class="flex flex-col items-center justify-center rounded-2xl border border-dashed border-zinc-800 bg-zinc-900/40 px-6 py-16 text-center"
  >
    <div class="mb-4 rounded-2xl bg-indigo-500/10 p-4 text-indigo-400">
      <BookOpen class="h-8 w-8" />
    </div>
    <h3 class="text-lg font-semibold text-zinc-100">Chưa có bộ thẻ nào</h3>
    <p class="mt-1.5 max-w-sm text-sm text-zinc-400">
      Tạo bộ thẻ đầu tiên để bắt đầu học tiếng Anh với thuật toán lặp lại ngắt quãng FSRS.
    </p>
    <button
      type="button"
      onclick={onCreateDeck}
      class="mt-6 inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-indigo-500"
    >
      <Plus class="h-4 w-4" />
      Tạo bộ thẻ đầu tiên
    </button>
  </div>
{:else}
  <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
    {#each decks as deck (deck.id)}
      {@const readyCount = deck.newCount + deck.dueCount}
      <div
        class="group flex flex-col justify-between rounded-2xl border border-zinc-800/90 bg-zinc-900/70 p-5 transition hover:border-zinc-700 hover:bg-zinc-900"
      >
        <div>
          <div class="flex items-start justify-between gap-2">
            <h3 class="text-base font-semibold text-zinc-100">{deck.name}</h3>
            <button
              type="button"
              onclick={() => onDelete(deck)}
              title="Xoá bộ thẻ"
              class="rounded-lg p-1.5 text-zinc-500 opacity-0 transition group-hover:opacity-100 hover:bg-rose-500/10 hover:text-rose-400"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
          {#if deck.description}
            <p class="mt-1 line-clamp-2 text-xs text-zinc-400">{deck.description}</p>
          {/if}

          <div class="mt-4 flex items-center gap-3 text-xs">
            <span class="rounded-lg bg-sky-500/10 px-2.5 py-1 font-medium text-sky-400">
              {deck.newCount} mới
            </span>
            <span class="rounded-lg bg-amber-500/10 px-2.5 py-1 font-medium text-amber-400">
              {deck.dueCount} cần ôn
            </span>
            <span class="ml-auto text-zinc-500">{deck.totalCards} tổng</span>
          </div>
        </div>

        <div class="mt-5 flex items-center gap-2 border-t border-zinc-800/80 pt-4">
          <button
            type="button"
            onclick={() => onStudy(deck)}
            disabled={readyCount === 0}
            class="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-indigo-600 px-3 py-2 text-xs font-semibold text-white transition hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-zinc-800 disabled:text-zinc-500"
          >
            <Play class="h-3.5 w-3.5" />
            {readyCount > 0 ? `Học ngay (${readyCount})` : "Đã xong hôm nay"}
          </button>
          <button
            type="button"
            onclick={() => onAddCard(deck)}
            title="Thêm thẻ mới"
            class="inline-flex items-center gap-1 rounded-xl border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-300 hover:bg-zinc-800"
          >
            <Plus class="h-3.5 w-3.5" />
            Thẻ
          </button>
          <button
            type="button"
            onclick={() => onManageNotes(deck)}
            title="Xem danh sách thẻ"
            class="inline-flex items-center rounded-xl border border-zinc-700 p-2 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
          >
            <List class="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
